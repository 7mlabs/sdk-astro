#!/usr/bin/env python3
"""Verify a public npm alpha and its installed-consumer evidence before publish."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "7mlabs/sdk-astro"
TARGETS = {("darwin", "arm64"), ("linux", "x64")}
MAX_BYTES = 256 * 1024 * 1024
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
GIT_SHA = re.compile(r"[0-9a-f]{40}\Z")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def checksum(value, label):
    require(isinstance(value, str) and SHA256.fullmatch(value), f"Invalid {label} checksum")


def integer(value, minimum=0):
    return type(value) is int and value >= minimum


def archive_entries(path):
    require(path.stat().st_size <= MAX_BYTES, "Release archive is too large")
    result = {}
    total = 0
    with tarfile.open(path, "r:gz") as archive:
        for entry in archive:
            parts = entry.name.split("/")
            require(parts[0] == "package" and len(parts) > 1
                    and all(part and part not in (".", "..") for part in parts)
                    and "\\" not in entry.name,
                    f"Unsafe archive path: {entry.name}")
            require(entry.isfile(), f"Only regular files are allowed: {entry.name}")
            require(entry.name not in result, f"Duplicate archive member: {entry.name}")
            total += entry.size
            require(total <= MAX_BYTES and len(result) < 1000, "Archive exceeds release limits")
            value = archive.extractfile(entry).read()
            require(len(value) == entry.size, f"Truncated archive member: {entry.name}")
            result[entry.name] = value
    return result


def binary_header(value, platform, arch):
    require(len(value) >= 64, "Truncated native binary")
    if (platform, arch) == ("darwin", "arm64"):
        require(value[:4] == b"\xcf\xfa\xed\xfe"
                and int.from_bytes(value[4:8], "little") == 0x0100000C
                and int.from_bytes(value[12:16], "little") == 6,
                "Expected an ARM64 Mach-O dynamic library")
        return "Mach-O64", "arm64"
    require(value[:6] == b"\x7fELF\x02\x01"
            and int.from_bytes(value[16:18], "little") == 3
            and int.from_bytes(value[18:20], "little") == 62,
            "Expected an x86-64 ELF shared object")
    return "ELF64", "x86_64"


def verify_reports(directory, manifest, manifest_sha):
    require(directory.is_dir(), "Reports directory is missing")
    reports = sorted(directory.rglob("*.json"))
    require(reports, "No installed-consumer reports found")
    seen = set()
    conformance_sha = sha((ROOT / "tests/conformance/cases.json").read_bytes())
    query_pattern = re.compile(
        r"Query contracts: (\d+) request cases; (\d+) responses "
        r"\((\d+) success, (\d+) error\); (\d+) malformed structures "
        r"and (\d+) broken semantic joins rejected\Z")
    for path in reports:
        report = read_json(path)
        prefix = f"{path.name}: "
        require(report.get("schemaVersion") == "1.0" and report.get("result") == "passed",
                prefix + "Installed-consumer checks did not pass")
        for key, expected in {
            "name": manifest["name"], "engineVersion": manifest["version"],
            "packageSha256": manifest["sha256"], "releaseManifestSha256": manifest_sha,
            "sourceCommit": manifest["source"]["commitSha"], "conformanceSha256": conformance_sha,
            "typescript": "passed", "installation": "fresh-consumer-offline-ignore-scripts",
            "runtimeDownloads": False,
        }.items():
            require(report.get(key) == expected, prefix + f"Evidence differs: {key}")
        target = (report.get("platform"), report.get("arch"))
        require(target in TARGETS, prefix + "Unexpected target")
        node = re.fullmatch(r"v(\d+)\.(\d+)\.(\d+)", report.get("nodeVersion", ""))
        require(node is not None and int(node.group(1)) >= 18, prefix + "Invalid Node version")
        identity = (*target, int(node.group(1)))
        require(identity not in seen, prefix + "Duplicate target/Node-major report")
        seen.add(identity)
        native = next(item for item in manifest["targets"]
                      if (item["platform"], item["arch"]) == target)
        require(report.get("nativeSha256") == native["sha256"], prefix + "Native evidence differs")
        checksum(report.get("referenceCliSha256"), prefix + "Rust reference")
        require(integer(report.get("conformanceCases"), 339), prefix + "Full conformance gate missing")
        require(integer(report.get("queryHelpers"), 8), prefix + "Query helpers gate missing")
        query = query_pattern.fullmatch(report.get("querySchemaAndSemanticChecks", ""))
        require(query is not None, prefix + "Query schema/semantic evidence missing")
        request_count, response_count, successes, errors, malformed, joins = map(int, query.groups())
        require(request_count > 0 and response_count >= 8 and successes == response_count
                and errors == 0 and malformed > 0 and joins > 0,
                prefix + "Query schema/semantic gate incomplete")
        compression = report.get("compressionSchemaAndRoundtripChecks")
        require(isinstance(compression, dict) and compression.get("result") == "passed"
                and integer(compression.get("contractCases"), 27)
                and integer(compression.get("realFixtureRoundtrips"), 13),
                prefix + "Compression contract/roundtrip gate incomplete")
        for key in ("encodedCollisionRoundtrip", "sourceErrorPreserved",
                    "incompleteMetadataRejected", "deepEncodingRejected"):
            require(compression.get(key) is True, prefix + f"Compression evidence missing: {key}")
        require(report.get("archivedNativeTargets") == ["darwin-arm64", "linux-x64"],
                prefix + "Archived native targets differ")
        require(report.get("unsupportedTargetsRejected") == ["win32-x64", "darwin-x64", "linux-arm64"],
                prefix + "Unsupported target checks missing")
    required = {(*target, node) for target in TARGETS for node in (18, 24)}
    require(required <= seen, "Require Node 18 and 24 consumer reports on both native targets")
    return [f"{platform}-{arch}/Node{node}" for platform, arch, node in sorted(seen)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--reports", type=Path)
    parser.add_argument("--tag")
    parser.add_argument("--run-id", default=os.environ.get("GITHUB_RUN_ID"))
    args = parser.parse_args()
    require(GIT_SHA.fullmatch(args.commit) is not None, "Pass a full immutable lowercase Git SHA")
    current = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    require(current == args.commit, "Expected source commit must equal current Git HEAD")
    manifest_bytes = args.manifest.read_bytes()
    manifest = json.loads(manifest_bytes)
    source = read_json(ROOT / "bindings/node/package.json")
    version = source["version"]
    require(re.fullmatch(r"\d+\.\d+\.\d+-alpha\.\d+", version), "Only explicit alpha versions may publish")
    core = tomllib.loads((ROOT / "neutral-engine/Cargo.toml").read_text())["workspace"]["package"]["version"]
    require(source.get("private") is True and core == version, "Source version/private guard differs")
    for key, expected in {"schemaVersion": "1.0", "name": "@7mlabs/astrology", "engineVersion": version,
                          "version": version, "abiVersion": 1, "tag": "alpha", "access": "public"}.items():
        require(manifest.get(key) == expected, f"Release metadata differs: {key}")
    if args.tag is not None:
        require(args.tag == "v" + version, "Git tag must exactly match the public alpha version")
    provenance = manifest.get("source")
    require(isinstance(provenance, dict) and provenance.get("repository") == REPOSITORY
            and provenance.get("commitSha") == args.commit, "Release source provenance differs")
    candidates = provenance.get("candidates")
    require(isinstance(candidates, list) and len(candidates) == 2, "Require two source candidates")
    candidate_targets = set()
    runs = set()
    filename = f"7mlabs-astrology-{version}.tgz"
    for candidate in candidates:
        target = (candidate.get("platform"), candidate.get("arch"))
        require(target in TARGETS and target not in candidate_targets, "Invalid/duplicate source candidate target")
        candidate_targets.add(target)
        require(candidate.get("filename") == filename
                and candidate.get("workflow") == ".github/workflows/neutral-engine.yml",
                "Candidate archive/workflow identity differs")
        run_id = candidate.get("runId")
        require(isinstance(run_id, str) and re.fullmatch(r"[1-9]\d*", run_id), "Invalid candidate run ID")
        runs.add(run_id)
        for key in ("sha256", "manifestSha256", "provenanceSha256"):
            checksum(candidate.get(key), "candidate " + key)
        evidence = candidate.get("evidence")
        expected_evidence = {"test-results.json", "compression-test-results.json", "compression-binding-tests.json"}
        require(isinstance(evidence, dict) and set(evidence) == expected_evidence, "Candidate evidence set differs")
        for name, value in evidence.items():
            checksum(value, name)
    require(candidate_targets == TARGETS and len(runs) == 1, "Candidates must share one workflow run")
    if args.run_id is not None:
        require(re.fullmatch(r"[1-9]\d*", args.run_id) and runs == {args.run_id}, "Candidate run ID differs from expected run")
    require(manifest.get("filename") == filename and integer(manifest.get("bytes"), 1), "Release archive name/size differs")
    checksum(manifest.get("sha256"), "release archive")
    artifact = args.manifest.parent / filename
    require(artifact.stat().st_size <= MAX_BYTES, "Release archive is too large")
    archive_bytes = artifact.read_bytes()
    require(len(archive_bytes) == manifest["bytes"] and sha(archive_bytes) == manifest["sha256"],
            "Release archive size/checksum differs")
    entries = archive_entries(artifact)
    metadata = json.loads(entries["package/package.json"])
    expected = dict(source)
    expected.pop("private")
    expected.update(publishConfig={"access": "public", "tag": "alpha"}, os=["darwin", "linux"], cpu=["arm64", "x64"])
    require(metadata == expected and "private" not in metadata, "Public npm metadata differs from guarded source")
    require(metadata.get("license") == "AGPL-3.0-only", "Release license differs")
    for key in ("scripts", "dependencies", "optionalDependencies", "peerDependencies", "bundledDependencies", "bundleDependencies"):
        require(not metadata.get(key), f"Unexpected runtime installation mechanism: {key}")
    targets = manifest.get("targets")
    require(isinstance(targets, list) and len(targets) == 2, "Require exactly two native binaries")
    seen_targets = set()
    expected_paths = set()
    for target in targets:
        pair = (target.get("platform"), target.get("arch"))
        require(pair in TARGETS and pair not in seen_targets, "Invalid/duplicate native target")
        seen_targets.add(pair)
        native_path = f"native/{pair[0]}-{pair[1]}/astro.node"
        require(target.get("path") == native_path and integer(target.get("bytes"), 64), "Native path/size differs")
        checksum(target.get("sha256"), native_path)
        member = "package/" + native_path
        expected_paths.add(member)
        value = entries[member]
        require(len(value) == target["bytes"] and sha(value) == target["sha256"], "Native size/checksum differs")
        require(binary_header(value, *pair) == (target.get("format"), target.get("machine")), "Native format metadata differs")
    require(seen_targets == TARGETS, "Native target set differs")
    portable = manifest.get("portableFiles")
    require(isinstance(portable, dict), "Portable file manifest missing")
    for name, value in portable.items():
        parts = name.split("/")
        require(all(part and part not in (".", "..") for part in parts)
                and "\\" not in name and parts[0] != "native", "Unsafe portable file path")
        checksum(value, name)
        member = "package/" + name
        expected_paths.add(member)
        require(sha(entries[member]) == value, f"Portable file checksum differs: {name}")
        if name != "package.json":
            local = ROOT / "bindings/node" / PurePosixPath(name)
            require(local.is_file() and entries[member] == local.read_bytes(), f"Portable source differs: {name}")
    require(set(entries) == expected_paths, "Archive contains unmanifested or missing files")
    require({"package.json", "index.cjs", "index.d.ts", "README.md", "LICENSE", "NOTICE"} <= set(portable),
            "Required npm wrapper/types/license/docs missing")
    require(any(name.startswith("third-party/") for name in portable), "Third-party license notices missing")
    require(entries["package/LICENSE"] == (ROOT / "LICENSE").read_bytes(), "Full AGPL license text differs")
    reports = verify_reports(args.reports, manifest, sha(manifest_bytes)) if args.reports else []
    print(json.dumps({"result": "passed", "name": manifest["name"], "version": version,
                      "packageSha256": manifest["sha256"], "sourceCommit": args.commit,
                      "candidateRunId": next(iter(runs)), "nativeTargets": sorted(f"{p}-{a}" for p, a in TARGETS),
                      "installedConsumerReports": reports, "tag": "alpha"}, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, tarfile.TarError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"npm release verification failed: {error}")
