"""Build and stage local packages. No publishing or runtime downloads.

Run with the Python build venv (setuptools+wheel installed). NODE_BIN/NPM_CLI
can select local tools. --offline disables registry access for Rust.
"""
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--offline", action="store_true")
args = parser.parse_args()


def run(command, cwd=ROOT):
    print("+", " ".join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), cwd=cwd, check=True)


system = {"Darwin": "darwin", "Linux": "linux", "Windows": "win32"}[platform.system()]
arch = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x64", "AMD64": "x64"}[platform.machine()]
if system == "win32":
    raise SystemExit("Windows native Node-API linking is a planned release target, not implemented by this sample builder")
node = os.environ.get("NODE_BIN", "node")
npm_cli = os.environ.get("NPM_CLI")
npm = [node, npm_cli] if npm_cli else ["npm"]
packages = ROOT / "artifacts/packages"
packages.mkdir(parents=True, exist_ok=True)
workspace = ROOT / "neutral-engine/Cargo.toml"
run(["cargo", "build", "--manifest-path", workspace, "--release", "--locked"] + (["--offline"] if args.offline else []))
target = ROOT / "neutral-engine/target/release"
library = "libastro_engine.dylib" if system == "darwin" else "libastro_engine.so"
addon = "libastro_node.dylib" if system == "darwin" else "libastro_node.so"
rid = ("osx" if system == "darwin" else "linux") + "-" + arch

for destination, source in [
    (ROOT / f"bindings/node/native/{system}-{arch}/astro.node", target / addon),
    (ROOT / f"bindings/python/sevenmlabs_astrology/native/{system}-{arch}/{library}", target / library),
    (ROOT / f"bindings/dotnet/native/{rid}/{library}", target / library),
]:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

run(npm + ["pack", "--ignore-scripts", "--cache", ROOT / "artifacts/npm-build-cache", "--pack-destination", packages], ROOT / "bindings/node")
run([sys.executable, "-m", "pip", "wheel", "--no-deps", "--no-build-isolation", "--no-index", "--wheel-dir", packages, ROOT / "bindings/python"])
run(["dotnet", "pack", ROOT / "bindings/dotnet/SevenMLabs.Astrology.csproj", "-c", "Release", "-o", packages,
     "-p:RestoreSources=" + str(packages), "-p:NuGetAudit=false", "--verbosity", "quiet"])
manifest = {"platform": system, "arch": arch, "rid": rid, "engineVersion": "0.10.0-alpha.1", "files": {}}
for item in packages.iterdir():
    if item.name.startswith(("7mlabs-astrology-0.10.0-alpha.1", "sevenmlabs_astrology-0.10.0a1", "SevenMLabs.Astrology.0.10.0-alpha.1")):
        manifest["files"][item.name] = hashlib.sha256(item.read_bytes()).hexdigest()
(packages / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print("Built local packages for", system, arch)
