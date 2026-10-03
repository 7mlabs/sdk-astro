#!/usr/bin/env python3
"""Verify that the published registry version is the exact tested tarball."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import time
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import urlopen


def registry_json(url):
    for attempt in range(3):
        try:
            with urlopen(url, timeout=15) as response:
                return json.load(response)
        except HTTPError as error:
            if error.code not in (404, 429, 500, 502, 503, 504) or attempt == 2:
                raise
        except URLError:
            if attempt == 2:
                raise
        time.sleep(2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    if manifest["name"] != "@7mlabs/astrology":
        raise ValueError("Unexpected package identity")
    artifact = args.manifest.parent / manifest["filename"]
    data = artifact.read_bytes()
    if hashlib.sha256(data).hexdigest() != manifest["sha256"]:
        raise ValueError("Local tested tarball checksum differs")
    expected = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()
    package_url = "https://registry.npmjs.org/" + quote(manifest["name"], safe="")
    published = registry_json(package_url + "/" + quote(manifest["version"], safe=""))
    if (published.get("name"), published.get("version"), published.get("license")) != (
        manifest["name"], manifest["version"], "AGPL-3.0-only"
    ):
        raise ValueError("Published identity/version/license differs")
    if published.get("dist", {}).get("integrity") != expected:
        raise ValueError("Registry integrity differs from the tested tarball")
    if published["dist"].get("shasum") != hashlib.sha1(data).hexdigest():
        raise ValueError("Registry legacy checksum differs")
    if registry_json(package_url).get("dist-tags", {}).get(manifest["tag"]) != manifest["version"]:
        raise ValueError("Registry dist-tag does not select the tested release")
    print(json.dumps({"result": "passed", "name": manifest["name"],
                      "version": manifest["version"], "sha256": manifest["sha256"],
                      "registryIntegrity": expected, "registry": "https://registry.npmjs.org/"}, indent=2))


if __name__ == "__main__":
    main()
