#!/usr/bin/env python3
"""Assemble one public npm alpha from two validated, same-commit candidates.

Candidate directories are downloaded GitHub artifacts containing packages/,
test-results.json, compression-test-results.json, compression-binding-tests.json
and candidate-provenance.json. Nothing is downloaded, built or published here.
"""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "7mlabs/sdk-astro"
TARGETS = (("darwin", "arm64"), ("linux", "x64"))
MAX_ARCHIVE_BYTES = 256 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def find_file(directory, name, manifest_directory=None):
    candidates = {directory / name, directory / "artifacts" / name}
    if manifest_directory is not None:
        candidates.update({manifest_directory / name, manifest_directory.parent / name})
    matches = {path.resolve() for path in candidates if path.is_file()}
    require(len(matches) == 1, f"Expected one {name} in {directory}; found {len(matches)}")
    return matches.pop()


def read_tarball(path):
    require(path.stat().st_size <= MAX_ARCHIVE_BYTES, f"Oversized archive: {path}")
    entries = {}
    total = 0
    with tarfile.open(path, "r:gz") as archive:
        for member in archive:
            parts = PurePosixPath(member.name).parts
            require(parts and parts[0] == "package" and not member.name.startswith("/")
                    and all(part not in (".", "..") for part in parts)
                    and "\\" not in member.name
                    and str(PurePosixPath(member.name)) == member.name.rstrip("/"),
                    f"Unsafe archive member: {member.name}")
            require(member.isfile() or member.isdir(), f"Links/devices are forbidden: {member.name}")
            if member.isdir():
                continue
            require(member.name not in entries, f"Duplicate archive member: {member.name}")
            total += member.size
            require(total <= MAX_ARCHIVE_BYTES and len(entries) < 1000, "Archive exceeds release limits")
            value = archive.extractfile(member).read()
            require(len(value) == member.size, f"Truncated archive member: {member.name}")
            entries[member.name] = value
    return entries


def binary_header(data, platform, arch):
    require(len(data) >= 64, "Native library is too short")
    if (platform, arch) == ("darwin", "arm64"):
        require(data[:4] == b"\xcf\xfa\xed\xfe", "Expected little-endian 64-bit Mach-O")
        require(int.from_bytes(data[4:8], "little") == 0x0100000C, "Expected ARM64 Mach-O")
        require(int.from_bytes(data[12:16], "little") == 6, "Expected Mach-O dynamic library")
        return {"format": "Mach-O64", "machine": "arm64"}
    require(data[:6] == b"\x7fELF\x02\x01", "Expected little-endian 64-bit ELF")
    require(int.from_bytes(data[16:18], "little") == 3, "Expected ELF shared object")
    require(int.from_bytes(data[18:20], "little") == 62, "Expected x86-64 ELF")
    return {"format": "ELF64", "machine": "x86_64"}


def load_candidate(directory, platform, arch, commit, version):
    directory = directory.resolve()
    manifest_candidates = {directory / "manifest.json", directory / "packages/manifest.json",
                           directory / "artifacts/packages/manifest.json"}
    matches = {path.resolve() for path in manifest_candidates if path.is_file()}
    require(len(matches) == 1, f"Expected one package manifest in {directory}")
    manifest_path = matches.pop()
    manifest = read_json(manifest_path)
    require((manifest.get("platform"), manifest.get("arch")) == (platform, arch), "Candidate target differs")
    require(manifest.get("rid") == ("osx" if platform == "darwin" else "linux") + "-" + arch,
            "Candidate RID differs")
    require(manifest.get("engineVersion") == version, "Candidate/core versions differ")
    files = manifest.get("files")
    require(isinstance(files, dict) and len(files) == 3, "Expected three candidate package artifacts")
    for filename, checksum in files.items():
        require(Path(filename).name == filename and re.fullmatch(r"[0-9a-f]{64}", checksum),
                "Invalid candidate artifact name/checksum")
        artifact = manifest_path.parent / filename
        require(artifact.is_file() and digest(artifact.read_bytes()) == checksum,
                f"Candidate checksum mismatch: {filename}")
    npm_name = f"7mlabs-astrology-{version}.tgz"
    require(npm_name in files, "Expected npm candidate is missing")
    fingerprint = digest(json.dumps(files, sort_keys=True).encode())[:12]
    evidence = {}
    for name in ("test-results.json", "compression-test-results.json", "compression-binding-tests.json"):
        path = find_file(directory, name, manifest_path.parent)
        report = read_json(path)
        require(report.get("result") == "passed" and report.get("engineVersion") == version,
                f"Failed/stale candidate evidence: {name}")
        require(report.get("consumerFingerprint") == fingerprint, f"Candidate evidence fingerprint differs: {name}")
        if name == "test-results.json":
            require(report.get("parityCases", 0) >= 339 and report.get("platform") == platform,
                    "Full candidate parity/platform evidence missing")
        if name == "compression-test-results.json":
            require(report.get("packageManifestSha256") == digest(manifest_path.read_bytes()),
                    "Compression evidence references another manifest")
        if name == "compression-binding-tests.json":
            require(report.get("typescript") == "passed", "Candidate TypeScript gate missing")
        evidence[name] = digest(path.read_bytes())
    provenance_path = find_file(directory, "candidate-provenance.json", manifest_path.parent)
    provenance = read_json(provenance_path)
    require(provenance.get("repository") == REPOSITORY and provenance.get("commitSha") == commit,
            "Candidate repository/commit provenance differs")
    require((provenance.get("platform"), provenance.get("arch"), provenance.get("engineVersion"))
            == (platform, arch, version), "Candidate provenance target/version differs")
    require(isinstance(provenance.get("workflow"), str) and provenance["workflow"].endswith((".yml", ".yaml")),
            "Candidate workflow provenance missing")
    require(re.fullmatch(r"[1-9][0-9]*", str(provenance.get("runId", ""))) is not None,
            "Candidate run provenance missing")
    archive = manifest_path.parent / npm_name
    entries = read_tarball(archive)
    metadata = json.loads(entries["package/package.json"])
    require(metadata.get("name") == "@7mlabs/astrology" and metadata.get("version") == version,
            "Candidate npm identity/version differs")
    require(metadata.get("private") is True and metadata.get("license") == "AGPL-3.0-only",
            "Expected guarded AGPL npm candidate")
    require(not metadata.get("scripts") and not any(metadata.get(field) for field in
            ("dependencies", "optionalDependencies", "peerDependencies", "bundledDependencies", "bundleDependencies")),
            "Candidate must not introduce installation scripts or dependency downloads")
    native_path = f"package/native/{platform}-{arch}/astro.node"
    native_members = [name for name in entries if name.startswith("package/native/")]
    require(native_members == [native_path], "Candidate must contain exactly its declared native addon")
    header = binary_header(entries[native_path], platform, arch)
    required = ("package/index.cjs", "package/index.d.ts", "package/README.md", "package/LICENSE", "package/NOTICE")
    require(all(name in entries for name in required), "Candidate wrapper/types/license/notice missing")
    require(entries["package/LICENSE"] == (ROOT / "LICENSE").read_bytes(), "Candidate AGPL text differs")
    require(any(name.startswith("package/third-party/") for name in entries), "Third-party notices missing")
    portable = {name: value for name, value in entries.items() if name != native_path}
    info = {"platform": platform, "arch": arch, "filename": npm_name, "sha256": files[npm_name],
            "manifestSha256": digest(manifest_path.read_bytes()), "provenanceSha256": digest(provenance_path.read_bytes()),
            "workflow": provenance["workflow"], "runId": str(provenance["runId"]), "evidence": evidence}
    for name in ("artifactId", "archiveSha256", "source", "runAttempt"):
        if name in provenance:
            info[name] = provenance[name]
    target = {"platform": platform, "arch": arch, "path": native_path.removeprefix("package/"),
              "sha256": digest(entries[native_path]), "bytes": len(entries[native_path]), **header}
    return portable, entries[native_path], info, target


