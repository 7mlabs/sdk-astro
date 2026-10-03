# Payload ngày, tháng, năm và sự kiện theo natal

`0.8.0-alpha.1` bổ sung hai operations chạy offline trong cùng Rust core: `events` quét lịch sự kiện thiên văn và `forecast` gắn lịch đó với natal của một cá nhân. Cùng request period dùng cho ngày, tháng hoặc năm. Node/Python/.NET chỉ gọi core cục bộ; không cần server, credential hoặc tải dữ liệu khi tính.

`forecast` trả dữ liệu để dựng thông điệp ngày và báo cáo tổng quan, gồm một natal/context dùng chung, snapshot transit, các mốc sự kiện chính xác, vị trí transit trong nhà natal, góc với natal, chủ tinh và views theo 10 lĩnh vực/30 sections. `messageContext.narrative` là `null`: engine cung cấp facts và references để ứng dụng viết nội dung. Không trả điểm tốt/xấu hoặc kết luận rằng một sự kiện chắc chắn xảy ra trong đời người dùng.

## Gọi SDK

```js
const { calculate } = require('@7mlabs/astrology');
const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  houseSystem: 'placidus'
};
const daily = calculate({ operation: 'forecast', birth,
  period: { kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 }
});
console.log(daily.data.domains.love.messageContext);
console.log(daily.data.events.map(e => e.personalImpact));
const monthly = calculate({ operation: 'forecast', birth,
  period: { kind: 'month', year: 2026, month: 3, utcOffsetMinutes: 420 }
});
console.log(monthly.data.overview, monthly.data.domains.career);
const annual = calculate({ operation: 'events', period: { kind: 'year', year: 2026 } });
console.log(annual.data.events);
```

Để quét năm **có đối chiếu cá nhân**, dùng `forecast` với `birth` và `period.kind: 'year'`. `events` không cần birth và không tạo `personalImpact`. Mỗi birth input được tính độc lập; bản này không forecast synastry/composite/Davison.

## Request và validation

| Trường | Mặc định | Contract |
|---|---|---|
| `operation` | Bắt buộc | `events` hoặc `forecast` |
| `period` | Bắt buộc | Day/month/year dưới đây |
| `birth` | Bắt buộc với forecast | Cùng birth contract [natal.md](natal.md); không gửi với events |
| `bodies` | Sun–Pluto | 1–10 unique IDs; lọc bodies cho ingress, station, planetary aspects và exact natal transits |
| `eventTypes` | Cả 6 nhóm | Unique ingress/station/lunarPhase/planetaryAspect/solarEclipse/lunarEclipse; `[]` chỉ giữ snapshot và optional natal transits |
| `aspectPreset` | `major` | `major` hoặc `extended`, loại trừ với aspectRules |
| `aspectRules` | Theo preset | 0–16 unique angles, orb theo contract hiện có; `[]` tắt matching và exact aspect search |
| `includeNatalTransits` | `true` | Chỉ forecast; tìm các lần exact của transit với 26 điểm natal cố định |
| `domains` | 10 lĩnh vực | Chỉ forecast; IDs giống cá nhân, không phải 6 IDs synastry |
| `customProfiles` | Không có | Chỉ forecast; 1–8 profiles, tối đa 8 sections/profile; custom-only dùng domains: [] |
| `rulership` | `traditional` | Chỉ forecast; traditional/modern cho chủ tinh natal và expansion |

Lunar phases và eclipses dùng Sun/Moon độc lập với `bodies`. Ví dụ bodies chỉ có Saturn thì snapshot và exact natal transits chỉ có Saturn, nhưng eclipse vẫn trả Sun/Moon tại cực đại. Muốn tắt các nhóm này, chọn `eventTypes` cụ thể.

```json
{"kind":"day","year":2026,"month":3,"day":3,"utcOffsetMinutes":420}
{"kind":"month","year":2026,"month":3,"utcOffsetMinutes":420}
{"kind":"year","year":2026,"utcOffsetMinutes":0}
```

Đây là ba object period riêng, không phải một JSON request nhiều dòng. Day cần month/day; month cấm day; year cấm month/day. UTC offset là số phút cố định từ -840 đến 840, mặc định 0. Không suy offset từ tọa độ sinh, không tra IANA hoặc daylight saving. Caller chuyển giờ sinh thành UTC và chọn offset kỳ tính riêng. Ngày Gregorian phải hợp lệ; year 1800–2399; window sau đổi offset phải nằm trong UTC 1800-01-01 đến 2400-01-01. Không tự cắt kỳ tính tại giới hạn coverage.

