"""Install packaged artifacts in fresh consumers, then run examples and parity.

All registry sources are disabled during installation. Consumers are under
artifacts/consumers; wrappers must load binaries shipped in their packages.
"""
import json
import hashlib
import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "artifacts/packages"
manifest = json.loads((PACKAGES / "manifest.json").read_text())
fingerprint = hashlib.sha256(json.dumps(manifest["files"], sort_keys=True).encode()).hexdigest()[:12]
CONSUMERS = ROOT / "artifacts/consumers" / fingerprint
CONSUMERS.mkdir(parents=True, exist_ok=True)
node = os.environ.get("NODE_BIN", "node")
npm_cli = os.environ.get("NPM_CLI")
npm = [node, npm_cli] if npm_cli else ["npm"]


def run(command, cwd=ROOT, capture=False):
    if not capture:
        print("+", " ".join(map(str, command)), flush=True)
    return subprocess.run(list(map(str, command)), cwd=cwd, check=True, text=True,
                          stdout=subprocess.PIPE if capture else None).stdout


for filename, expected_digest in manifest["files"].items():
    if hashlib.sha256((PACKAGES / filename).read_bytes()).hexdigest() != expected_digest:
        raise SystemExit("Package checksum differs from manifest: " + filename)
with zipfile.ZipFile(PACKAGES / "SevenMLabs.Astrology.0.10.0-alpha.1.nupkg") as package:
    native_assets = [name for name in package.namelist() if name.startswith("runtimes/")]
    library = "libastro_engine.dylib" if sys.platform == "darwin" else "libastro_engine.so"
    expected_path = f"runtimes/{manifest['rid']}/native/{library}"
    if native_assets != [expected_path]:
        raise SystemExit("Unexpected NuGet native asset layout: " + repr(native_assets))


node_dir = CONSUMERS / "node"
node_dir.mkdir(exist_ok=True)
for source in (ROOT / "examples/node").iterdir():
    if source.is_file(): shutil.copy2(source, node_dir / source.name)
run(npm + ["install", "--offline", "--ignore-scripts", "--no-audit", "--no-fund",
    "--cache", CONSUMERS / "npm-cache", PACKAGES / "7mlabs-astrology-0.10.0-alpha.1.tgz"], node_dir)
run([node, "test.cjs"], node_dir)

python_dir = CONSUMERS / "python"
python_dir.mkdir(exist_ok=True)
venv = python_dir / "venv"
if not venv.exists(): run([sys.executable, "-m", "venv", venv])
python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
wheels = sorted(PACKAGES.glob("sevenmlabs_astrology-0.10.0a1-*.whl"))
if len(wheels) != 1: raise SystemExit("Expected exactly one wheel for the current target")
run([python, "-m", "pip", "install", "--no-index", "--no-deps", "--force-reinstall", wheels[0]])
for name in ("sample.py", "test_sample.py", "domains.py", "custom_profiles.py", "couple.py", "composite.py", "couple_composite.py", "forecast.py", "query.py"):
    shutil.copy2(ROOT / "examples/python" / name, python_dir / name)
run([python, "test_sample.py"], python_dir)

dotnet_dir = CONSUMERS / "dotnet"
dotnet_dir.mkdir(exist_ok=True)
for name in ("Example.csproj", "Program.cs"):
    shutil.copy2(ROOT / "examples/dotnet" / name, dotnet_dir / name)
run(["dotnet", "restore", "Example.csproj", "--source", PACKAGES,
     "--packages", CONSUMERS / "nuget-cache", "--verbosity", "quiet"], dotnet_dir)
run(["dotnet", "build", "Example.csproj", "--no-restore", "--verbosity", "quiet"], dotnet_dir)
dll = dotnet_dir / "bin/Debug/net10.0/Example.dll"
run(["dotnet", dll, "--test"], dotnet_dir)

run(["cargo", "build", "--manifest-path", ROOT / "examples/rust/Cargo.toml", "--offline", "--quiet"])
rust_binary = ROOT / "examples/rust/target/debug/astrology-rust-example"
c_binary = CONSUMERS / "c-example"
if sys.platform == "darwin":
    run(["clang", ROOT / "examples/c/main.c", "-I", ROOT / "neutral-engine/include", "-L", ROOT / "neutral-engine/target/release",
        "-lastro_engine", "-Wl,-rpath," + str(ROOT / "neutral-engine/target/release"), "-o", c_binary])
