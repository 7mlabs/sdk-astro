"""Verify installed packages, shared-core parity and real report compression offline.

Run after scripts/test-packages.py with the same Python/Node environment.
No model tokenizer is required: measured savings are UTF-8 bytes, not tokens.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
evidence = json.loads((ROOT / "artifacts/test-results.json").read_text())
consumer = ROOT / "artifacts/consumers" / evidence["consumerFingerprint"]
node = os.environ.get("NODE_BIN", "node")
sdk = consumer / "node/node_modules/@7mlabs/astrology"
python = consumer / "python/venv/bin/python"
node_cli = consumer / "node/compression-parity.cjs"
node_cli.write_text("""const fs=require('node:fs');
const sdk=require('@7mlabs/astrology');
const input=fs.readFileSync(0,'utf8');
if(process.argv[2]==='--expand-context-json') console.log(sdk.expandContextJson(input));
else { const wrapper=JSON.parse(input);
 console.log(sdk.compressPayloadJson(JSON.stringify(wrapper.payload),JSON.stringify(wrapper.options||{}))); }
""")
python_cli = consumer / "python/compression_parity.py"
python_cli.write_text("""import json,sys
from sevenmlabs_astrology import compress_payload_json, expand_context_json
raw=sys.stdin.read()
if sys.argv[1]=='--expand-context-json': print(expand_context_json(raw))
else:
 wrapper=json.loads(raw)
 print(compress_payload_json(json.dumps(wrapper['payload']),json.dumps(wrapper.get('options',{}))))
""")
commands = {
    "node": [node, node_cli],
    "python": [python, python_cli],
    "dotnet": ["dotnet", consumer / "dotnet/bin/Debug/net10.0/Example.dll"],
    "rust": [ROOT / "examples/rust/target/debug/astrology-rust-example"],
    "c": [consumer / "c-example"],
}
checks = 0


def run(language, action, value):
    global checks
    raw = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    completed = subprocess.run(list(map(str, commands[language])) + [action],
        input=raw, text=True, capture_output=True, check=True, cwd=ROOT)
    assert completed.stderr == "", (language, completed.stderr)
    output = completed.stdout.strip()
    result = json.loads(output)
    if action == "--compress-json" and not result["errors"]:
        assert result["data"]["metrics"]["outputBytes"] == len(output.encode("utf-8")), language
        assert result["data"]["metrics"]["estimatedTokens"] == len(output.encode("utf-8")), language
    checks += 1
    return result


def envelope(data):
    return {"schemaVersion": "1.0", "engineVersion": evidence["engineVersion"],
            "data": data, "warnings": [], "errors": []}


source = json.loads((ROOT / "examples/natal-result.json").read_text())
cases = [
    {"payload": source},
    {"payload": source, "options": {"mode": "budgeted", "maxBytes": 64}},
    {"payload": source, "options": {"mode": "budgeted", "maxTokens": 64}},
    {"payload": source, "options": {"mode": "compact", "maxBytes": 0}},
    {"payload": source, "options": {"mode": "focused", "domains": ["missing"]}},
    {"payload": envelope({"collision": {"$astroRef": "user value"},
        "nullable": None, "applying": None, "number": 280.36891967534336,
        "rows": [{"id": "sun", "speed": None}, {"id": "moon", "speed": 13.1}],
        "unicode": "Công việc — 情感"})},
]
for index, wrapper in enumerate(cases):
    reference = run("rust", "--compress-json", wrapper)
    for language in commands:
        result = run(language, "--compress-json", wrapper)
        assert result == reference, (index, language, "compression parity")
        if not result["errors"]:
            decoded = run(language, "--expand-context-json", result)
            assert decoded == wrapper["payload"], (index, language, "roundtrip")

# Verify source payloads containing megabytes of real house/aspect evidence.
measurements = []
for stem in ["natal", "natal-domains", "couple", "composite", "couple-composite",
             "forecast-day", "forecast-month", "forecast-year", "events-year",
             "custom-profiles", "custom-couple", "custom-composite", "custom-forecast"]:
    original = json.loads((ROOT / "examples" / (stem + "-result.json")).read_text())
    started = time.monotonic()
    context = run("node", "--compress-json", {"payload": original})
    assert not context["errors"], (stem, context["errors"])
    assert context["data"]["coverage"]["complete"] is True, stem
    assert context["data"]["omitted"] == [], stem
    assert run("node", "--expand-context-json", context) == original, stem
    metrics = context["data"]["metrics"]
    measurements.append({"fixture": stem, "inputBytes": metrics["inputBytes"],
        "outputBytes": metrics["outputBytes"],
        "byteReductionPercent": round((1 - metrics["outputBytes"] / metrics["inputBytes"]) * 100, 2),
        "roundtripSeconds": round(time.monotonic() - started, 3)})

for stem, domain in [("natal-domains", "career"), ("couple", "communication"),
                     ("composite", "love"), ("forecast-day", "career")]:
    original = json.loads((ROOT / "examples" / (stem + "-result.json")).read_text())
    wrapper = {"payload": original, "options": {"mode": "focused", "domains": [domain]}}
    context = run("rust", "--compress-json", wrapper)
    assert not context["errors"], (stem, context["errors"])
    assert context["data"]["coverage"]["complete"] is False, stem
    assert context["data"]["omitted"], stem
    decoded = run("rust", "--expand-context-json", context)
    # Retaining the shared namespace guarantees every selected report can join its facts.
    assert decoded["warnings"] == original["warnings"], stem
    for field in ["context", "subjects", "natal", "subject", "events", "snapshot"]:
        if field in original["data"]:
            assert decoded["data"][field] == original["data"][field], (stem, field)
    for language in ["node", "python", "dotnet", "c"]:
        assert run(language, "--compress-json", wrapper) == context, (stem, language)

digest = hashlib.sha256((ROOT / "artifacts/packages/manifest.json").read_bytes()).hexdigest()
summary = {"result": "passed", "engineVersion": evidence["engineVersion"],
    "consumerFingerprint": evidence["consumerFingerprint"], "packageManifestSha256": digest,
    "languages": list(commands), "checks": checks, "realFixtureRoundtrips": len(measurements),
    "measurements": measurements, "tokenCounting": "UTF-8 byte conservative estimate; no exact tokenizer benchmark"}
(ROOT / "artifacts/compression-test-results.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
