# Composite: lá số thứ ba và 10 khía cạnh

`0.7.0-alpha.1` bổ sung midpoint composite C, được dựng từ hai natal A/B. C là chart của mối quan hệ theo quy ước hình học của engine. Nó có 10 profiles, 30 sections, custom profiles, 26 points, 325 cặp quan hệ và 66 liên hệ chủ tinh nhà như cấu trúc báo cáo cá nhân. Các facts được tính lại từ vị trí và nhà của C.

SDK có hai đường gọi, cùng chạy local và dùng một implementation Rust:

- `operation: "composite"`: nhận hai birth inputs, trả hai natal nguồn và composite.
- `operation: "couple"` với `composite: {}`: trả synastry hiện có và thêm composite. Hai natal chỉ được tính một lần trong request này; bước dựng C không gọi provider.

Composite lấy trung điểm các vị trí tương ứng trong hai natal. Đây là phương pháp dựng chart bằng hình học, không có một ngày sinh của C để tính bầu trời. Astrodienst mô tả riêng phương pháp midpoint và phương pháp reference place; bản này triển khai midpoint, cùng lựa chọn Whole Sign được khai báo rõ. Xem [Astrodienst về composite](https://www.astro.com/faq/fq_fh_compo_h.htm).

## Request và defaults

```json
{
  "operation": "composite",
  "personA": {
    "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
    "location": {"latitude": 10.8231, "longitude": 106.6297},
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": {"year": 1998, "month": 6, "day": 15, "hour": 6, "minute": 30},
    "location": {"latitude": 21.0278, "longitude": 105.8342},
    "houseSystem": "placidus"
  },
  "houseMethod": "midpoint",
  "antipodalPolicy": "error",
  "rulership": "traditional",
  "aspectPreset": "extended"
}
```

Birth input có cùng validation với [natal](natal.md): Gregorian UTC 1800–2399, ngày lịch hợp lệ, tọa độ hợp lệ, Placidus/Whole Sign. Các calendar components có giá trị nguyên; raw JSON `2000.0`/`2e3` chuẩn hóa thành `2000`, tiny fractions không bị làm tròn để chấp nhận. Caller chịu trách nhiệm đổi giờ địa phương sang UTC. Unknown fields, `null` ở options và type sai trả lỗi.

| Option standalone | Mặc định | Ý nghĩa |
|---|---|---|
| `houseMethod` | `midpoint` | Cusp midpoint, hoặc `wholeSignFromMidpointAscendant` |
| `antipodalPolicy` | `error` | Hai nguồn đối đỉnh trả lỗi; `lowerLongitude` là lựa chọn tường minh |
| `domains` | Đủ 10 IDs cá nhân | Tối đa 10 unique IDs; `[]` yêu cầu custom profiles không rỗng |
| `customProfiles` | Không có | 1–8 profiles khi cung cấp, tối đa 8 sections/profile |
| `rulership` | `traditional` | Hoặc `modern`; tính chủ tinh/chuỗi từ C |
| `aspectPreset` | `extended` | Hoặc `major`; không đi cùng `aspectRules` |
| `aspectRules` | Theo preset | Tối đa 16 angles unique; `[]` tắt matches, giữ đủ geometric pairs |

Schema tại [composite-request.schema.json](../schemas/composite-request.schema.json). Calendar validity, unique profile/section IDs và unique aspect angles vẫn được core xác minh sau structural schema validation.

## Gắn vào payload cặp đôi

```js
const result = calculate({
  operation: 'couple',
  personA: birthA,
  personB: birthB,
  domains: ['communication', 'longTerm'],
  rulership: 'traditional',
  aspectPreset: 'extended',
  composite: {
    houseMethod: 'wholeSignFromMidpointAscendant',
    domains: ['career', 'love', 'finance']
  }
});
const thirdChart = result.data.composite;
console.log(thirdChart.chart, thirdChart.domains);
```

`birthA`/`birthB` là các object UTC/location/houseSystem như request trên. `domains` ở request gốc chọn **6 profiles synastry**; `composite.domains` chọn **10 profiles của C**. Hai catalogs có phạm vi riêng. `composite: {}` chọn đủ 10 profiles C, không bắt buộc gửi danh sách.

Options bên trong `couple.composite` chỉ gồm `houseMethod`, `antipodalPolicy`, `domains`, `customProfiles`. C dùng chung `rulership` và aspect rules của request cặp đôi. Không gửi lại `aspectPreset`/`aspectRules`/`rulership` ở child object. Muốn policies khác giữa synastry và composite, gọi standalone riêng.

Bỏ `composite` giữ nguyên payload `couple` của bản 0.6, ngoài `engineVersion`. Bật option thêm `data.chartCount: 3`, `data.composite` và `calculation.composite`. Nếu dựng C lỗi, toàn request trả standard error envelope với `data: null`; không trả một chart thứ ba hoặc báo cáo cặp đôi thiếu dữ liệu.

## Quy ước tính

Sun–Pluto được ghép theo cùng ID: Sun A với Sun B, Moon A với Moon B. Longitudes chuẩn hóa vào `[0, 360)`, lấy trung điểm theo cung ngắn. Ví dụ `350°` và `10°` có midpoint `0°`, không phải `180°`. Thứ tự A/B không đổi kết quả hình học.

ASC và MC được tính midpoint riêng. DSC = ASC C + 180°, IC = MC C + 180°, chuẩn hóa vào vòng zodiac. Việc dựng các trục đối diện được ghi rõ trong provenance. Engine không tự xoay Mercury/Venus thêm 180° để làm chart giống một bầu trời vật lý.

### Hai điểm đối đỉnh

Hai nguồn cách nhau 180° có hai trung điểm tương đương. Tolerance phân loại là `1e-10°`:

- `antipodalPolicy: "error"` trả `CALCULATION_FAILED` và nêu point ID.
- `lowerLongitude` chọn candidate có longitude chuẩn hóa nhỏ hơn. Đây là quy ước phần mềm tường minh, không phải một midpoint duy nhất suy ra từ dữ liệu.
- DSC/IC dựng từ trục đối diện, không tự chọn một branch riêng. Whole Sign cusps dựng từ ASC C, không cần giải ambiguity của cặp cusp nguồn.

`provenance.points[].antipodal` ghi tình trạng endpoints nguồn; `resolution` phân biệt `unambiguous`, `lowerLongitude`, `notRequiredForDerivedPoint`. Hai loại sau không được UI trình bày như cùng một thao tác.

### Dựng 12 nhà

`midpoint` tính circular midpoint giữa hai cusp có cùng số nhà. Kết quả phải đi quanh zodiac đúng một lần, không trùng cusp. Có những cặp natal làm các midpoint cusp đảo thứ tự; engine trả lỗi thay vì sửa thứ tự, đổi branch từng nhà hoặc chuyển house method im lặng. `provenance.sourceHouseSystems` giữ hệ nhà của cả A/B; C có `houseMethod` riêng, không được dán nhãn như một Placidus thiên văn mới.

`wholeSignFromMidpointAscendant` đặt cusp H1 ở đầu sign chứa ASC C; các cusps sau cách nhau 30°. MC C vẫn là midpoint MC, có thể khác cusp H10. Đây là cách dựng Whole Sign từ ASC composite; không thay house systems của natal nguồn. Birth input ở vĩ độ khiến natal Placidus thất bại vẫn phải chọn Whole Sign cho người đó trước.

Trong cả hai phương pháp, house placement dùng longitude theo cùng quy tắc boundary của core: điểm đúng trên cusp thuộc nhà bắt đầu ở cusp đó.

## Dữ liệu đầu ra

Standalone giữ standard success/error envelope. Success có:

```text
data.chartKind = compositeRelationship
data.subjectCount = 2
data.chartCount = 3
data.subjects.A/B = { id, natal, calculation }
data.composite = {
  id: C, chartKind: midpointComposite, sourceSubjectIds: [A, B],
  chart, context, domains, customDomains, profileCatalog, provenance
}
```

Trong combined `couple`, A/B vẫn có natal context đầy đủ như trước; `data.composite` có đúng cấu trúc C của standalone. Geometry nguồn và cross context được giữ nguyên. `calculation.composite` tương ứng metadata standalone.

| Thành phần C | Nội dung |
|---|---|
| `chart` | 10 placements, 4 angles, 12 houses/cusps và body aspects |
| `context` | 26 semantic points, 325 unordered relations, matched aspects, 12 house facts, 66 house-ruler links và advanced facts |
| `domains` | Views theo các builtin IDs được chọn |
| `customDomains` | Views theo custom IDs, không override builtin |
| `profileCatalog` | Đủ 10 định nghĩa normalized, cùng catalog dùng cho cá nhân |
| `provenance` | Phương pháp, policy, tolerance, hệ nhà nguồn và cách dựng từng điểm |

`chart` của C **không có UTC/location, latitude/distance**. `speed`/`isRetrograde` của placements là `null`; mọi C aspect `applying` là `null`; body state `motion` là `notApplicable`. `calculation.speedUnit` của standalone là `null`; khi bật trong `couple`, trường này nằm ở `calculation.composite.speedUnit`. Không suy Rx/applying hoặc trung bình natal speed cho C.

Advanced facts được tính lại theo C: elements/modalities/polarities, house types, dignity sign-only, geometric solar conditions, distributions, dispositor chains/cycles, mutual domicile receptions và supported aspect patterns. `advanced.rules` dùng `sevenmlabs-composite-facts`, motion source `symbolicMidpointConstruction`, distribution population `allCompositeBodies`. Dignity/solar labels giữ nghĩa hình học được công bố; không là tình trạng vật lý của một hành tinh tại thời điểm sinh C.

Metadata dùng `scope: midpoint-composite-data`, `chartKind: midpointComposite`, `construction: symbolic`, method/policies và `sourceProvider`/`sourceEphemeris`. `sourceCalculations.A/B` có TT/UT1 JD và hệ nhà của natal nguồn; không có một JD của C. Domain selection `2.0` và report structure `1.1` được dùng chung, nhưng từng report ghi `chartKind: midpointComposite`.

Schema tại [composite-response.schema.json](../schemas/composite-response.schema.json). `$ref` trong schema được resolve qua URNs của các local schema files; không tải schema từ mạng khi validate.

## 10 khía cạnh và 30 mục

Selectors giữ đúng catalog [profiles.md](profiles.md), áp dụng lên **chart C**. Tên lĩnh vực ở đây là nhóm dữ liệu phục vụ báo cáo về mối quan hệ, không phải kết luận sẵn hoặc phép cộng đặc điểm hai người.

| ID | Phạm vi nhóm dữ liệu C | Sections |
|---|---|---|
| `career` | Công việc, mục tiêu và hoạt động chung | profession, workHabits, resources |
| `love` | Tình cảm và gắn kết | romance, marriage, intimacy |
| `relationships` | Giao tiếp và các quan hệ | communication, partnerships, friendships |
| `family` | Tổ ấm và gia đình | roots, home, parenting |
| `finance` | Tài nguyên, tài chính | personalResources, sharedResources, income |
| `identity` | Bản sắc của mối quan hệ | coreIdentity, emotionalStyle, personalPresentation |
| `learning` | Học hỏi và khám phá chung | thinking, education, exploration |
| `creativity` | Sáng tạo và biểu đạt | selfExpression, creativeThinking, play |
| `innerLife` | Dữ liệu nội tâm, chuyển đổi | emotionalRoots, transformation, reflection |
| `dailyLife` | Nếp sinh hoạt | routines, selfCare, rest |

Mỗi view có nhà trọng tâm, selected bodies/angles, selection reasons, relations/aspects và report mở rộng nhà/facts hỗ trợ. Các sections chọn độc lập và hợp nhất vào report; selectors có thể vượt parent focus. `primaryContact` vẫn theo parent profile. Dữ liệu đầy đủ H1–H12 luôn nằm trong context C, ngay cả khi caller chỉ chọn một domain.

Các nhóm này dùng được làm nguồn dữ liệu cho tình cảm, công việc, bạn bè hoặc gia đình. Engine không tự quyết định loại quan hệ hay viết đoạn luận giải cho từng loại; ứng dụng tổ chức nội dung theo nhu cầu của mình.

## Custom profiles và reference joins

Custom selectors là cùng [contract cá nhân](profiles.md): IDs ASCII unique, không trùng 10 builtin IDs/prototype keys; houses 1–12, bodies Sun–Pluto, 4 angle IDs; ít nhất một selector; tối đa 8 profiles và 8 sections/profile. Các selector này chỉ chọn C, không chọn riêng A/B. `domains: []` cùng custom profiles cho custom-only output.

IDs trong `data.composite.context`, views và reports **local trong C**: `sun`, `H1`, `dispositor:sun`. Không join chúng vào natal A/B chỉ vì trùng tên. `provenance.points[].sourcePointIds`, ví dụ `A:sun`/`B:sun`, mới trỏ về endpoints của nguồn. Standalone subjects không có qualified cross context; hãy resolve prefix qua `data.subjects.<id>.natal`.

`context.aspects[].index` là index toàn cục **trong context C**, không phải index trong synastry context. Report aspects, section `aspectIndexes` và pattern indexes resolve tại đó. `report.sections` tham chiếu nhà/body/chain/reception/pattern facts theo cùng quy tắc của [báo cáo cá nhân](individual-reports.md), với chart kind/motion của composite.

Provenance có 26 entries, mỗi entry giữ `id`, `kind`, hai source IDs/longitudes, longitude C, `construction`, `antipodal`, `resolution`. Construction là `shortestArcMidpoint`, `oppositeCompositeAscendant`, `oppositeCompositeMidheaven` hoặc `wholeSignFromMidpointAscendant`. Nhãn construction phải được kiểm tra trước khi tính lại midpoint từ source endpoints.

## Gọi bằng 5 ngôn ngữ

Sau khi [cài local package](packages.md), cùng JSON request hoạt động trên mọi binding:

```js
const { calculate } = require('@7mlabs/astrology');
const result = calculate({ operation: 'composite', personA: birthA, personB: birthB });
console.log(result.data.composite.domains.career.report);
```

```python
from sevenmlabs_astrology import calculate
result = calculate({"operation": "composite", "personA": birth_a, "personB": birth_b})
print(result["data"]["composite"]["domains"]["career"]["report"])
```

```csharp
using SevenMLabs.Astrology;
var result = Engine.Calculate(new { operation = "composite", personA = birthA, personB = birthB });
Console.WriteLine(result.GetProperty("data").GetProperty("composite"));
```

```rust
let result = astro_core::calculate_json(&request_json);
println!("{result}");
```

```c
char *result = astro_calculate_json(request_json);
if (result) { puts(result); astro_free_string(result); }
```

Biến birth/request ở các snippets là input ở phần request; SDK không tự tìm người hay chuyển timezone. Rust/C raw JSON APIs trả cả success/error envelope. Object APIs Node/Python/.NET throw khi core trả errors.

Dedicated examples: [Node composite](../examples/node/composite.cjs), [Python composite](../examples/python/composite.py), [.NET --composite](../examples/dotnet/Program.cs), [Node combined](../examples/node/couple-composite.cjs), [Python combined](../examples/python/couple_composite.py), .NET `--coupleComposite`. Rust/C CLI samples nhận raw request như trước.

Samples thực: [request composite](../examples/composite-request.json), [kết quả composite](../examples/composite-result.json), [request custom](../examples/custom-composite-request.json), [kết quả custom](../examples/custom-composite-result.json), [request combined](../examples/couple-composite-request.json), [kết quả combined](../examples/couple-composite-result.json). Các result files được tạo bằng package đã cài; xem [kiểm thử](testing.md) cho môi trường và số trường hợp thực tế.

## Giới hạn và hướng giao diện

Đây là midpoint composite; Davison, reference-place composite, progressed composite và forecast trên C chưa được triển khai. Transit cá nhân và lịch sự kiện có trong operations [forecast/events](forecast.md) từ 0.8. Không tạo compatibility score hoặc narrative. Packages vẫn là local alpha, chưa publish registry; có [mẫu MCP local/frontend worker](integrations.md), còn browser WASM/frontend UI là phase riêng.

Frontend nên có chế độ natal A, natal B, synastry và C; 10 tab C cùng 30 sections; bảng provenance và source endpoints; house method/policy có nhãn rõ; Rx/applying/speed của C hiển thị không áp dụng. Inspector và export ghi đầy đủ options/scope, giữ index spaces riêng của natal/synastry/C. Native package hiện không chạy trực tiếp trong browser; xem [frontend plan](frontend.md).