elif sys.platform.startswith("linux"):
    run(["cc", ROOT / "examples/c/main.c", "-I", ROOT / "neutral-engine/include", "-L", ROOT / "neutral-engine/target/release",
        "-lastro_engine", "-Wl,-rpath," + str(ROOT / "neutral-engine/target/release"), "-o", c_binary])
else: raise SystemExit("C consumer builder currently supports macOS/Linux")

cases = json.loads((ROOT / "tests/conformance/cases.json").read_text())
commands = {"node": [node, node_dir / "sample.cjs"], "python": [python, python_dir / "sample.py"],
            "dotnet": ["dotnet", dll], "rust": [rust_binary], "c": [c_binary]}


def first_difference(actual, expected, path="$"):
    if type(actual) is not type(expected):
        return f"{path}: {type(actual).__name__} != {type(expected).__name__}"
    if isinstance(actual, dict):
        if actual.keys() != expected.keys():
            return f"{path}: keys {sorted(actual)} != {sorted(expected)}"
        for key in actual:
            if actual[key] != expected[key]:
                return first_difference(actual[key], expected[key], f"{path}.{key}")
    elif isinstance(actual, list):
        if len(actual) != len(expected):
            return f"{path}: length {len(actual)} != {len(expected)}"
        for index, (left, right) in enumerate(zip(actual, expected)):
            if left != right:
                return first_difference(left, right, f"{path}[{index}]")
    return f"{path}: {actual!r} != {expected!r}"


for case in cases:
    raw = case["rawRequest"] if "rawRequest" in case else json.dumps(case["request"], separators=(",", ":"))
    outputs = {name: json.loads(run(command + [raw], capture=True)) for name, command in commands.items()}
    reference = outputs["rust"]
    for name, output in outputs.items():
        if output != reference:
            raise AssertionError(f"{case['name']}: {name} differs from Rust; "
                                 + first_difference(output, reference))
    if bool(reference["errors"]) != case.get("error", False): raise AssertionError(f"Unexpected validation result: {case['name']}")
    print("PASS parity:", case["name"])

couple_request = (ROOT / "examples/couple-request.json").read_text()
couple_reference = json.loads(run(commands["rust"] + [couple_request], capture=True))
couple_examples = {"node": [node, node_dir / "couple.cjs"],
                   "python": [python, python_dir / "couple.py"],
                   "dotnet": ["dotnet", dll, "--couple"]}
for language, command in couple_examples.items():
    output = json.loads(run(command, capture=True))
    if output != couple_reference:
        raise AssertionError(f"Dedicated couple example differs: {language}")
    print("PASS couple example:", language)

composite_examples = {
    "composite": ("composite-request.json", "composite.cjs", "composite.py", "--composite"),
    "couple-composite": ("couple-composite-request.json", "couple-composite.cjs", "couple_composite.py", "--coupleComposite"),
}
for example, (request_file, node_file, python_file, dotnet_option) in composite_examples.items():
    request = (ROOT / "examples" / request_file).read_text()
    reference = json.loads(run(commands["rust"] + [request], capture=True))
    for language, command in {
        "node": [node, node_dir / node_file],
        "python": [python, python_dir / python_file],
        "dotnet": ["dotnet", dll, dotnet_option],
    }.items():
        if json.loads(run(command, capture=True)) != reference:
            raise AssertionError(f"Dedicated {example} example differs: {language}")
        print(f"PASS {example} example:", language)

forecast_request = (ROOT / "examples/forecast-day-request.json").read_text()
forecast_reference = json.loads(run(commands["rust"] + [forecast_request], capture=True))
for language, command in {"node": [node, node_dir / "forecast.cjs"],
                          "python": [python, python_dir / "forecast.py"],
                          "dotnet": ["dotnet", dll, "--forecast"]}.items():
    if json.loads(run(command, capture=True)) != forecast_reference:
        raise AssertionError(f"Dedicated forecast example differs: {language}")
    print("PASS forecast example:", language)

query_request = (ROOT / "examples/query-houses-request.json").read_text()
query_reference = json.loads(run(commands["rust"] + [query_request], capture=True))
for language, command in {"node": [node, node_dir / "query.cjs"],
                          "python": [python, python_dir / "query.py"],
                          "dotnet": ["dotnet", dll, "--query"]}.items():
    if json.loads(run(command, capture=True)) != query_reference:
        raise AssertionError(f"Dedicated query example differs: {language}")
    print("PASS query example:", language)

references = json.loads((ROOT / "tests/conformance/natal-references.json").read_text())
tol = references["tolerances"]
max_independent_angle = 0.0


