#!/usr/bin/env python3
"""Verify that the published registry version is the exact tested tarball."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import socket
import sys
import time
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import urlopen

REGISTRY_ATTEMPTS = 61
REGISTRY_PROPAGATION_SECONDS = 300
REGISTRY_RETRY_SECONDS = 5
REGISTRY_TIMEOUT_SECONDS = 10
RETRYABLE_HTTP_STATUS = {404, 429, 500, 502, 503, 504}


def release_tag(version):
    number = r"(?:0|[1-9][0-9]*)"
    match = re.fullmatch(rf"{number}\.{number}\.{number}(-alpha\.{number})?", version) if isinstance(version, str) else None
    if match is None:
        raise ValueError("Only stable x.y.z or alpha x.y.z-alpha.N versions may publish")
    return "alpha" if match.group(1) else "latest"


class RegistryPropagationError(ValueError):
    """Registry metadata is not yet consistent with the tested release."""


def registry_json(url, deadline=None):
    timeout = REGISTRY_TIMEOUT_SECONDS
    if deadline is not None:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise RegistryPropagationError("Registry propagation window expired")
        timeout = min(timeout, remaining)
    with urlopen(url, timeout=timeout) as response:
        return json.load(response)


def verify_registry(package_url, manifest, integrity, shasum, deadline=None):
    published = registry_json(package_url + "/" + quote(manifest["version"], safe=""), deadline)
    if not isinstance(published, dict):
        raise RegistryPropagationError("Published metadata is not an object")
    if (published.get("name"), published.get("version"), published.get("license")) != (
        manifest["name"], manifest["version"], "AGPL-3.0-only"
    ):
        raise RegistryPropagationError("Published identity/version/license differs")
    dist = published.get("dist")
    if not isinstance(dist, dict) or dist.get("integrity") != integrity:
        raise RegistryPropagationError("Registry integrity differs from the tested tarball")
    if dist.get("shasum") != shasum:
        raise RegistryPropagationError("Registry legacy checksum differs")
    root_metadata = registry_json(package_url, deadline)
    tags = root_metadata.get("dist-tags") if isinstance(root_metadata, dict) else None
    if not isinstance(tags, dict) or tags.get(manifest["tag"]) != manifest["version"]:
        raise RegistryPropagationError("Registry dist-tag does not select the tested release")


def wait_for_registry(package_url, manifest, integrity, shasum):
    deadline = time.monotonic() + REGISTRY_PROPAGATION_SECONDS
    attempts = 0
    reason = "Registry propagation window expired"
    for attempt in range(REGISTRY_ATTEMPTS):
        if time.monotonic() >= deadline:
            break
        attempts = attempt + 1
        try:
            verify_registry(package_url, manifest, integrity, shasum, deadline)
            return
        except HTTPError as error:
            if error.code not in RETRYABLE_HTTP_STATUS:
                raise
            reason = f"Registry HTTP {error.code}"
        except (URLError, socket.timeout, json.JSONDecodeError, RegistryPropagationError) as error:
            reason = str(error)
        remaining = deadline - time.monotonic()
        if remaining <= 0 or attempt == REGISTRY_ATTEMPTS - 1:
            break
        print(f"Waiting for registry propagation ({attempt + 1}/{REGISTRY_ATTEMPTS}): {reason}", file=sys.stderr)
        time.sleep(min(REGISTRY_RETRY_SECONDS, remaining))
    raise ValueError(f"Registry verification failed after {attempts} attempts within "
                     f"{REGISTRY_PROPAGATION_SECONDS} seconds: {reason}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    if manifest["name"] != "@7mlabs/astrology":
        raise ValueError("Unexpected package identity")
    if manifest["tag"] != release_tag(manifest["version"]):
        raise ValueError("Release dist-tag differs from its stable/alpha version")
    artifact = args.manifest.parent / manifest["filename"]
    data = artifact.read_bytes()
    if hashlib.sha256(data).hexdigest() != manifest["sha256"]:
        raise ValueError("Local tested tarball checksum differs")
    expected = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()
    package_url = "https://registry.npmjs.org/" + quote(manifest["name"], safe="")
    wait_for_registry(package_url, manifest, expected, hashlib.sha1(data).hexdigest())
    print(json.dumps({"result": "passed", "name": manifest["name"],
                      "version": manifest["version"], "sha256": manifest["sha256"],
                      "registryIntegrity": expected, "registry": "https://registry.npmjs.org/"}, indent=2))


if __name__ == "__main__":
    main()
