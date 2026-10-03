"""Build and stage local packages. No publishing or runtime downloads.

Run with the Python build venv (setuptools+wheel installed). NODE_BIN/NPM_CLI
can select local tools. --offline disables registry access for Rust.
"""
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--offline", action="store_true")
args = parser.parse_args()
version = tomllib.loads((ROOT / "neutral-engine/Cargo.toml").read_text())["workspace"]["package"]["version"]
python_version = tomllib.loads((ROOT / "bindings/python/pyproject.toml").read_text())["project"]["version"]
node_version = json.loads((ROOT / "bindings/node/package.json").read_text())["version"]
dotnet_version = ET.parse(ROOT / "bindings/dotnet/SevenMLabs.Astrology.csproj").getroot().findtext(".//Version")
if version != node_version or version != dotnet_version or python_version != re.sub(r"-alpha\.(\d+)$", r"a\1", version):
    raise SystemExit("Core and package versions must be synchronized before building")


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
manifest = {"platform": system, "arch": arch, "rid": rid, "engineVersion": version, "files": {}}
wheels = sorted(packages.glob(f"sevenmlabs_astrology-{python_version}-*.whl"))
if len(wheels) != 1:
    raise SystemExit("Expected exactly one wheel for the current version/target")
for item in (packages / f"7mlabs-astrology-{version}.tgz", packages / f"SevenMLabs.Astrology.{version}.nupkg", wheels[0]):
    manifest["files"][item.name] = hashlib.sha256(item.read_bytes()).hexdigest()
(packages / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print("Built local packages for", system, arch)
