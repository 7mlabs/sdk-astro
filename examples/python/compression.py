"""Prepare an LLM context with the same native core as Node/.NET/Rust/C."""
import json
import sys
from sevenmlabs_astrology import calculate, compress_payload, expand_context, calculate_with_context

request = {
    "operation": "natalDomains",
    "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
    "location": {"latitude": 10.8231, "longitude": 106.6297},
    "domains": ["career", "love"], "aspectPreset": "extended",
}
result = calculate(request)
compact = compress_payload(result)
assert expand_context(compact) == result

calculation = calculate_with_context(request, {
    "mode": "budgeted", "domains": ["career"], "maxBytes": 64 * 1024,
})
context = calculation["context"]
retained = expand_context(context)
assert "love" in calculation["result"]["data"]["domains"]
assert list(retained["data"]["domains"]) == ["career"]
print(json.dumps({
    "compact": compact["data"]["metrics"],
    "focused": context["data"]["metrics"],
    "coverage": context["data"]["coverage"],
    "omitted": context["data"]["omitted"],
    "budget": context["data"]["budget"],
    "warnings": context["warnings"],
}, indent=2))
if "--payload" in sys.argv:
    print(json.dumps(context, separators=(",", ":"), ensure_ascii=False))