def write_tarball(entries, filename):
    with filename.open("wb") as destination:
        with gzip.GzipFile(filename="", mode="wb", fileobj=destination, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.USTAR_FORMAT) as archive:
                for name, value in sorted(entries.items()):
                    member = tarfile.TarInfo(name)
                    member.size = len(value)
                    member.mode = 0o644
                    member.mtime = member.uid = member.gid = 0
                    member.uname = member.gname = ""
                    archive.addfile(member, io.BytesIO(value))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--darwin", required=True, type=Path)
    parser.add_argument("--linux", required=True, type=Path)
    parser.add_argument("--commit", required=True, help="Full immutable source commit SHA")
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/npm-release")
    args = parser.parse_args()
    require(re.fullmatch(r"[0-9a-f]{40}", args.commit) is not None, "A full lowercase Git SHA is required")
    source = read_json(ROOT / "bindings/node/package.json")
    version = source["version"]
    require(re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+", version) is not None,
            "This assembler only publishes the explicit alpha channel")
    import tomllib
    core_version = tomllib.loads((ROOT / "neutral-engine/Cargo.toml").read_text())["workspace"]["package"]["version"]
    require(version == core_version and source.get("private") is True, "Source core/version/guard differs")
    candidates = [load_candidate(directory, platform, arch, args.commit, version)
                  for directory, (platform, arch) in zip((args.darwin, args.linux), TARGETS)]
    require(candidates[0][0] == candidates[1][0], "Candidate wrapper/types/metadata/notices differ between platforms")
    require((candidates[0][2]["workflow"], candidates[0][2]["runId"])
            == (candidates[1][2]["workflow"], candidates[1][2]["runId"]),
            "Candidates must come from the same workflow run")
    entries = dict(candidates[0][0])
    metadata = json.loads(entries["package/package.json"])
    require(metadata == source, "Candidate metadata differs from the current guarded source")
    for name in entries:
        relative = PurePosixPath(name).relative_to("package")
        local = ROOT / "bindings/node" / relative
        require(local.is_file() and local.read_bytes() == entries[name], f"Candidate source differs: {relative}")
    metadata.pop("private")
    metadata["publishConfig"] = {"access": "public", "tag": "alpha"}
    metadata["os"] = ["darwin", "linux"]
    metadata["cpu"] = ["arm64", "x64"]
    entries["package/package.json"] = (json.dumps(metadata, indent=2) + "\n").encode()
    for _, native, _, target in candidates:
        entries["package/" + target["path"]] = native
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    filename = f"7mlabs-astrology-{version}.tgz"
    artifact = output / filename
    write_tarball(entries, artifact)
    manifest = {"schemaVersion": "1.0", "name": metadata["name"], "engineVersion": version,
                "version": version, "abiVersion": 1, "tag": "alpha", "access": "public",
                "filename": filename, "sha256": digest(artifact.read_bytes()), "bytes": artifact.stat().st_size,
                "source": {"repository": REPOSITORY, "commitSha": args.commit,
                           "candidates": [candidate[2] for candidate in candidates]},
                "targets": [candidate[3] for candidate in candidates],
                "portableFiles": {name.removeprefix("package/"): digest(value)
                                  for name, value in sorted(entries.items()) if "/native/" not in name}}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (output / "SHA256SUMS").write_text(f"{manifest['sha256']}  {filename}\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, tarfile.TarError) as error:
        raise SystemExit(f"npm release assembly failed: {error}")
