import json
from sevenmlabs_astrology import calculate

request = json.loads('''
{
  "operation": "couple",
  "personA": {
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
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": {
      "year": 1998,
      "month": 6,
      "day": 15,
      "hour": 6,
      "minute": 30
    },
    "location": {
      "latitude": 21.0278,
      "longitude": 105.8342
    },
    "houseSystem": "wholeSign"
  },
  "domains": [
    "attraction",
    "communication",
    "emotionalConnection",
    "longTerm",
    "sharedResources",
    "homeFamily"
  ],
  "rulership": "traditional",
  "aspectPreset": "extended",
  "composite": {
    "houseMethod": "wholeSignFromMidpointAscendant"
  }
}
''')
print(json.dumps(calculate(request), indent=2))