Các integer fields chuẩn hóa chính xác raw JSON decimal/exponent (`2026.0`, `2.026e3`), từ chối tiny fractions, strings, booleans, null, unknown fields và options sai cấp. Tất cả options, period và birth được validate trước khi tính; lỗi trả `data: null`, không có kết quả một phần. Structural schemas: [events request](../schemas/events-request.schema.json), [forecast request](../schemas/forecast-request.schema.json). Core còn kiểm tra calendar, offset-adjusted bounds và unique custom IDs/angles.

## Các nhóm sự kiện

| Type | Phép tính | Details |
|---|---|---|
| `ingress` | Longitude đi qua biên zodiac 30° | boundaryLongitude, fromSign/toSign, direct/retrograde; giữ cả lần đi lùi |
| `station` | Longitude speed đổi dấu qua 0 | direction retrograde/direct |
| `lunarPhase` | Moon minus Sun longitude đi qua 0/90/180/270° | newMoon/firstQuarter/fullMoon/lastQuarter |
| `planetaryAspect` | Hai bodies transit đạt góc theo rules | angle và branchLongitude; cặp body theo thứ tự canonical Sun–Pluto; branch là longitude body thứ nhất trừ body thứ hai |
| `solarEclipse` | Swiss global solar eclipse search | Loại eclipse, flags và các mốc global quanh cực đại |
| `lunarEclipse` | Swiss global lunar eclipse search | Loại eclipse, flags và các mốc penumbral/partial/total |
| `natalTransit` | Transit body đạt góc với một điểm natal cố định | Chỉ forecast; targetPointId N:..., targetLongitude, angle/branchLongitude; branch là transit longitude trừ targetLongitude natal |

Một new/full Moon có thể đồng thời có event lunarPhase và event planetaryAspect Sun/Moon; eclipse maximum là một mốc riêng. Types giữ nghĩa riêng, không gộp chúng thành một sự kiện giả. Mọi lần exact được giữ, gồm các lượt đi tới/đi lùi/đi tới của cùng body/target/angle trong mùa retrograde.

Eclipse lấy **global maximum** bằng API Swiss Moshier, không suy eclipse chỉ từ conjunction/opposition. Nó không cho biết người dùng nhìn thấy eclipse tại nơi ở. Contacts có tên, UTC/local và JD riêng: Solar eclipseBegin/End và centralPhaseBegin/End; Lunar penumbralBegin/End, partialBegin/End, totalityBegin/End khi tồn tại. Contacts của eclipse có thể nằm ngoài period nếu maximum nằm trong period; period membership xét maximum.