def close_angle(actual, expected, tolerance):
    delta = abs((actual - expected + 180) % 360 - 180)
    if delta > tolerance: raise AssertionError(f"Angle difference {delta} > {tolerance}")
    return delta


for case in references["cases"]:
    output = json.loads(run(commands["rust"] + [json.dumps(case["request"])], capture=True))
    if output["errors"]: raise AssertionError(output["errors"])
    expected = case["swiss"]
    for field in ("julianDayTt", "julianDayUt1"):
        if abs(output["calculation"][field] - expected[field]) > tol["julianDay"]:
            raise AssertionError(f"Time scale differs: {case['name']} {field}")
    for actual, swiss, jpl in zip(output["data"]["placements"], expected["positions"], case["skyfield"]):
        if actual["id"] != swiss["id"]: raise AssertionError("Body order differs")
        close_angle(actual["longitude"], swiss["longitude"], tol["swissAngleDegrees"])
        if abs(actual["latitude"] - swiss["latitude"]) > tol["swissAngleDegrees"]: raise AssertionError("Latitude differs")
        if abs(actual["speed"] - swiss["speed"]) > tol["swissSpeedDegreesPerDay"]: raise AssertionError("Speed differs")
        if abs(actual["distanceAu"] - swiss["distanceAu"]) > tol["swissDistanceAu"]: raise AssertionError("Distance differs")
        max_independent_angle = max(max_independent_angle, close_angle(actual["longitude"], jpl["longitude"], tol["independentAngleDegrees"]))
        if abs(actual["latitude"] - jpl["latitude"]) > tol["independentAngleDegrees"]: raise AssertionError("Independent latitude differs")
    for actual, swiss in zip(output["data"]["houseCusps"], expected["houseCusps"]):
        close_angle(actual, swiss, tol["swissAngleDegrees"])
    for i, name in enumerate(("ascendant", "midheaven")):
        close_angle(output["data"]["angles"][i]["longitude"], expected[name], tol["swissAngleDegrees"])
    print("PASS reference:", case["name"])
summary = {"platform": sys.platform, "engineVersion": "0.10.0-alpha.1", "languages": list(commands),
           "consumerFingerprint": fingerprint,
           "parityCases": len(cases), "nodeRepeatedCalls": 1000, "pythonConcurrentCalls": 1000,
           "dotnetConcurrentCalls": 1000, "natalRepeatedNodeCalls": 1000,
           "natalConcurrentPythonCalls": 1000, "natalConcurrentDotnetCalls": 1000, "result": "passed", "scope": "query-forecast-events-composite-couple-natal-domains-natal-and-geometry"}
summary["domainRepeatedNodeCalls"] = 20
summary["domainConcurrentPythonCalls"] = 20
summary["domainConcurrentDotnetCalls"] = 20
summary["customProfileRepeatedNodeCalls"] = 20
summary["customProfileConcurrentPythonCalls"] = 20
summary["customProfileConcurrentDotnetCalls"] = 20
summary["coupleRepeatedNodeCalls"] = 20
summary["coupleConcurrentPythonCalls"] = 20
summary["coupleConcurrentDotnetCalls"] = 20
summary["coupleParityCases"] = sum(case["name"].startswith("couple ") for case in cases)
summary["coupleDedicatedExamples"] = len(couple_examples)
summary["compositeRepeatedNodeCalls"] = 20
summary["compositeConcurrentPythonCalls"] = 20
summary["compositeConcurrentDotnetCalls"] = 20
summary["compositeParityCases"] = sum(case["name"].startswith("composite ") for case in cases)
summary["compositeDedicatedExamples"] = 6
summary["forecastRepeatedNodeCalls"] = 20
summary["forecastConcurrentPythonCalls"] = 20
summary["forecastConcurrentDotnetCalls"] = 20
summary["temporalParityCases"] = sum(case["name"].startswith(("forecast ", "events ")) for case in cases)
summary["forecastDedicatedExamples"] = 3
summary["queryRepeatedNodeCalls"] = 20
summary["queryConcurrentPythonCalls"] = 20
summary["queryConcurrentDotnetCalls"] = 20
summary["queryParityCases"] = sum(case["name"].startswith("query ") for case in cases)
summary["queryDedicatedExamples"] = 3
summary["natalReferenceCases"] = len(references["cases"])
summary["maxIndependentLongitudeDifferenceDegrees"] = max_independent_angle
(ROOT / "artifacts/test-results.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
