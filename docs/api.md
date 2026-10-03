# API natal, cặp đôi, composite, forecast và events

Phiên bản `0.9.0-alpha.1` có operation `natal` nhận ngày giờ UTC và tọa độ, xem [contract natal](natal.md) và `schemas/natal-request.schema.json`. `natalDomains` dùng cùng birth input để trả các nhóm dữ liệu theo lĩnh vực; xem [contract lĩnh vực](domains.md) và `schemas/natal-domains-request.schema.json`. Payload `natalDomains` là **một người** (`chartKind: individualNatal`, `subjectCount: 1`), có derived facts, 10 lĩnh vực/30 sections và custom profiles; xem [profile contract](profiles.md); xem [contract báo cáo cá nhân](individual-reports.md). `couple` nhận `personA`/`personB` birth inputs để tính hai natal và dữ liệu so sánh theo 6 lĩnh vực; xem [contract cặp đôi](couple.md). `composite` nhận cùng hai birth inputs để dựng chart C và 10 lĩnh vực/30 sections; `couple` có thể bật C bằng `composite: {}`; xem [contract composite](composite.md). `events` quét ngày/tháng/năm theo period; `forecast` thêm birth natal, event impacts và views/messageContext theo 10 lĩnh vực; xem [forecast.md](forecast.md). `query` chọn phép tính theo 4 groups/8 actions; xem [query.md](query.md). Ba operations geometry `chart`, `harmonic`, `synastry` nhận positions theo contract dưới đây; schema nằm ở `schemas/geometry-request.schema.json`.

| Operation | Input chính | Data trả về |
|---|---|---|
| `natal` | UTC và tọa độ | Một natal cơ bản |
| `natalDomains` | UTC, tọa độ và options lĩnh vực | Một natal, full context, 10 builtin views và custom views |
| `couple` | Hai birth inputs và options lĩnh vực | Hai natal/full contexts, 676 cross-relations, aspects, overlays, paired views và optional composite C |
| `composite` | Hai birth inputs và options midpoint/nhà/profile | Hai natal nguồn và chart C/full context, 10 builtin views cùng custom views |
| `events` | Period ngày/tháng/năm, families/bodies/rules | Global events, midpoint snapshot, coverage và numeric search metadata |
| `forecast` | Birth natal và period ngày/tháng/năm | Shared natal, snapshot, global/exact natal events, personalImpact, 10 views/30 sections và overview |
| `chart` | Positions và cusps optional | Placements, aspects, midpoints |
| `harmonic` | Positions và factor | Geometry sau biến đổi harmonic |
| `synastry` | Hai tập positions | Cross-aspects và house overlays |

`natalDomains` mặc định chọn cả 10 lĩnh vực cá nhân; `couple` mặc định chọn 6 lĩnh vực cặp đôi. `composite` mặc định chọn 10 lĩnh vực như cá nhân. Ba operations này mặc định rulership traditional và aspect preset extended, nhận `domains`, `customProfiles`, `rulership`, `aspectPreset`; các trường đó trong `natal` trả lỗi. `natalDomains` và C dùng catalog 10 IDs; synastry `couple` dùng catalog 6 IDs riêng. Custom profiles và references luôn thuộc context của chart được chọn. `aspectPreset` không được kết hợp với `aspectRules`. Context cá nhân giữ 26 points và 325 relations, cross context cặp đôi giữ 52 points và 676 relations kể cả khi caller tắt aspects bằng `aspectRules: []`; domains chỉ chọn views, không loại full natal/context.

## Input chung

```json
{
  "operation": "chart",
  "positions": [
    { "id": "moon", "longitude": 359, "speed": 13 },
    { "id": "sun", "longitude": 1, "speed": 1 }
  ]
}
```

Đây là dữ liệu toán học tổng hợp để kiểm tra wrap, không phải chart thiên văn của một người thật. `positions` có 1–64 bodies; ID không rỗng, unique trong từng chart, tối đa 128 UTF-8 bytes. Longitude là số hữu hạn, được normalize vào `[0,360)`. Speed optional/null, degrees/day, âm nghĩa retrograde. Unknown speed không được coi là zero; `isRetrograde`/`applying` có thể null.

`aspectRules` optional; mặc định angles 0/60/90/120/180 với maxOrb 8/6/8/8/8 độ. Đây là một policy minh bạch có thể thay, không phải chuẩn bắt buộc của mọi trường phái. Tối đa 16 rules, angles unique từ 0 đến 180 và orb 0–15. `[]` tắt aspects. Unknown fields bị từ chối để tránh typo im lặng.

`houses` optional/null: 12 cusp longitudes theo thứ tự nhà 1–12. Phải đi quanh zodiac đúng một vòng, không trùng cusp. Vị trí trên cusp thuộc nhà bắt đầu tại cusp đó. Không có houses → house null; operation `chart` không tự tạo Placidus hoặc Whole Sign; dùng `natal` để tính nhà từ thời gian/địa điểm.

## Chart

`chart` trả placements, aspects và toàn bộ midpoints theo cặp. Không chấp nhận harmonic hoặc chart B. Midpoint dùng cung ngắn; cặp đối đỉnh có hai midpoint tương đương nên trả `longitude: null`, `ambiguous: true`.

`applying` được suy từ relative longitude speed, là chỉ báo cục bộ tuyến tính. Khi speed thiếu hoặc aspect exact, trả null. Không quảng bá là exact-event search; gần các cusp của khoảng cách góc cần dùng module event theo thời gian khi có provider.

## Harmonic

