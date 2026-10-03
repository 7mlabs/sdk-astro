"""Validate candidate source/metadata; optionally verify built package evidence.

Uses Python 3.11+ standard library only. This is not a public-release approval:
license, registry ownership and the final platform packaging remain separate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


def load_json(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_source():
    cargo = tomllib.loads((ROOT / "neutral-engine/Cargo.toml").read_text())
    version = cargo["workspace"]["package"]["version"]
    node = load_json("bindings/node/package.json")
    python = tomllib.loads((ROOT / "bindings/python/pyproject.toml").read_text())["project"]
    dotnet = ET.parse(ROOT / "bindings/dotnet/SevenMLabs.Astrology.csproj").getroot()
    python_version = re.sub(r"-alpha\.(\d+)$", r"a\1", version)
    require(node["name"] == "@7mlabs/astrology", "Unexpected npm package name")
    require(python["name"] == "sevenmlabs-astrology", "Unexpected Python package name")
    require(dotnet.findtext(".//PackageId") == "SevenMLabs.Astrology", "Unexpected NuGet package ID")
    require(node["version"] == dotnet.findtext(".//Version") == version,
            "Core/npm/NuGet versions differ")
    require(python["version"] == python_version, "Python/core versions differ")
    repository = "https://github.com/7mlabs/sdk-astro"
    node_repository = node.get("repository", {}).get("url", "").removeprefix("git+").removesuffix(".git")
    require(node_repository == repository, "npm repository metadata differs from destination")
    require(python.get("urls", {}).get("Repository") == repository,
            "Python repository metadata differs from destination")
    require(dotnet.findtext(".//RepositoryUrl") == repository,
            "NuGet repository metadata differs from destination")
    require(node.get("private") is True, "Candidate npm package must retain private:true")
    for manifest in sorted((ROOT / "neutral-engine/crates").glob("*/Cargo.toml")):
        package = tomllib.loads(manifest.read_text())["package"]
        require(package.get("publish") is False, f"Registry guard missing: {manifest.relative_to(ROOT)}")
    workflow = (ROOT / ".github/workflows/neutral-engine.yml").read_text()
    require(not re.search(r"(?:npm|cargo)\s+publish|twine\s+upload|dotnet\s+nuget\s+push|gh\s+release|(?:contents|id-token):\s*write", workflow),
            "Candidate workflow must not grant publication permissions or publish packages")
    for name in ("Astro.Engineer", "Astro.Physics", "astro-sandbox"):
        require(not (ROOT / name).exists(), f"Legacy application should not be in SDK snapshot: {name}")
    for filename in (ROOT / "schemas").glob("*.json"):
        json.loads(filename.read_text(encoding="utf-8"))

    markdown = [ROOT / "README.md", *sorted((ROOT / "docs").glob("*.md")),
                *sorted((ROOT / "bindings").glob("*/README.md")),
                *sorted((ROOT / "examples").glob("*/README.md"))]
    broken = []
    link_count = 0
    for filename in markdown:
        require(filename.is_file(), f"Missing documentation: {filename.relative_to(ROOT)}")
        text = re.sub(r"(?ms)^\s*(`{3,}|~{3,}).*?^\s*\1\s*$", "", filename.read_text(encoding="utf-8"))
        for match in re.finditer(r"!?\[[^\]]*\]\(([^)]+)\)", text):
            target = match.group(1).strip()
            target = target[1:target.index(">")] if target.startswith("<") else target.split()[0]
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            path = (filename.parent / unquote(parsed.path)).resolve()
            link_count += 1
            if not path.is_relative_to(ROOT) or not path.exists():
                broken.append(f"{filename.relative_to(ROOT)} -> {target}")
    require(not broken, "Broken/outside-repository documentation links:\n" + "\n".join(broken))
    blockers = ["Registry ownership/trusted publishers not verified",
                "Final multi-platform npm/NuGet assembly and portable Linux wheels not verified",
                "Windows package builder not implemented"]
    if not any((ROOT / name).is_file() for name in ("LICENSE", "LICENSE.md", "LICENSE.txt")):
        blockers.insert(0, "Root license/provenance decision pending")
    return {"engineVersion": version, "pythonVersion": python_version,
            "documentationFiles": len(markdown), "localLinks": link_count,
            "candidateWorkflowPublishingEnabled": False, "releaseBlockers": blockers}


def check_artifacts(summary, expected_platform, expected_arch):
    manifest = load_json("artifacts/packages/manifest.json")
    version = summary["engineVersion"]
    require(manifest["engineVersion"] == version, "Built/core versions differ")
    if expected_platform:
        require(manifest["platform"] == expected_platform, "Built platform differs from matrix")
    if expected_arch:
        require(manifest["arch"] == expected_arch, "Built architecture differs from matrix")
    rid = ("osx" if manifest["platform"] == "darwin" else "linux") + "-" + manifest["arch"]
    require(manifest["rid"] == rid, "Built runtime identifier differs from platform/architecture")
    files = manifest["files"]
    require(len(files) == 3, "Expected exactly one npm package, Python wheel and NuGet package")
    require(f"7mlabs-astrology-{version}.tgz" in files, "Missing npm artifact")
    require(f"SevenMLabs.Astrology.{version}.nupkg" in files, "Missing NuGet artifact")
    wheels = [name for name in files if name.startswith(f"sevenmlabs_astrology-{summary['pythonVersion']}-") and name.endswith(".whl")]
    require(len(wheels) == 1, "Expected one wheel for this candidate platform")
    for name, digest in files.items():
        require(Path(name).name == name, "Invalid package filename")
        file = ROOT / "artifacts/packages" / name
        require(file.is_file(), f"Missing artifact: {name}")
        require(hashlib.sha256(file.read_bytes()).hexdigest() == digest, f"Artifact checksum differs: {name}")
    fingerprint = hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()[:12]
    for relative in ("artifacts/test-results.json", "artifacts/compression-test-results.json",
                     "artifacts/compression-binding-tests.json"):
        evidence = load_json(relative)
        require(evidence.get("result") == "passed", f"Candidate checks did not pass: {relative}")
        require(evidence.get("engineVersion") == version, f"Stale evidence version: {relative}")
        require(evidence.get("consumerFingerprint") == fingerprint, f"Stale package evidence: {relative}")
        if relative.endswith("compression-test-results.json"):
            require(evidence.get("packageManifestSha256") == hashlib.sha256((ROOT / "artifacts/packages/manifest.json").read_bytes()).hexdigest(),
                    "Compression evidence uses another package manifest")
        if relative.endswith("compression-binding-tests.json"):
            require(evidence.get("typescript") == "passed", "Installed TypeScript contracts did not pass")
    summary["candidateArtifacts"] = {"platform": manifest["platform"], "arch": manifest["arch"],
                                     "files": sorted(files), "consumerFingerprint": fingerprint}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", action="store_true")
    parser.add_argument("--platform", choices=("darwin", "linux"))
    parser.add_argument("--arch", choices=("arm64", "x64"))
    args = parser.parse_args()
    try:
        summary = check_source()
        if args.artifacts:
            check_artifacts(summary, args.platform, args.arch)
        summary["result"] = "passed"
        print(json.dumps(summary, indent=2))
    except (ValueError, KeyError, OSError, ET.ParseError) as error:
        print(f"Repository check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
