"""Exercise context helpers against installed packages, including TypeScript contracts.

Run after build-packages.py. Registry access is disabled; packages are installed
from artifacts/packages and all native calls use binaries contained in them.
"""
import argparse
import hashlib
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "artifacts/packages"


def run(command, cwd, capture=False):
    return subprocess.run(list(map(str, command)), cwd=cwd, check=True, text=True,
                          stdout=subprocess.PIPE if capture else None).stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--consumer-dir", type=Path)
    compiler = os.environ.get("TSC_JS") or shutil.which("tsc")
    parser.add_argument("--typescript", type=Path,
                        default=Path(compiler).resolve() if compiler else None)
    args = parser.parse_args()
    if args.typescript is not None:
        args.typescript = args.typescript.resolve()
    manifest = json.loads((PACKAGES / "manifest.json").read_text())
    fingerprint = hashlib.sha256(json.dumps(manifest["files"], sort_keys=True).encode()).hexdigest()[:12]
    consumer = (args.consumer_dir or ROOT / "artifacts/consumers" / fingerprint).resolve()
    consumer.mkdir(parents=True, exist_ok=True)
    node = os.environ.get("NODE_BIN", "node")
    npm_cli = os.environ.get("NPM_CLI")
    npm = [node, npm_cli] if npm_cli else ["npm"]
    version = manifest["engineVersion"]
    package_names = manifest["files"]
    node_package = next(PACKAGES / name for name in package_names if name.endswith(".tgz"))
    python_package = next(PACKAGES / name for name in package_names if name.endswith(".whl"))
    for filename, digest in package_names.items():
        if hashlib.sha256((PACKAGES / filename).read_bytes()).hexdigest() != digest:
            raise AssertionError("Package checksum mismatch: " + filename)

    node_dir = consumer / "node"
    node_dir.mkdir(exist_ok=True)
    run(npm + ["install", "--offline", "--ignore-scripts", "--no-audit", "--no-fund",
               "--cache", consumer / "npm-cache", node_package], node_dir)
    (node_dir / "compression-check.cjs").write_text(NODE_CHECK)
    run([node, "compression-check.cjs"], node_dir)
    if args.typescript is None or not args.typescript.is_file():
        raise SystemExit("TypeScript compiler unavailable; pass --typescript /path/to/tsc.js")
    (node_dir / "compression-types.ts").write_text(TYPES_CHECK)
    run([node, args.typescript, "--strict", "--noEmit", "--target", "ES2022",
         "--module", "node16", "--moduleResolution", "node16", "compression-types.ts"], node_dir)

    python_dir = consumer / "python"
    python_dir.mkdir(exist_ok=True)
    venv = python_dir / "venv"
    if not venv.exists():
        run([sys.executable, "-m", "venv", venv], python_dir)
    python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    run([python, "-m", "pip", "install", "--no-index", "--no-deps", "--force-reinstall", python_package], python_dir)
    (python_dir / "compression_check.py").write_text(PYTHON_CHECK)
    run([python, "compression_check.py"], python_dir)

    dotnet_dir = consumer / "compression-dotnet"
    dotnet_dir.mkdir(exist_ok=True)
    (dotnet_dir / "CompressionChecks.csproj").write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType>'
        '<TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings>'
        '<Nullable>enable</Nullable></PropertyGroup><ItemGroup>'
        f'<PackageReference Include="SevenMLabs.Astrology" Version="{version}" />'
        '</ItemGroup></Project>')
    (dotnet_dir / "Program.cs").write_text(DOTNET_CHECK)
    run(["dotnet", "restore", "CompressionChecks.csproj", "--source", PACKAGES,
         "--packages", consumer / "nuget-cache", "--verbosity", "quiet", "-p:NuGetAudit=false"], dotnet_dir)
    run(["dotnet", "build", "--no-restore", "--verbosity", "quiet"], dotnet_dir)
    run(["dotnet", dotnet_dir / "bin/Debug/net10.0/CompressionChecks.dll"], dotnet_dir)
    summary = {"result": "passed", "engineVersion": version, "consumerFingerprint": fingerprint,
               "consumerDirectory": str(consumer),
               "installedSdkLanguages": ["node", "python", "dotnet"], "typescript": "passed",
               "checks": ["calculateWithContext", "compactRoundtrip", "inputUnchanged",
                          "budgetExceeded", "rawErrorEnvelope", "objectErrorException", "literalNulRejected",
                          "nonFiniteRejected"]}
    (ROOT / "artifacts/compression-binding-tests.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


NODE_CHECK = r'''const assert = require('node:assert/strict');
const sdk = require('@7mlabs/astrology');
const request = { operation: 'query', group: 'geometry', action: 'normalize', longitude: -10 };
const original = sdk.calculate(request);
const snapshot = JSON.stringify(original);
const pair = sdk.calculateWithContext(request);
assert.deepEqual(pair.result, original);
assert.equal(pair.context.data.format, 'astro-context/1');
assert.equal(pair.context.data.coverage.complete, true);
assert.deepEqual(sdk.expandContext(pair.context), original);
assert.deepEqual(JSON.parse(sdk.expandContextJson(sdk.compressPayloadJson(snapshot))), original);
assert.equal(JSON.stringify(original), snapshot);
const constrained = sdk.compressPayload(original, { mode: 'budgeted', maxTokens: 1 });
assert.equal(constrained.data.budget.exceeded, true);
for (const raw of ['{', 'null']) assert.ok(JSON.parse(sdk.compressPayloadJson(raw)).errors.length);
assert.ok(JSON.parse(sdk.compressPayloadJson(snapshot, '{"mode":"invalid"}')).errors.length);
assert.throws(() => sdk.compressPayload(original, { mode: 'invalid' }), e => !!e.code && !!e.result);
assert.ok(JSON.parse(sdk.expandContextJson('{}')).errors.length);
assert.throws(() => sdk.expandContext({}), e => !!e.code && !!e.result);
assert.throws(() => sdk.compressPayloadJson(snapshot + '\0'), TypeError);
assert.throws(() => sdk.expandContextJson('{}\0'), TypeError);
assert.throws(() => sdk.compressPayloadJson(original), TypeError);
for (const unsupported of [NaN, Infinity, -Infinity, undefined, () => 1, Symbol('fact')]) {
  assert.throws(() => sdk.compressPayload({ ...original, data: { unsupported } }), TypeError);
  assert.throws(() => sdk.expandContext({ ...pair.context, unsupported }), TypeError);
}
assert.throws(() => sdk.compressPayload(original, { mode: 'budgeted', maxTokens: NaN }), TypeError);
assert.throws(() => sdk.compressPayload({ ...original, [Symbol('fact')]: 1 }), TypeError);
console.log('PASS installed Node context helpers');
'''

TYPES_CHECK = r'''import { calculateWithContext, compressPayload, compressPayloadJson, expandContext,
  expandContextJson, CompressionOptions, ContextResult, GeometryNormalizeResult, NatalDomainsResult,
  ForecastResult, CoupleWithCompositeResult, RetainedPayloadResult } from '@7mlabs/astrology';
const options: CompressionOptions = { mode: 'focused', domains: ['career'], sections: ['profession'] };
const geometry = calculateWithContext({ operation: 'query', group: 'geometry', action: 'normalize', longitude: -10 });
const result: GeometryNormalizeResult = geometry.result;
const normalized: number = result.data.longitude;
const context: ContextResult = compressPayload(result);
const retained: RetainedPayloadResult = expandContext(context);
const birth = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10, longitude: 106 } };
const natal: NatalDomainsResult = calculateWithContext({ ...birth, operation: 'natalDomains' }, options).result;
const forecast: ForecastResult = calculateWithContext({ operation: 'forecast', birth, period: { kind: 'day', year: 2026, month: 10, day: 3 } }).result;
const couple: CoupleWithCompositeResult = calculateWithContext({ operation: 'couple', personA: birth, personB: birth, composite: { antipodalPolicy: 'lowerLongitude' } }).result;
const format: 'astro-context/1' = context.data.format;
const mode: 'compact' | 'focused' | 'budgeted' = context.data.mode;
const tokenMethod: 'utf8-bytes-conservative' = context.data.metrics.tokenEstimateMethod;
const json: string = expandContextJson(compressPayloadJson('{}'));
// @ts-expect-error Unknown compression modes are rejected by the typed API.
compressPayload(result, { mode: 'lossy' });
// @ts-expect-error The raw helper requires a JSON string.
compressPayloadJson(result);
// @ts-expect-error Domains are strings, not house numbers.
const invalid: CompressionOptions = { domains: [7] };
void [normalized, retained, natal, forecast, couple, format, mode, tokenMethod, json, invalid];
'''

PYTHON_CHECK = r'''import json
from sevenmlabs_astrology import (calculate, calculate_with_context, compress_payload,
    compress_payload_json, expand_context, expand_context_json, EngineError)
request = {"operation": "query", "group": "geometry", "action": "normalize", "longitude": -10}
original = calculate(request)
snapshot = json.dumps(original)
pair = calculate_with_context(request)
assert pair["result"] == original
assert pair["context"]["data"]["format"] == "astro-context/1"
assert pair["context"]["data"]["coverage"]["complete"] is True
assert expand_context(pair["context"]) == original
assert json.loads(expand_context_json(compress_payload_json(snapshot))) == original
assert json.dumps(original) == snapshot
assert compress_payload(original, {"mode": "budgeted", "maxTokens": 1})["data"]["budget"]["exceeded"] is True
for raw in ("{", "null"):
    assert json.loads(compress_payload_json(raw))["errors"]
assert json.loads(compress_payload_json(snapshot, '{"mode":"invalid"}'))["errors"]
try:
    compress_payload(original, {"mode": "invalid"})
    raise AssertionError("Expected EngineError")
except EngineError as error:
    assert error.code and error.result
assert json.loads(expand_context_json("{}"))["errors"]
try:
    expand_context({})
    raise AssertionError("Expected EngineError")
except EngineError as error:
    assert error.code and error.result
for call in (lambda: compress_payload_json(snapshot + "\0"), lambda: expand_context_json("{}\0")):
    try:
        call()
        raise AssertionError("Expected literal NUL rejection")
    except ValueError:
        pass
for unsupported in (float("nan"), float("inf"), float("-inf")):
    try:
        compress_payload({**original, "data": {"unsupported": unsupported}})
        raise AssertionError("Expected non-finite rejection")
    except ValueError:
        pass
print("PASS installed Python context helpers")
'''

DOTNET_CHECK = r'''using SevenMLabs.Astrology;
using System.Text.Json;
using System.Text.Json.Nodes;
void Check(bool condition, string label) { if (!condition) throw new Exception(label); }
void Same(JsonElement left, JsonElement right) => Check(JsonNode.DeepEquals(JsonNode.Parse(left.GetRawText()), JsonNode.Parse(right.GetRawText())), "JSON roundtrip differs");
var request = new { operation = "query", group = "geometry", action = "normalize", longitude = -10 };
var original = Engine.Calculate(request);
var snapshot = original.GetRawText();
var pair = Engine.CalculateWithContext(request);
Same(pair.Result, original);
Check(pair.Context.GetProperty("data").GetProperty("format").GetString() == "astro-context/1", "Wrong format");
Check(pair.Context.GetProperty("data").GetProperty("coverage").GetProperty("complete").GetBoolean(), "Incomplete compact context");
Same(Engine.ExpandContext(pair.Context), original);
using (var doc = JsonDocument.Parse(Engine.ExpandContextJson(Engine.CompressPayloadJson(snapshot)))) Same(doc.RootElement, original);
Check(original.GetRawText() == snapshot, "Original payload mutated");
var constrained = Engine.CompressPayload(original, new { mode = "budgeted", maxTokens = 1 });
Check(constrained.GetProperty("data").GetProperty("budget").GetProperty("exceeded").GetBoolean(), "Budget must report exceeded");
foreach (var raw in new[] { "{", "null" }) {
    using var doc = JsonDocument.Parse(Engine.CompressPayloadJson(raw));
    Check(doc.RootElement.GetProperty("errors").GetArrayLength() > 0, "Expected raw error envelope");
}
using (var doc = JsonDocument.Parse(Engine.CompressPayloadJson(snapshot, "{\"mode\":\"invalid\"}")))
    Check(doc.RootElement.GetProperty("errors").GetArrayLength() > 0, "Expected invalid options envelope");
try { Engine.CompressPayload(original, new { mode = "invalid" }); throw new Exception("Expected EngineException"); }
catch (EngineException error) { Check(error.Code.Length > 0 && error.ResultJson.Length > 0, "Error details missing"); }
using (var doc = JsonDocument.Parse(Engine.ExpandContextJson("{}")))
    Check(doc.RootElement.GetProperty("errors").GetArrayLength() > 0, "Expected invalid context envelope");
try { Engine.ExpandContext(new { }); throw new Exception("Expected EngineException"); }
catch (EngineException error) { Check(error.Code.Length > 0 && error.ResultJson.Length > 0, "Error details missing"); }
foreach (var call in new Func<string>[] { () => Engine.CompressPayloadJson(snapshot + "\0"), () => Engine.ExpandContextJson("{}\0") }) {
    try { call(); throw new Exception("Expected literal NUL rejection"); }
    catch (ArgumentException) { }
}
foreach (var unsupported in new[] { double.NaN, double.PositiveInfinity, double.NegativeInfinity }) {
    try { Engine.CompressPayload(new { schemaVersion = "1.0", data = new { unsupported } }); throw new Exception("Expected non-finite rejection"); }
    catch (ArgumentException) { }
}
JsonNode? nested = JsonValue.Create("leaf");
for (var index = 0; index < 60; index++) nested = new JsonObject { ["node"] = nested };
var deepPayload = new JsonObject {
    ["schemaVersion"] = "1.0", ["engineVersion"] = "deep-source", ["data"] = nested,
    ["warnings"] = new JsonArray(), ["errors"] = new JsonArray()
};
var deepContext = Engine.CompressPayload(deepPayload);
var deepRestored = Engine.ExpandContext(deepContext);
Check(JsonNode.DeepEquals(deepPayload, JsonNode.Parse(deepRestored.GetRawText(),
    documentOptions: new JsonDocumentOptions { MaxDepth = 128 })), "Deep object API roundtrip differs");
Console.WriteLine("PASS installed .NET context helpers");
'''


if __name__ == "__main__":
    main()