```json
{"operation":"harmonic","harmonic":5,"positions":[{"id":"a","longitude":0},{"id":"b","longitude":72}]}
```

Factor integer 1–360; JSON `5.0` hoặc `5e0` được chuẩn hóa thành `5`, còn `5.5` bị từ chối. Tọa độ mới là `(longitude × factor) mod 360`, speed được nhân factor, aspects/sign/midpoints được tính lại. Không nhận houses hoặc chart B; không giữ nhầm natal house. Output có warning rằng đây là tọa độ biến đổi, không là bầu trời vật lý mới.

## Couple từ birth input

`couple` tính natal cho từng `personA`/`personB`, có house system độc lập, rồi trả dữ liệu so sánh. Không truyền top-level `utc`, `location`, `positions`, `houses` hoặc `houseSystem`. Chọn `domains` chỉ lọc views, vẫn giữ hai natal và cross context đầy đủ. `aspectRules: []` tắt matching contacts nhưng giữ đủ 676 cross-relations và overlays. Schema và cách join namespace nằm trong [couple.md](couple.md). Khi thêm `composite: {}`, child options chọn house method, antipodal policy và 10 IDs lĩnh vực cá nhân; root `domains` vẫn chọn 6 lĩnh vực synastry. C kế thừa rulership/aspect rules ở root, trả tại `data.composite`. Không bật C thì output cũ giữ nguyên shape.

## Composite từ birth input

`operation: "composite"` dựng chart C theo `shortestArcMidpoint`. Chọn `houseMethod: "midpoint"` (mặc định) hoặc `"wholeSignFromMidpointAscendant"`; chọn `antipodalPolicy: "error"` (mặc định) hoặc `"lowerLongitude"`. Houses midpoint sai thứ tự trả lỗi, không tự đổi phương pháp. C trả `chart`, `context`, `domains`, `customDomains`, `profileCatalog`, `provenance`; đủ 26 points/325 relations/66 house-ruler pairs. Speeds, retrograde và applying là null vì C là phép dựng tượng trưng. Xem [composite.md](composite.md) cho request/schema, metadata và reference joins.

## Forecast và event search

Các requests dùng `period: { kind: "day" | "month" | "year", year, ... }` và fixed `utcOffsetMinutes`. `forecast` cần `birth`, tạo natal một lần, root-solve exact transits tới 26 điểm cố định và trả vị trí transit trong nhà natal cùng ruler/contact evidence. Default aspect preset là major; lunar/eclipses dùng Sun/Moon độc lập với filter bodies. Snapshot và events có scope/indexes riêng; numeric roots, eclipse maxima, highlight policy, validation và schemas trong [forecast.md](forecast.md). Engine không viết narrative; `messageContext` là payload bằng chứng.

## Synastry từ positions

```json
{"operation":"synastry","positions":[{"id":"a","longitude":10}],"otherPositions":[{"id":"b","longitude":130}]}
```

Positions là A; otherPositions là B. `houses` thuộc A, `otherHouses` thuộc B. Trả `personA`, `personB`, `crossAspects`, `overlaysAtoB`, `overlaysBtoA`. Cross-aspect body1 luôn thuộc A; body2 thuộc B. `applying` null vì hai natal speeds không mô tả evolution của cùng một bầu trời. Operation geometry `synastry` không dựng composite; chart C có operation birth-input riêng. Không trả compatibility score hoặc giải thích tâm lý.

## Output và lỗi

Success envelope gồm schemaVersion, engineVersion, calculation, data, warnings, errors. `composite` có scope `midpoint-composite-data`, metadata construction `symbolic` và time/provider metadata riêng của hai natal nguồn; C không có UTC/location/JD. Khi bật trong `couple`, metadata C nằm ở `calculation.composite`. Metadata geometry scope là `geometry-from-positions`, provider `supplied-positions`. Natal có scope `basic-natal`, provider `swiss-ephemeris`, ephemeris `moshier` và đầy đủ time/frame/units. `couple` có metadata riêng cho hai subjects; xem [couple metadata](couple.md#metadata-và-lỗi). `natalDomains` có scope `natal-domain-data`, bổ sung `aspectPreset`, `rulership`, `domainProfile`; data chứa `natal`, `context`, `domains`.

Domain context aspects dùng `point1`/`point2` vì endpoints có thể là body, angle hoặc cusp H1–H12. Chúng có `index` toàn cục trong response; domain aspects giữ index đó và thêm `selectedEndpoints`. Natal/geometry body aspects vẫn dùng `body1`/`body2`. Xem [quy tắc join và coverage](domains.md#response) trước khi lọc/render dữ liệu.

`calculateJson`/`calculate_json`/`CalculateJson` trả raw JSON kể cả lỗi validation. `calculate`/`Calculate` trả object và throw error có code khi envelope có errors. Lỗi wrapper (type sai, binary thiếu, NUL hoặc input quá lớn) có thể được throw trước khi vào core. C ABI input có hợp đồng riêng về pointer.

Các mã core: `INVALID_INPUT`, `INPUT_TOO_LARGE`, `CALCULATION_FAILED`, `INTERNAL_ERROR`. Error envelope có data null và errors dạng `{code,message}`. Core giới hạn request 4 MiB; FFI C caller phải bảo đảm chuỗi hợp lệ trước khi core đọc.

## Những API chưa có

Timezone IANA conversion, sidereal/topocentric, solar-return chart, local eclipse visibility, orb-entry/exit windows, Davison, fixed stars, progression, SVG renderer và MCP chưa được triển khai trong Rust alpha. Các API legacy .NET không biến thành API mới chỉ vì cùng tên astrology.
