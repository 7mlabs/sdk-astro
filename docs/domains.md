# Payload natal theo lĩnh vực

Bản `0.9.0-alpha.1` cung cấp operation `natalDomains`: nhận dữ liệu sinh như natal cơ bản, tính **một lá số cá nhân** rồi trả dữ liệu nâng cao để dựng báo cáo thuộc 10 lĩnh vực cá nhân, hoặc lĩnh vực tùy chỉnh do dev định nghĩa. Package tính cục bộ qua cùng Rust core. `love` và `relationships` dùng natal của người đó, không cần hoặc so sánh chart người thứ hai.

Profile `sevenmlabs-domain-selection` phiên bản `2.0` là bộ quy tắc chọn dữ liệu được dự án công bố. Mapping lĩnh vực và orb là cấu hình của profile, không phải chuẩn duy nhất của mọi trường phái. Output không có điểm số, lời luận giải hoặc dự đoán. View trọng tâm được giữ; `domain.report` bổ sung nhà hỗ trợ và facts nâng cao có evidence. Xem [contract báo cáo cá nhân](individual-reports.md) cho cách join, quy tắc dẫn xuất và giới hạn.

## Request

```json
{
  "operation": "natalDomains",
  "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
  "location": { "latitude": 10.8231, "longitude": 106.6297 },
  "houseSystem": "placidus",
  "domains": ["career", "love", "relationships", "family", "finance", "identity", "learning", "creativity", "innerLife", "dailyLife"],
  "rulership": "traditional",
  "aspectPreset": "extended"
}
```

`utc`, `location` và `houseSystem` có cùng validation như [natal cơ bản](natal.md): UTC Gregorian 1800–2399, tọa độ north/east-positive, Placidus hoặc Whole Sign. Caller chuyển giờ địa phương sang UTC. Engine từ chối fields lạ và options không hợp lệ.

Schema nằm tại [natal-domains-request.schema.json](../schemas/natal-domains-request.schema.json). [Request mẫu](../examples/natal-domains-request.json) và source [Node.js](../examples/node/domains.cjs)/[Python](../examples/python/domains.py) sử dụng contract này.

| Trường | Mặc định | Quy tắc |
|---|---|---|
| `domains` | Cả 10 lĩnh vực | Mảng tối đa 10 builtin IDs duy nhất; `[]` chỉ hợp lệ khi có `customProfiles` không rỗng |
| `customProfiles` | Không có | Mảng 1–8 profile tùy chỉnh; định nghĩa và validation trong [profiles.md](profiles.md) |
| `rulership` | `traditional` | `traditional` hoặc `modern` |
| `aspectPreset` | `extended` | `extended` hoặc `major`; không đi cùng `aspectRules` |
| `aspectRules` | Theo preset | Rules tùy chỉnh theo [API chung](api.md); `[]` tắt matching aspects |

Bỏ `domains` tạo đủ 10 nhóm, ngay cả khi request có thêm custom profiles. Để chỉ lấy custom profiles, truyền `domains: []` và ít nhất một `customProfiles`. ID lạ, ID lặp hoặc không chọn profile nào trả `INVALID_INPUT`. Explicit null không thay thế cho việc bỏ trường. Operation `natal` cơ bản không nhận `domains`, `customProfiles`, `rulership` hoặc `aspectPreset`; để sử dụng chúng phải đổi operation thành `natalDomains`.

