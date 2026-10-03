const { calculate, calculateJson } = require('@7mlabs/astrology');
if (process.argv[2]) process.stdout.write(calculateJson(process.argv[2]));
else console.log(JSON.stringify(calculate({ operation: 'natal',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }
}), null, 2));
