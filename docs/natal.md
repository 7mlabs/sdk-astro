# Raw birth data → natal chart cơ bản

Bản `0.9.0-alpha.1` tính thiên văn thực bằng Swiss Ephemeris native/Moshier. Node, Python, .NET, Rust và C cùng dùng provider này; không gọi HTTP, process Python hoặc dịch vụ bên ngoài.

Tài liệu này mô tả operation `natal` cơ bản. [Operation natalDomains](domains.md) bổ sung views theo lĩnh vực, chủ tinh/occupants, body–angle–cusp aspects và đủ pair relations bằng cùng birth input/provider. Shape và defaults của `natal` được giữ riêng.

## Request

```json
{
  "operation": "natal",
  "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0, "second": 0 },
  "location": { "latitude": 10.8231, "longitude": 106.6297 },
  "houseSystem": "placidus"
}
```

Schema: `schemas/natal-request.schema.json`. Request đầy đủ chạy được: `examples/natal-request.json`. Thời điểm này là 19:00 giờ Việt Nam; caller phải đổi giờ địa phương sang UTC, kể cả thay đổi ngày/tháng/năm. Engine không đoán timezone từ tọa độ.

- `year/month/day/hour/minute` nhận JSON number có giá trị nguyên: `2000`, `2000.0`, `2e3` đều được chuẩn hóa về integer nội bộ/output. Số có phần lẻ, numeric string, boolean và null bị từ chối; không làm tròn/cắt phần lẻ, kể cả `2000.0000000000000001`.
- Calendar Gregorian, năm 1800–2399, tháng 1–12; ngày được kiểm tra theo tháng và năm nhuận.
- `hour` 0–23, `minute` 0–59 bắt buộc; `second` mặc định 0, hữu hạn trong `[0,60)`. Chưa nhận leap-second input `60`.
- Latitude north-positive, longitude east-positive, đơn vị độ. Tọa độ `(0,0)` hợp lệ. Latitude phải nằm giữa hai cực; longitude từ -180 đến 180.
- `houseSystem`: `placidus` mặc định hoặc `wholeSign`. Placidus không khả dụng ở một số vĩ độ cao sẽ trả `CALCULATION_FAILED`, không tự đổi sang Porphyry. Geographic poles ±90 bị từ chối cho cả hai hệ nhà.
- `aspectRules` dùng contract chung trong `api.md`; mặc định 0/60/90/120/180 với orb 8/6/8/8/8 độ. Mảng rỗng tắt aspects.
- Unknown fields bị từ chối. Không nhận positions, timezone, sidereal, topocentric hay ephemerisPath trong natal request này. `domains`, `rulership`, `aspectPreset` yêu cầu operation `natalDomains`; không dùng chúng với `natal`.

## Response

Envelope chung: `schemaVersion`, `engineVersion`, `calculation`, `data`, `warnings`, `errors`. Output đầy đủ từ request trên lưu tại `examples/natal-result.json`.

| Trường trong data | Ý nghĩa |
|---|---|
| `utc`, `location` | Dữ liệu thời gian và tọa độ caller đã truyền |
| `placements` | 10 bodies theo thứ tự sun, moon, mercury, venus, mars, jupiter, saturn, uranus, neptune, pluto |
| `angles` | ascendant, midheaven, descendant, imumCoeli, cùng longitude/sign/degreeInSign |
| `houses` | 12 nhà, mỗi nhà có number 1–12 và cusp longitude/sign/degreeInSign |
| `houseCusps` | Mảng 12 longitude theo thứ tự nhà, dùng lại cho geometry API |
| `aspects` | Aspects giữa 10 bodies; để lấy body-to-angle/cusp aspects dùng `natalDomains` |

Mỗi placement có longitude `[0,360)`, ecliptic latitude, distanceAu, speed degrees/day, isRetrograde, signIndex 0–11, sign, degreeInSign và house 1–12. Gán nhà theo ecliptic longitude nằm giữa hai cusps; chưa áp dụng house-position 3D theo latitude của thiên thể. Speed âm là retrograde. Angles không tính tốc độ hoặc house placement, nên các trường này trả null.

Ví dụ kết quả Sun: longitude `280.3689238651°`, Capricorn `10.3689238651°`, speed `1.0194320961°/day`, nhà 6. ASC `119.0161707961°`, MC `29.1379867619°`. Đây là số tính cho request trên, không phải fixture tọa độ được caller nhập.

## Provider và time model

Source Swiss 2.10.03 được pin commit và ghi trong `neutral-engine/crates/astro-provider-swiss/PROVENANCE.md`. Mode cố định Moshier analytical, geocentric, tropical, apparent ecliptic-of-date. Không dùng JPL/Swiss binary data files trong runtime. Flag trả về phải xác nhận Moshier; nếu provider không trả đúng mode hoặc số không hữu hạn, engine trả lỗi.

Swiss chuyển UTC sang TT/UT1 bằng compiled leap-second và Delta T tables. Trước 1972, hàm upstream coi civil input là UT1. Future UTC dùng mô hình tích hợp và không có live Earth-orientation corrections; warning luôn có trong natal envelope. Metadata trả julianDayTt/julianDayUt1 để caller kiểm tra epoch thực. Không cam kết độ chính xác UTC lịch sử/tương lai ngoài mô hình này.

Bản vá build `ASTRO_MOSHIER_ONLY` chặn đọc ephemeris/leap-second/Delta T files từ ngoài. Mutex giữ toàn bộ chu kỳ provider; cleanup gọi swe_close cả khi lỗi. Các consumer không cần cài Swiss, Rust, Python hay .NET bên cạnh runtime của ngôn ngữ đang dùng.

## Lỗi và phạm vi kiểm chứng

`INVALID_INPUT`: ngày giờ/tọa độ/options không hợp lệ. `CALCULATION_FAILED`: provider hoặc house system thất bại. Các lỗi dùng `data: null`; high-level wrappers throw exception có code, low-level JSON functions giữ nguyên envelope.

Đã kiểm tra calendar, leap day, tọa độ 0, thời gian thay đổi, hai bán cầu, high latitude, giới hạn năm, concurrent options và parity năm ngôn ngữ. Có bốn reference cases so PySwissEph và vị trí Skyfield/JPL ở cùng TT. Endpoint tests không chứng minh mọi ngày trong 1800–2399; xem `testing.md` cho tolerance và kết quả.

Mở rộng lĩnh vực hiện tổ chức facts của natal theo [profile công khai](domains.md), bổ sung derived facts và report context cá nhân trong [individual-reports.md](individual-reports.md), không bổ sung thiên thể hoặc time model mới. Chart C đã có operation riêng, xem [composite.md](composite.md). Transit/natal event search và calendar ngày/tháng/năm đã có, xem [forecast.md](forecast.md). Progression, return charts, Davison, timezone helper, frontend, MCP và public registry release chưa được triển khai.
