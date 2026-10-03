const { calculate } = require('@7mlabs/astrology');
const request = {
  "operation": "forecast",
  "birth": {
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
  "period": {
    "kind": "day",
    "year": 2026,
    "month": 3,
    "day": 3,
    "utcOffsetMinutes": 420
  }
};
console.log(JSON.stringify(calculate(request), null, 2));