Trong `0.4.0-alpha.1`, bỏ `domains` chỉ tạo năm nhóm cũ. Caller cần giữ phạm vi đó có thể truyền rõ `domains: ["career", "love", "relationships", "family", "finance"]`. Mapping của năm nhóm này không đổi; metadata selection profile chuyển sang `2.0` và report bổ sung sections. Đọc [migration](profiles.md#migration-từ-040-alpha1) trước khi phụ thuộc vào số keys hoặc kích thước response.

Ví dụ tùy chỉnh chỉ hai góc và lấy một lĩnh vực:

```json
{
  "operation": "natalDomains",
  "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
  "location": { "latitude": 10.8231, "longitude": 106.6297 },
  "domains": ["career"],
  "aspectRules": [{ "angle": 0, "maxOrb": 3 }, { "angle": 90, "maxOrb": 3 }]
}
```

## Profile chọn dữ liệu

| Lĩnh vực | Nhà trọng tâm | Thiên thể trọng tâm | Góc trọng tâm |
|---|---|---|---|
| `career` | H2, H6, H10 | Sun, Mercury, Jupiter, Saturn | MC |
| `love` | H5, H7, H8 | Moon, Venus, Mars | DSC |
| `relationships` | H3, H7, H11 | Moon, Mercury, Venus | ASC, DSC |
| `family` | H3, H4, H5 | Sun, Moon, Saturn | IC |
| `finance` | H2, H8, H10, H11 | Venus, Jupiter, Saturn | MC |
| `identity` | H1 | Sun, Moon | ASC |
| `learning` | H3, H9 | Mercury, Jupiter | — |
| `creativity` | H5 | Sun, Venus, Mercury | — |
| `innerLife` | H4, H8, H12 | Moon, Saturn, Neptune | IC |
| `dailyLife` | H1, H6, H12 | Sun, Moon, Mars, Saturn | ASC |

Mỗi builtin profile có ba mục con; xem catalog và selectors đầy đủ trong [profiles.md](profiles.md). Ví dụ `love` có `romance`, `marriage`, `intimacy`; `family` có `roots`, `home`, `parenting`. Sections dùng cùng natal, có thể thêm selectors ngoài focus của parent và được hợp nhất vào report để giữ đầy đủ evidence.

IDs output dùng chữ thường cho bodies: `sun`, `moon`, `mercury`, `venus`, `mars`, `jupiter`, `saturn`, `uranus`, `neptune`, `pluto`. Góc dùng `ascendant`, `midheaven`, `descendant`, `imumCoeli`. Cusps dùng `H1` đến `H12`.

Mỗi profile mở rộng tập thiên thể từ ba nguồn: thiên thể trọng tâm, thiên thể đang ở các nhà trọng tâm, và chủ tinh của những nhà đó. Một body chỉ xuất hiện một lần trong nhóm `bodies`, nhưng có thể có nhiều `selectionReasons`:

```json
[
  { "rule": "profileBody" },
  { "rule": "houseOccupant", "houseId": "H10" },
  { "rule": "houseRuler", "houseId": "H2" }
]
```

Đoạn trên minh họa shape của reasons; reasons thực phụ thuộc lá số. Tập `pointIds` của lĩnh vực gồm các bodies đã chọn, góc trọng tâm và cusps của các nhà trọng tâm. Một aspect/relation thuộc lĩnh vực khi **ít nhất một endpoint** nằm trong tập này. Endpoint còn lại có thể là bất kỳ body, angle hoặc cusp nào trong context; quy tắc này giữ các contacts với thiên thể ngoài nhóm ban đầu.

## Chủ tinh và dispositors

Bảng `rulershipRulesVersion: "1.0"` dùng một chủ tinh cho mỗi cung:

| Cung | Traditional | Modern |
|---|---|---|
| Aries | Mars | Mars |
| Taurus | Venus | Venus |
| Gemini | Mercury | Mercury |
| Cancer | Moon | Moon |
| Leo | Sun | Sun |
| Virgo | Mercury | Mercury |
| Libra | Venus | Venus |
| Scorpio | Mars | Pluto |
| Sagittarius | Jupiter | Jupiter |
| Capricorn | Saturn | Saturn |
| Aquarius | Saturn | Uranus |
| Pisces | Jupiter | Neptune |

Chủ tinh nhà được lấy từ cung tại cusp của nhà. `rulerPlacement` là vị trí của body đó trong natal. `context.dispositors` trả 10 records, mỗi record liên kết body với chủ tinh cung của body qua `bodyId`, `sign`, `dispositorBodyId`, `dispositorPlacement`. `context.advanced.dispositorChains` bổ sung path đầy đủ, self-ruler terminal hoặc cycle; `context.advanced.receptions` có mutual domicile receptions theo bảng rulership được chọn. Không tự khẳng định có một final dispositor duy nhất cho toàn natal.

`context.houseRulerRelations` giữ đủ 66 cặp H1–H12, với ruler IDs và tham chiếu tới raw relation/aspects giữa rulers. Hai nhà cùng ruler trả `sameRuler: true`, `relationId: null`, `aspectIndexes: []`, không tạo self-aspect.

## Aspect presets và toàn bộ quan hệ góc

| Góc độ | Major maxOrb | Extended maxOrb |
|---|---|---|
| 0 | 8 | 8 |
| 30 | — | 2 |
| 45 | — | 2 |
| 60 | 6 | 6 |
| 72 | — | 2 |
| 90 | 8 | 8 |
| 120 | 8 | 8 |
| 135 | — | 2 |
| 144 | — | 2 |
| 150 | — | 3 |
| 180 | 8 | 8 |

Đơn vị góc/orb là độ. Matching dùng `abs(separation - angle) <= maxOrb`; nhiều custom rules có thể cùng match một cặp. Rules thực được trả lại trong `calculation.aspectRules`; `calculation.aspectPreset` là `major`, `extended` hoặc `custom`.

Context có **26 điểm**: 10 bodies + 4 angles + 12 cusps. Engine tính đủ **325 cặp không thứ tự**, không có self-pair. Mỗi `context.relations` record có:

| Trường | Ý nghĩa |
|---|---|
| `id` | ID cặp theo thứ tự points, ví dụ `sun:moon` |
| `point1`, `point2` | IDs của hai endpoint trong `context.points` |
| `signedDelta` | Longitude point2 trừ point1, normalize vào `[-180,180)` |
| `separation` | Giá trị tuyệt đối của signedDelta, từ 0 đến 180 |
| `aspectIndexes` | Các indexes trong `context.aspects` khớp rules; rỗng nếu không match |

Ngay cả `aspectRules: []`, 26 points và 325 relations vẫn có đủ; `context.aspects`, `natal.aspects` và aspects trong các nhóm trả mảng rỗng. Nhờ vậy caller có thể đọc khoảng cách giữa bất kỳ cặp nào mà không bị giới hạn bởi preset.

`context.aspects` dùng shape `{index, point1, point2, angle, separation, orb, maxOrb, applying}`. Bao phủ body–body, body–angle, body–cusp, angle–angle, angle–cusp và cusp–cusp. Bodies có tốc độ thực; angles/cusps chưa có speed nên contacts với chúng trả `applying: null`. Khi aspect exact, applying cũng null như API geometry.

Angles và cusps có thể trùng longitude nhưng vẫn giữ ID riêng theo ý nghĩa. Ví dụ trong Placidus, ASC/H1 hoặc MC/H10 có thể tạo conjunction hình học; đó không phải hai thiên thể độc lập. Whole Sign giữ actual MC và cusp H10 riêng, không lấy H10 thay MC. Không cộng số contact để suy ra điểm lĩnh vực trong engine này.

## Response

Envelope vẫn gồm `schemaVersion`, `engineVersion`, `calculation`, `data`, `warnings`, `errors`. `calculation.scope` là `natal-domain-data`; metadata provider/time/units giống natal, bổ sung `aspectPreset`, `rulership` và `domainProfile`.

| Trường | Nội dung |
|---|---|
| `data.chartKind`, `data.subjectCount` | `"individualNatal"`, `1` |
| `data.natal` | Natal đầy đủ: utc/location, 10 placements, 4 angles, 12 houses/cusps, body–body aspects |
| `data.context.points` | 26 points; `kind` là `body`, `angle` hoặc `houseCusp` |
| `data.context.houses` | 12 nhà có id/sign/cusp, occupants, rulerBodyId và rulerPlacement |
| `data.context.dispositors` | 10 body→dispositor records |
| `data.context.relations` | 325 cặp, gồm cả cặp không match aspect |
| `data.context.aspects` | Tất cả matches trên 26 points theo rules |
| `data.context.houseRulerRelations` | 66 cặp nhà với liên hệ giữa rulers |
| `data.context.advanced` | Versioned rules, body states, distributions, dispositor chains, receptions và supported body-only aspect patterns |
| `data.context.coverage` | Counts cố định và `aspectMatching: "configuredRulesOnly"` |
| `data.profileCatalog` | Định nghĩa của cả 10 builtin profiles, gồm selectors và sections |
| `data.domains.<id>` | View cho từng builtin lĩnh vực được yêu cầu |
| `data.customDomains.<id>` | View cho từng custom profile; không trộn vào builtin domains |
| `data.domains.<id>.report` | Nhà liên quan có roles/evidence, points/aspects/conditions/chains/receptions/patterns để dựng báo cáo cá nhân |

`data.natal` giữ shape của natal cơ bản. Nó dùng cùng aspect rules của request `natalDomains`, vì vậy extended mặc định có thể cho nhiều body–body aspects hơn `operation: "natal"` với major mặc định; tọa độ và nhà của cùng birth input giữ nguyên.

Mỗi domain view có `origin` (`builtin` hoặc `custom`), `definition`, `profileId`, `profileVersion`, `selectionRules`, `houses`, `bodies`, `angles`, `pointIds`, `relationIds`, `aspects` và `report`. `bodies` chứa `{placement, selectionReasons}`. Aspects trong view giữ `index` toàn cục của `context.aspects` và bổ sung `selectedEndpoints`; index không phải vị trí của record trong mảng domain đã lọc. IDs/indexes dùng để join trong **cùng response**; không lấy index của một request để đọc request khác.

`report` version `1.1` dùng policy `primaryProfileWithTraceableSupport`: chọn focus houses, nhà đặt/làm ruler của primary bodies, nhà đặt/làm ruler của bodies trong dispositor paths, nhà đặt focus angles và primary aspect endpoints. Sau đó lấy mọi contact trên cusp/occupants/ruler của các nhà đã chọn. Không mở rộng nhà một cách đệ quy. `report.aspects` thêm `primaryContact` và `relatedHouseIds`; `report.houses` thêm `roles`, `evidence`, `aspectIndexes`, `rulerPlacementHouseId`, `rulerDispositorChainId`. Đây là payload nên dùng để dựng báo cáo sâu; context chung vẫn đủ 12 nhà và 325 raw pairs. `report.sections` tổ chức các chủ đề nhỏ của profile bằng evidence references. Parent report hợp nhất dữ liệu của mọi section, không lặp full report ở từng mục. [individual-reports.md](individual-reports.md) mô tả cách join các mục con.

Chọn riêng `domains: ["career"]` vẫn trả full natal/context; chỉ thu hẹp `data.domains`. Có thể chọn custom profiles cùng builtin profiles hoặc dùng riêng chúng; context vẫn tính một lần cho cùng natal. Số aspects và số bodies được chọn tùy lá số/rules; không có số lượng aspect cố định cho mỗi lĩnh vực.

[Payload đầy đủ từ package đã cài](../examples/natal-domains-result.json) đi kèm request mẫu. Các contacts có thể được nhiều domain chọn lại; `index` dùng để nhận biết cùng một fact. Context có cả body/body, body/angle, body/cusp, angle/angle, angle/cusp và cusp/cusp. Caller có thể tra `context.points[].kind` để lọc loại contacts cần dùng.

## Gọi từ SDK

Sau khi [cài artifact local](packages.md), Node.js gọi cùng hàm đồng bộ `calculate`:

```js
const { calculate } = require('@7mlabs/astrology');
const result = calculate({
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['career', 'love'],
  rulership: 'traditional',
  aspectPreset: 'extended'
});

console.log(result.data.context.coverage); // 10 bodies, 4 angles, 12 cusps, 325 pairs
console.log(result.data.domains.career.houses);
console.log(result.data.domains.love.aspects);
console.log(result.data.domains.career.report.houses);
console.log(result.data.domains.career.report.bodyFacts);
console.log(result.data.context.advanced.aspectPatterns);
console.log(result.data.domains.love.report.sections);

const byId = new Map(result.data.context.points.map(point => [point.id, point]));
for (const aspect of result.data.domains.career.aspects) {
  console.log(byId.get(aspect.point1), byId.get(aspect.point2), aspect.orb);
}
```

Python dùng cùng payload:

```python
from sevenmlabs_astrology import calculate

result = calculate({
    "operation": "natalDomains",
    "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
    "location": {"latitude": 10.8231, "longitude": 106.6297},
    "domains": ["family", "finance"],
})
print(result["data"]["domains"]["family"]["bodies"])
```

.NET dùng `Engine.Calculate`:

```csharp
using SevenMLabs.Astrology;

var result = Engine.Calculate(new {
    operation = "natalDomains",
    utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
    location = new { latitude = 10.8231, longitude = 106.6297 },
    domains = new[] { "relationships" }
});
Console.WriteLine(result.GetProperty("data").GetProperty("domains")
    .GetProperty("relationships").GetProperty("aspects"));
```

Rust/C dùng JSON input với cùng operation qua `astro_core::calculate_json`/`astro_calculate_json`. High-level SDKs throw lỗi có code khi envelope có errors; low-level JSON API giữ nguyên error envelope để caller tự xử lý.

## Phạm vi và kiểm chứng

Gán nhà giữ phương pháp `context.houseAssignment: "eclipticLongitude"`; chưa dùng house-position 3D theo ecliptic latitude. Chỉ có 10 bodies hiện tại: không thêm Chiron, Juno, nodes, Lilith, lots hoặc fixed stars. `relationships` là nhóm facts của **một** natal, chưa đối chiếu hai người. Transit, progression, event search và luận giải vẫn ngoài operation này.

Advanced rules version `1.0` bổ sung sign-derived element/modality/polarity, house type, motion từ instantaneous longitude speed, traditional sign-only domicile/exaltation/detriment/fall cho Sun–Saturn, solar proximity hình học, distributions, chuỗi dispositor/cycles, mutual domicile receptions và năm body-only patterns: Grand Trine, T-square, Yod, Grand Cross, Kite. Không bao phủ mọi dignity system, mọi configuration hoặc mọi trường phái. Quy tắc, ngưỡng và empty/custom aspect semantics được mô tả đầy đủ trong [individual-reports.md](individual-reports.md).

Provider không thay đổi; version profile chỉ mô tả selection/rulership rules. Kiểm thử topic cần kiểm tra coverage 26/325, references hợp lệ, mở rộng occupants/rulers, endpoint ngoài nhóm, custom/disabled aspects, hai rulership modes, Whole Sign MC và parity consumer. Kết quả chạy thực và nền tảng đã kiểm tra được ghi trong [testing.md](testing.md); không suy ra độ chính xác thiên văn mới từ việc thêm các nhóm dữ liệu.

Lá số hai người dùng operation `couple` và catalog riêng; xem [contract cặp đôi](couple.md). Contract một người của tài liệu này được giữ nguyên.
