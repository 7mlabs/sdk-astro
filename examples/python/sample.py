import json
import sys
from sevenmlabs_astrology import calculate, calculate_json

if len(sys.argv) > 1:
    print(calculate_json(sys.argv[1]))
else:
    print(json.dumps(calculate({"operation": "natal",
        "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
        "location": {"latitude": 10.8231, "longitude": 106.6297}
    }), indent=2))
