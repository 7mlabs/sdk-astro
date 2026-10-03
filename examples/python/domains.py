import json
from sevenmlabs_astrology import calculate

result = calculate({"operation": "natalDomains",
    "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
    "location": {"latitude": 10.8231, "longitude": 106.6297},
    "domains": ["career", "love", "relationships", "family", "finance", "identity", "learning", "creativity", "innerLife", "dailyLife"],
    "rulership": "traditional", "aspectPreset": "extended"
})
print(json.dumps(result, indent=2))
