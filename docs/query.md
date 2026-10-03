# Tính và truy vấn theo nhóm

`0.9.0-alpha.1` bổ sung operation `query` và các nhóm SDK `geometry`, `aspects`, `houses`, `points`. Dev chọn một phép tính hoặc một phần natal cần dùng; cùng Rust core thực hiện cho Node/Python/.NET/Rust/C, không cần server.

## Các nhóm và phương thức

| Nhóm | Phương thức | Input chính | Dữ liệu trả |
| --- | --- | --- | --- |
| geometry | normalize | longitude | Longitude 0–360, zodiac sign và độ trong cung |
| geometry | separation | longitude1, longitude2 | Góc nhỏ 0–180 và signedDelta có hướng |
| geometry | midpoint | longitude1, longitude2 | Trung điểm cung ngắn, policy cho đối đỉnh |
| aspects | between | 2 positions, rule, motionMode | Góc tùy ý, orb, matched, applying và distanceRate |
| aspects | inspect | birth, pointIds, aspect policy | Raw pairs và góc giữa các điểm natal được chọn |
| houses | locate | longitude, houseCusps | Nhà chứa một longitude theo 12 cusps cung cấp |
| houses | inspect | birth, houseNumbers | Nhà natal, cusp, chủ tinh, occupants và các liên hệ |
| points | inspect | birth, pointIds | Các điểm natal được chọn và liên hệ với phần còn lại |

Mọi phương thức SDK nhận **một options object/dictionary**, trả nguyên envelope `{schemaVersion, engineVersion, data, calculation, warnings, errors}`. Các helper không chứa công thức riêng. SDK tự đặt operation/group/action; caller không được đặt lại ba trường đó trong options. `calculate` và các raw JSON APIs vẫn dùng được.

## Gọi Node.js

```js
const { geometry, aspects, houses, points } = require('@7mlabs/astrology');
const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  houseSystem: 'placidus'
};

const angle = geometry.separation({ longitude1: 359, longitude2: 1 });
console.log(angle.data.separation); // 2 degrees

const arbitrary = aspects.between({
  positions: [
    { id: 'moving', longitude: 350, speed: 1 },
    { id: 'fixed', longitude: 28, speed: 4 }
  ],
  rule: { angle: 37.5, maxOrb: 1 },
  motionMode: 'fixedSecond'
});
console.log(arbitrary.data.aspect); // orb 0.5, matched true, applying true

const seventhHouse = houses.inspect({ birth, houseNumbers: [7] });
const financeHouses = houses.inspect({ birth, houseNumbers: [2, 8], rulership: 'modern' });
const selectedPoints = points.inspect({ birth, pointIds: ['sun', 'midheaven', 'H7'] });
const selectedAngles = aspects.inspect({ birth, pointIds: ['sun', 'moon'],
  aspectRules: [{ angle: 37.5, maxOrb: 15 }] });
console.log(seventhHouse.data.houses, selectedAngles.data.relations, selectedAngles.data.aspects);
```

`aspects.between` luôn trả relation/aspect, kể cả `matched: false`. `aspects.inspect` giữ raw relations nhưng chỉ đưa matches vào `aspects`; một mảng aspects rỗng không có nghĩa không tính cặp đó. Góc chiếu là độ tách nhỏ nhất trong `[0, 180]`, nhận góc lẻ tùy ý như 37.5 hoặc 72.25. `maxOrb` trong `[0, 15]`; không giới hạn vào các góc phổ biến. Longitude đầu vào có thể âm hoặc vượt 360 và được chuẩn hóa trước tính.

## Raw JSON và các ngôn ngữ

Request dưới đây tương đương `houses.inspect`:

```json
{
  "operation": "query",
  "group": "houses",
  "action": "inspect",
  "birth": {
    "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
    "location": { "latitude": 10.8231, "longitude": 106.6297 },
    "houseSystem": "placidus"
  },
  "houseNumbers": [7]
}
```

```python
from sevenmlabs_astrology import houses, aspects
result = houses.inspect({"birth": birth, "houseNumbers": [7]})
result = aspects.inspect({"birth": birth, "pointIds": ["sun", "moon"],
    "aspectRules": [{"angle": 37.5, "maxOrb": 15}]})
```

```csharp
using SevenMLabs.Astrology;
var result = Engine.Houses.Inspect(new { birth, houseNumbers = new[] { 7 } });
var angle = Engine.Geometry.Separation(new { longitude1 = 359, longitude2 = 1 });
```

Rust dùng `astro_core::calculate_json(&request_json)`; C dùng `astro_calculate_json(request_json)` rồi `astro_free_string`. Các biến birth/request_json trong snippets do caller khai báo như Node/raw request phía trên. SDK object APIs throw khi core có errors; raw JSON APIs giữ error envelope và `data: null`.

## Quy tắc geometry và motion

