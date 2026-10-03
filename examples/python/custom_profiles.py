import json
from sevenmlabs_astrology import calculate

result = calculate({
  "operation": "natalDomains",
  "utc": {
    "year": 2000,
    "month": 1,
    "day": 1,
    "hour": 12,
    "minute": 0
  },
  "location": {
    "latitude": 10.8231,
    "longitude": 106.6297
  },
  "domains": [],
  "rulership": "traditional",
  "aspectPreset": "extended",
  "customProfiles": [
    {
      "id": "personalGrowth",
      "version": "1.0",
      "houses": [
        1,
        9
      ],
      "bodies": [
        "sun",
        "moon",
        "mercury"
      ],
      "angles": [
        "ascendant"
      ],
      "sections": [
        {
          "id": "study",
          "houses": [
            3,
            9
          ],
          "bodies": [
            "mercury"
          ]
        }
      ]
    }
  ]
})
print(json.dumps(result, indent=2))