Nguồn thuật toán provider: [Swiss programming interface](https://www.astro.com/swisseph/swephprg.htm). Các mốc kiểm chứng độc lập: [USNO primary Moon phases](https://aa.usno.navy.mil/data/MoonPhases) và [NASA eclipse calendar 2026](https://eclipse.gsfc.nasa.gov/OH/OH2026.html). Engine sử dụng dữ liệu compile sẵn; không gọi các trang này lúc runtime.

## Quét và độ chính xác

Scanner dùng UT1 JD cho tìm nghiệm, provider tính vị trí apparent tropical/geocentric ở time model Swiss; UTC/TT/UT1 giữ đúng giao diện provider. Quét grid 6 giờ, chia khoảng tại extrema của relative velocity, unwrap longitude và tìm crossing trên từng đoạn. Refine dùng bracket với provider thật; không gọi một contact là exact chỉ vì snapshot nằm trong orb. Hai roots gần station và repeated hits được kiểm thử riêng.

Khi secant/bisection dừng do không còn mốc JD biểu diễn được giữa hai endpoints, scanner có thể kiểm tra tối đa 128 mốc JD lân cận trong khoảng gốc. Chỉ nhận mốc có residual từ provider thực đạt cùng tolerance và có bracket đổi dấu bao quanh theo cùng chiều, rộng tối đa `0.25 s`; nếu không đạt vẫn trả lỗi. `maximumRepresentableTimeProbes` ghi giới hạn bổ sung này. Cơ chế xử lý nhiễu tốc độ ở độ phân giải rất nhỏ, không nới tolerances hoặc dùng tốc độ nội suy làm kết quả.

- Angular root residual tối đa `1e-6°`, bracket tối đa `0.25 s`.
- Station root residual tối đa `1e-8°/day`.
- Internal relative-velocity extrema tolerance `1e-7°/day`, tangency matching `1e-10°`; đây là quy tắc tìm/isolating roots, không phải orb của report.
- Eclipse dùng `precision.method: swissEclipseSearch`, bracket/residual/units là null vì API không cung cấp chúng. Angular/station events dùng `bracketedRoot` với precision thực từng event.
- Tối đa 30.000 events và 2.000.000 provider body evaluations/request; vượt budget trả lỗi, không truncate. `calculation.search` khai báo grid/tolerances/budgets và `truncated: false`.

Các tolerances trên là mức hội tụ **số học của scanner**, không là cam kết độ chính xác thiên văn tuyệt đối. Provider Moshier và Delta T/leap-second model tích hợp vẫn có giới hạn như [natal.md](natal.md). Event UTC/local biểu diễn millisecond và có thể giữ second 60.x khi provider trả leap second; birth input không nhận leap second. Membership dùng interval `[start, end)`, dữ liệu period khai báo start/end UTC và JD.

## Forecast response và ảnh hưởng theo natal

```text
data.chartKind = individualForecast
data.subjectCount = 1
data.subject = { id: N, natal, context, calculation }
data.period = { kind, year, month, day, utcOffsetMinutes, startUtc, endUtc, ... }
data.snapshot = { utc, local, julianDayUt1, positions, relations, aspects, houseOverlays }
data.events = [ event + personalImpact + highlightReasons ]
data.domains/customDomains/profileCatalog
data.overview
```

Natal/context được lưu một lần. Subject giữ 10 bodies, 4 angles, H1–H12, 26 points/325 relations, rulers/occupants và advanced facts như natal cá nhân. Forecast không sao chép 10 full natal reports vào từng event. `snapshot` nằm ở **midpoint của period**: day thường ở 12:00 local; month/year là snapshot đại diện, còn toàn bộ kỳ dùng event search. Nó chỉ có bodies geocentric, không tạo ASC/MC/houses của transit khi không có current location.

Snapshot có đủ selectedBodies × 26 raw relations (mặc định 260), aspects theo orb rules và overlay transit trong nhà natal. Applying dùng **speed transit so với natal longitude cố định**, không dùng speed hành tinh lúc sinh. Tại exact hoặc instantaneous speed không xác định chiều, applying null. Các snapshot contacts là trạng thái tại một thời điểm; exact event list là kết quả quét toàn kỳ.

Mỗi event có positions của body tham gia tại đúng mốc; personalImpact trả:

- `primaryNatalPointId`/`primaryAngle`: target chính của natalTransit; null với global event.
- `houseOverlays`: T:body nằm trong N:Hn, chủ tinh N:body và occupants natal có thể resolve.
- `contacts`: các góc của event bodies với đủ natal bodies/angles/cusps theo orb policy; indexes local trong event.
- `affectedNatalPointIds`: endpoints của các contacts và primary exact target.
- `affectedNatalHouseIds`: nhà nhận transit, nhà chứa natal endpoints và nhà do natal bodies được chạm làm chủ tinh.
- `rulerLinks`: natal body endpoints → những nhà do body đó làm chủ tinh.

`affected` là liên hệ hình học và selection evidence, không là tuyên bố về mức độ ảnh hưởng thực tế. Exact target vẫn có reference riêng khi maxOrb 0 khiến contact list không có match do residual số học; không làm tròn orb để giả exact bằng 0. Các điều kiện dignity/chains/reception trong subject là facts **của natal**, không bị tính lại như một chart chung giữa hai epochs.

## 10 lĩnh vực, messageContext và references

Forecast dùng cùng catalog 10 lĩnh vực/30 sections và [custom selectors](profiles.md). Selectors áp dụng vào **natal**: focus bodies/angles cùng occupants/rulers của nhà được chọn. Events được đưa vào view khi có contact với natal point được chọn, primary exact target được chọn, hoặc transit đi qua focus house. Reasons công khai: natalContact, exactNatalTarget, transitThroughFocusHouse.

Sections chọn độc lập, có thể mở rộng ngoài focus của parent. Parent view hợp nhất refs từ sections, giữ `primaryContact` theo selectors parent; sectionIds chỉ những sections có evidence. `primaryNatalPointIds`/`primaryNatalHouseIds` giữ parent scope; `natalPointIds`/`natalHouseIds` là union. Mỗi view có snapshotAspectIndexes, snapshotOverlayPointIds, eventReferences, highlightEventIds và sections. `messageContext.kind` là dailyMessage/monthlyOverview/yearlyOverview; chứa refs cần để dựng thông điệp/tổng quan, narrative null.

Qualified IDs `N:sun`, `N:H7` resolve trong `data.subject.context` sau bỏ prefix N; `T:moon` resolve trong snapshot/event positions tương ứng. Snapshot aspect indexes resolve tại snapshot; event contact indexes chỉ resolve trong personalImpact của event đó. EventReferences/messageContext/overview/section eventIds resolve trong `data.events`; không join với natal aspects, synastry hoặc composite chỉ vì cùng numeric index. Event IDs là references trong response; dùng type/bodies/target/time nếu đối chiếu giữa các requests có grid khác.

## Tổng quan tháng/năm và sự kiện nổi bật

`overview` có eventCount/byType, counts theo nhà natal và theo domain, highlightEventIds, days/months chứa event IDs. Days chỉ có ngày có events; yearly months có đủ 12 tháng kể cả rỗng. Một event có thể liên quan nhiều nhà/lĩnh vực nên không cộng counts các nhóm để suy ra eventCount.

Highlights dùng policy `sevenmlabs-calendar-highlights` version 1.0, không tính importance score:

| Reason | Policy |
|---|---|
| eclipse | Solar/lunar eclipse maximum |
| station | Các lần đổi chiều được chọn |
| newOrFullMoon | New/full Moon |
| slowBodyIngress | Ingress của Mars/Jupiter/Saturn/Uranus/Neptune/Pluto |
| slowBodyPlanetaryAspect | Cặp không có Moon, có ít nhất một body trong nhóm trên |
| nonLunarNatalTransit | Exact natal transit của body khác Moon |

Overview giữ toàn bộ events và highlight subset; view highlights còn phải có selection evidence trong domain. Các policy là lựa chọn phần mềm được công bố, có thể thay version sau; không là quy chuẩn chung về “quan trọng” cho mọi người.

## Các ngôn ngữ và examples

Cùng object JSON ở đầu tài liệu dùng cho các bindings:

```python
from sevenmlabs_astrology import calculate
result = calculate({"operation": "forecast", "birth": birth,
    "period": {"kind": "month", "year": 2026, "month": 3, "utcOffsetMinutes": 420}})
print(result["data"]["overview"])
```

```csharp
using SevenMLabs.Astrology;
var result = Engine.Calculate(new { operation = "forecast", birth,
    period = new { kind = "year", year = 2026, utcOffsetMinutes = 420 } });
Console.WriteLine(result.GetProperty("data").GetProperty("overview"));
```

```rust
println!("{}", astro_core::calculate_json(&request_json));
```

```c
char *result = astro_calculate_json(request_json);
if (result) { puts(result); astro_free_string(result); }
```

Node calculate/Python calculate/.NET Calculate throw khi core có errors; raw JSON APIs giữ error envelope. Các biến birth/request ở snippets là input đã khai báo, không có tự tìm dữ liệu sinh. Native calls synchronous; scans tháng/năm có thể lâu hơn natal, ứng dụng Node/UI nên gọi trong worker để giữ giao diện phản hồi. Year all-bodies/full-targets/extended có nhiều events và refs; dùng bodies/eventTypes/domains/customProfiles phù hợp khi chỉ cần subset. Tắt một nhóm không có nghĩa nhóm đó đã được quét và không tìm thấy gì.

Samples có request và kết quả từ package đã cài:

| Kỳ tính | Request | Kết quả |
| --- | --- | --- |
| Ngày cá nhân | [day request](../examples/forecast-day-request.json) | [day result](../examples/forecast-day-result.json) |
| Tháng cá nhân | [month request](../examples/forecast-month-request.json) | [month result](../examples/forecast-month-result.json) |
| Năm cá nhân | [year personal request](../examples/forecast-year-request.json) | [year personal result](../examples/forecast-year-result.json) |
| Năm thiên văn | [year sky request](../examples/events-year-request.json) | [year sky result](../examples/events-year-result.json) |
| Profile tùy chỉnh | [custom request](../examples/custom-forecast-request.json) | [custom result](../examples/custom-forecast-result.json) |

Year personal sample lọc Jupiter–Pluto; year sky sample dùng toàn bộ Sun–Pluto. Dedicated examples [Node](../examples/node/forecast.cjs), [Python](../examples/python/forecast.py), [.NET --forecast](../examples/dotnet/Program.cs); Rust/C CLI nhận raw JSON. Các counts kiểm chứng được ghi tại [testing.md](testing.md).

## Phạm vi tiếp theo và frontend

Hiện có transit longitude/aspects vào natal, ingress, stations, primary Moon phases và global solar/lunar eclipses. Chưa có local eclipse visibility, transit horizon angles/houses từ current location, sidereal/topocentric, nodes/asteroids/fixed stars, progression, solar-return chart, event duration/orb-entry-exit windows, forecasting chart đôi hoặc narrative templates.

Frontend có thể thêm day/month/year calendar, chọn body/family/domain, timeline exact contacts, highlight filter, lunar/eclipse labels, event drawer với natal overlays/contacts/ruler links và evidence navigation. Day panel đọc messageContext; month/year đọc overview và event references. Hiển thị cả precision/method/options/coverage. Browser runtime WASM chưa có; UI native có thể gọi package, browser tĩnh cần provider WASM riêng. Không render một lunarPhase như eclipse hay dùng natal house placement làm visibility tại nơi ở.