- `signedDelta` đo **longitude thứ hai trừ longitude thứ nhất**, được wrap vào `[-180, 180)`. Separation là trị tuyệt đối; đơn vị degrees.
- Midpoint mặc định `antipodalPolicy: error`. Hai longitudes đối đỉnh trong tolerance `1e-10°` có hai trung điểm và trả `CALCULATION_FAILED`; chọn `lowerLongitude` rõ ràng để lấy candidate có longitude nhỏ hơn. Policy này tương thích [composite](composite.md).
- `motionMode` mặc định `none`, trả applying/distanceRate null. `relative` dùng speed2 − speed1, cần cả hai tốc độ; `fixedSecond` coi target thứ hai đứng yên, dùng −speed1 và giữ nguyên metadata speed caller gửi.
- `distanceRate` là tốc độ thay đổi separation, degrees/day. Applying nghĩa separation đang tiến về góc yêu cầu, có thể được tính kể cả ngoài orb. Nếu thiếu tốc độ, ở separation 0/180 hoặc đã exact trong tolerance `1e-12°`, phần chiều không xác định trả null theo contract. Khi exact, distanceRate có thể vẫn có giá trị nếu separation không ở biên; applying vẫn null.
- `houses.locate` nhận đúng 12 cusps hữu hạn, đi quanh zodiac đúng một vòng, không trùng. Cusp thuộc chính nhà bắt đầu tại cusp đó; cusp kế tiếp thuộc nhà kế tiếp. Có wrap H12/H1. Đây là phân nhà theo ecliptic longitude, không phải house-position 3D.

## Inspection natal và phạm vi selection

Birth contract giữ [natal.md](natal.md): Gregorian UTC 1800–2399, tọa độ hợp lệ, Placidus hoặc Whole Sign. `rulership` mặc định traditional, có modern; aspectPreset mặc định major, có extended, loại trừ với aspectRules. Rules có tối đa 16 góc unique; `aspectRules: []` tắt matched aspects và vẫn giữ raw relations.

- `houses.inspect`: houseNumbers gồm 1–12 số nguyên unique trong 1..12, mặc định tất cả. Primary points gồm các cusps được chọn, occupants và rulers của những nhà đó. Chọn contacts/pairs có ít nhất một endpoint primary.
- `points.inspect`: pointIds gồm 1–26 IDs unique, mặc định tất cả. Chọn pairs có ít nhất một endpoint primary.
- `aspects.inspect`: pointIds gồm 2–26 IDs unique, mặc định tất cả. Chỉ chọn pairs có **cả hai endpoints** trong selection. Chọn đúng hai IDs để tính đúng một cặp natal với rule tùy ý.
- 26 IDs hiện có: Sun–Pluto dùng lowercase body IDs, ascendant/midheaven/descendant/imumCoeli và H1–H12. Không tự tính tọa độ Chiron/nodes/asteroids; `aspects.between` có thể dùng ID tùy ý với tọa độ caller cung cấp.

Data inspection gồm selection, primaryPointIds, primaryHouseIds, points, houses, relations, aspects và coverage. Points còn có supporting endpoints để mọi tham chiếu resolve trong response; số phần tử points có thể lớn hơn pointIds. Houses được chọn có đầy đủ rulerPlacement/occupants. Aspect indexes được đánh lại từ 0 trong response query; không join chúng bằng index của natalDomains/couple/forecast khác.

`calculation.computationalScope` khai báo rõ: helpers dùng `pureGeometry`; inspection dùng `fullNatalThenSelection`. Provider hiện vẫn tính natal và shared context rồi chọn payload cần trả, không quảng bá rằng chọn một nhà sẽ chỉ tính một cusp hoặc tiết kiệm đúng tỷ lệ CPU. Đây là API gọi/lấy dữ liệu theo nhu cầu; có thể tối ưu nội bộ sau khi đo mà giữ contract.

Mọi options và selector được kiểm tra trước provider. Null, unknown fields, duplicate selectors, ngày sai và options sai action bị từ chối; không trả một phần natal nếu provider lỗi. Số nhà raw JSON `7.0`/`7e0` được chuẩn hóa chính xác; `7.0000000000000001` bị từ chối. Dùng raw JSON API khi cần giữ literal trước khi runtime ngôn ngữ làm tròn số.

## Schemas, examples và integrations

Schemas: [query request](../schemas/query-request.schema.json), [query response](../schemas/query-response.schema.json). Semantic validator kiểm tra wrap/midpoint/motion, cusps, complete selected pairs, local indexes và endpoint closure; schema validation riêng không thay thế kiểm tra ngày Gregorian/cusp traversal của core.

| Ví dụ | Request | Result |
| --- | --- | --- |
| Nhà tài chính | [houses request](../examples/query-houses-request.json) | [houses result](../examples/query-houses-result.json) |
| Body/angle/cusp | [points request](../examples/query-points-request.json) | [points result](../examples/query-points-result.json) |
| Góc natal tùy ý | [aspects request](../examples/query-aspects-request.json) | [aspects result](../examples/query-aspects-result.json) |
| Hai tọa độ tùy ý | [between request](../examples/query-between-request.json) | [between result](../examples/query-between-result.json) |

Dedicated calls: [Node](../examples/node/query.cjs), [Python](../examples/python/query.py), [.NET --query](../examples/dotnet/Program.cs). MCP và frontend host dùng chính các requests này, xem [integrations.md](integrations.md). Kết quả cài package thật và parity tại [testing.md](testing.md).
