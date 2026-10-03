# Lá số cặp đôi từ dữ liệu sinh

`operation: "couple"` trong `0.9.0-alpha.1` nhận dữ liệu sinh của hai người, tính hai natal thực bằng Swiss Ephemeris/Moshier rồi trả dữ liệu synastry để ứng dụng dựng báo cáo. Package chạy cục bộ qua cùng Rust core, không gọi server. Phạm vi là so sánh hai natal: giữ vị trí, nhà, góc chiếu, overlays và evidence; engine không trả điểm tương hợp, lời luận giải hoặc dự đoán.

`natalDomains` vẫn là một người, có 10 lĩnh vực cá nhân; `love` và `relationships` của operation đó không trở thành chart đôi. Geometry `synastry` vẫn nhận positions/cusps do caller cung cấp. `couple` là operation mới nhận hai birth inputs và tự tính hai natal.

## Request

```json
{
  "operation": "couple",
  "personA": {
    "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
    "location": { "latitude": 10.8231, "longitude": 106.6297 },
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": { "year": 1998, "month": 6, "day": 15, "hour": 6, "minute": 30 },
    "location": { "latitude": 21.0278, "longitude": 105.8342 },
    "houseSystem": "wholeSign"
  },
  "domains": ["attraction", "communication", "emotionalConnection", "longTerm", "sharedResources", "homeFamily"],
  "rulership": "traditional",
  "aspectPreset": "extended"
}
```

Mỗi subject chỉ nhận `utc`, `location`, `houseSystem`; validation giống [natal cơ bản](natal.md): UTC Gregorian 1800–2399, north/east-positive coordinates, Placidus hoặc Whole Sign. Bỏ `houseSystem` ở một người dùng Placidus cho người đó; hai người có thể chọn hệ nhà khác nhau. Caller chuyển giờ địa phương sang UTC. Không có timezone lookup, unknown-time mode hoặc tạo cusp giả.

Các options so sánh đặt ở request gốc, không đặt trong subject. Top-level `utc`, `location`, `houseSystem`, positions/cusps, IDs lạ, null hoặc field lạ trả lỗi. Xem [request schema](../schemas/couple-request.schema.json), [response schema](../schemas/couple-response.schema.json) và [request mẫu](../examples/couple-request.json) và [kết quả mẫu](../examples/couple-result.json).

| Trường | Mặc định | Validation |
|---|---|---|
| `personA`, `personB` | Bắt buộc | Hai birth inputs độc lập; không chứa operation/options lĩnh vực |
| `domains` | Cả 6 lĩnh vực | Unique builtin IDs, tối đa 6; `[]` chỉ hợp lệ khi có custom profiles |
| `customProfiles` | Không có | 1–8 definitions khi gửi; tối đa 8 sections/profile |
| `rulership` | `traditional` | `traditional` hoặc `modern`, áp dụng cho cả hai natal contexts |
| `aspectPreset` | `extended` | `major` hoặc `extended`, không kết hợp `aspectRules` |
| `aspectRules` | Theo preset | Tối đa 16 rules, unique angles 0–180°, maxOrb 0–15°; `[]` tắt matching |

Chọn domains chỉ đổi views, không bỏ natal hoặc cross context. Bỏ `domains` cùng custom profiles trả cả builtin và custom. `domains: []` cùng custom profiles không rỗng trả custom-only. Native input giới hạn 4 MiB; output có thể lớn hơn input vì chứa facts/report evidence.

## Dữ liệu đầy đủ và chiều so sánh

| Lớp dữ liệu | Phạm vi |
|---|---|
| Natal từng người | Sun–Pluto, 4 góc thực, 12 nhà, placements/speed/retrograde và metadata thời gian/địa điểm |
| Context từng người | 26 local points, 325 local relations, enriched houses/rulers/dispositors, advanced facts |
| Cross points | 52 điểm có namespace A/B, giữ local ID và người sở hữu |
| Cross relations | 26 × 26 = 676 cặp, endpoint thứ nhất thuộc A, thứ hai thuộc B |
| Cross aspects | Mọi match theo preset/custom rules, có indices/evidence; `applying: null` |
| Body overlays | 10 thiên thể A vào 12 nhà B và 10 thiên thể B vào 12 nhà A |
| House-ruler pairs | 12 nhà A × 12 nhà B = 144 quan hệ giữa các chủ tinh |
| Views | 6 builtin lĩnh vực, 18 sections; custom profiles dùng cùng context |

Nhà overlay được tính theo longitude thiên thể người gửi và cusp của người nhận; không dùng nhà natal người gửi làm nhà nhận. House assignment vẫn là `eclipticLongitude`, chưa phải house position 3D theo body latitude. Các nhà/góc/cusps giữ semantic IDs riêng, kể cả khi trùng longitude; Whole Sign MC không thay bằng H10 cusp.

Cross aspect luôn có `applying: null`: hai natal tại hai thời điểm khác nhau không mô tả chuyển động của cùng bầu trời. Các aspects nội bộ trong từng natal/context vẫn có applying hoặc null theo contract cá nhân. `aspectRules: []` giữ đủ cross pairs và overlays nhưng matching aspects rỗng; không diễn giải mảng rỗng thành không có quan hệ hoặc mọi kỹ thuật đều đã được tính.

## Lĩnh vực và quy tắc chọn dữ liệu

| ID | Nhóm báo cáo | Houses | Bodies | Angles |
|---|---|---|---|---|
| `attraction` | Thu hút, biểu đạt tình cảm và thân mật | H1, H5, H7, H8 | Sun, Moon, Venus, Mars, Pluto | ASC, DSC |
| `communication` | Giao tiếp, tư duy và chia sẻ | H3, H7, H9 | Moon, Mercury, Jupiter, Saturn | DSC |
| `emotionalConnection` | Kết nối cảm xúc, nhu cầu và gốc rễ | H4, H7, H8, H12 | Moon, Venus, Neptune, Pluto | IC, DSC |
| `longTerm` | Quan hệ lâu dài, cam kết và định hướng | H4, H7, H10, H11 | Sun, Venus, Jupiter, Saturn | DSC, MC |
| `sharedResources` | Nguồn lực cá nhân/chung và phối hợp | H2, H6, H8, H10 | Venus, Jupiter, Saturn, Pluto | MC |
| `homeFamily` | Tổ ấm, gia đình và con cái | H4, H5, H7 | Sun, Moon, Venus, Saturn | IC, DSC |

### 18 sections

| Profile | Section | Houses | Bodies | Angles |
|---|---|---|---|---|
| `attraction` | `chemistry` | H1, H5 | Venus, Mars | ASC |
| `attraction` | `romance` | H5, H7 | Moon, Venus | DSC |
| `attraction` | `intimacy` | H8 | Venus, Mars, Pluto | — |
| `communication` | `dialogue` | H3 | Moon, Mercury | — |
| `communication` | `understanding` | H3, H9 | Mercury, Jupiter | — |
| `communication` | `conflict` | H3, H7 | Mercury, Mars, Saturn | DSC |
| `emotionalConnection` | `emotionalNeeds` | H4 | Moon, Venus | IC |
| `emotionalConnection` | `emotionalSafety` | H4, H8 | Moon, Saturn | IC |
| `emotionalConnection` | `empathy` | H12 | Moon, Neptune | — |
| `longTerm` | `commitment` | H7 | Venus, Saturn | DSC |
| `longTerm` | `sharedDirection` | H9, H10 | Sun, Jupiter, Saturn | MC |
| `longTerm` | `resilience` | H7, H8 | Mars, Saturn, Pluto | DSC |
| `sharedResources` | `values` | H2 | Venus, Jupiter | — |
| `sharedResources` | `jointResources` | H8 | Venus, Saturn, Pluto | — |
| `sharedResources` | `practicalCooperation` | H6, H10 | Mercury, Jupiter, Saturn | MC |
| `homeFamily` | `domesticLife` | H4 | Moon, Venus | IC |
| `homeFamily` | `familyBonds` | H4 | Moon, Saturn | IC |
| `homeFamily` | `parenting` | H5 | Sun, Moon, Saturn | — |

Đây là policy tổ chức dữ liệu của dự án, không phải kết luận đã tính sẵn hoặc mapping duy nhất của mọi trường phái. Response công bố catalog và normalized definitions; caller cần đọc selectors từ catalog cùng version thay vì suy từ tên nhóm.

Selectors đối xứng: cùng houses/bodies/angles được chọn trong natal A và natal B, mỗi bên mở rộng bằng occupants và rulers của nhà bên đó. Cross contacts được chọn khi có ít nhất một endpoint được chọn; luôn giữ endpoint người còn lại dù ngoài focus. Overlays giữ chiều người gửi/người nhận; sections dùng selectors độc lập và references để facts ngoài parent focus vẫn truy cập được. Shared context giữ toàn bộ dữ liệu để join.

## Response và reference joins

```text
data.chartKind = "coupleSynastry"
data.subjectCount = 2
data.subjects.A/B = { id, natal, context, calculation }
data.context = { points, relations, aspects, overlaysAtoB, overlaysBtoA,
                 houseRulerRelations, coverage }
data.domains.<builtinId> = view
data.customDomains.<customId> = view
data.profileCatalog = 6 definitions
```

Global point IDs có dạng `A:sun`, `B:moon`, `A:ascendant`, `B:H7`, đồng thời có `chartId`/`localId`. `data.subjects.A.context.points` giữ IDs local như `sun`, `H7`; không join bằng ID local trong global context. Cross relation ID có dạng `A:sun:B:moon`, endpoint `point1` luôn A và `point2` luôn B, kể cả overlay B→A. `context.aspects[].index` unique trong cross context, khác index của aspects local; mỗi aspect có `relationId`, relations giữ `aspectIndexes`.

View giữ `origin`, normalized `definition`, `selectionRules`, `selections.A/B` và global `pointIds`, `relationIds`, `aspects`, `overlayIds`, `houseRulerRelationIds`, `report`. `selections.A/B` có `referenceScope: subjectLocalNatal`: house/body/angle/point/relation/aspect IDs bên trong selection đó phải join với context của subject tương ứng. Cross aspects giữ `selectedEndpoints`; endpoint ngoài focus vẫn có trong global context.

| Reference | Join tới |
|---|---|
| View/report qualified points | `data.context.points[].id` |
| View `relationIds`, aspect `relationId` | `data.context.relations[].id` |
| Cross aspect index | `data.context.aspects[].index` |
| `overlayIds` | Hợp hai mảng `context.overlaysAtoB/BtoA` theo `id` |
| `houseRulerRelationIds` | `context.houseRulerRelations[].id` |
| `dispositorChainRefs` `{chartId,id}` | `data.subjects[chartId].context.advanced.dispositorChains[].id` |
| `natalReceptionRefs` `{chartId,id}` | `data.subjects[chartId].context.advanced.receptions[].id` |
| `natalAspectPatternRefs` `{chartId,id}` | `data.subjects[chartId].context.advanced.aspectPatterns[].id` |

Report khai báo `chartKind: coupleSynastry`, `selectionPolicy: primaryProfileWithTraceableSupport`, `primaryPointIds`, `points`, `houses`, `bodyFacts`, `aspectFacts` và các references trên. Report `houses[].house`/`bodyFacts[].facts` vẫn dùng local natal IDs; wrapper ngoài có qualified `id`, `chartId`, `localId`. Points/houses/body facts có `primary` và `sectionIds`; cross aspect facts có `primaryContact` cùng `sectionIds`. `primaryContact` luôn theo parent selection, không đổi khi section chọn thêm contacts.

Mỗi `report.sections` là references tới facts trong report/context: `primaryPointIds`, `focusHouseIds`, `pointIds`, `houseIds`, `bodyFactIds`, `aspectIndexes`, `overlayIds`, `houseRulerRelationIds` và subject-local advanced refs. Không có full nested report lặp lại. Parent report hợp nhất sections chọn độc lập, nên sections có thể dùng facts ngoài parent focus. Coverage ghi `localAdvancedScope: subjectNatalOnly` và `sectionSelection: independentSymmetricSelectorsSharedNatals`.

Overlay ID ví dụ `A:sun->B:H7`; record giữ người gửi/nhận, source natal house, target house/cusp/ruler và related cross relation IDs/aspect indices. House-ruler relation ID ví dụ `A:H2:B:H8`, có `ruler1`/`ruler2` qualified và `relationId`. `sameBodyId: true` nghĩa hai chủ tinh có cùng local body ID, vẫn là hai vị trí của hai người và vẫn có cross relation; không bỏ cặp này như self-pair.

Support mở rộng facts bằng các endpoints của contacts, overlay sources/receiving houses/rulers, cặp chủ tinh nhà và chuỗi dispositor natal liên quan. Những facts hỗ trợ không tự chọn thêm cross contacts đệ quy. Counts report có thể lớn hơn primary view; không cộng counts từ nhiều lĩnh vực để suy ra tổng hoặc đánh giá quan hệ.

## Custom profiles

```json
{
  "id": "sharedLearning",
  "version": "1.0",
  "houses": [3, 9],
  "bodies": ["mercury", "jupiter"],
  "angles": [],
  "sections": [
    { "id": "ideas", "houses": [3], "bodies": ["mercury"] }
  ]
}
```

Tham khảo [request custom cặp đôi](../examples/custom-couple-request.json) và [kết quả custom](../examples/custom-couple-result.json). Thêm object trên vào `customProfiles` của request; dùng `domains: []` nếu chỉ cần custom view. Definition dùng cấu trúc [profiles cá nhân](profiles.md#định-nghĩa-custom-profile), nhưng selectors được áp dụng trên cả hai natal; không có selector riêng theo subject trong alpha này. Custom view nằm riêng với builtin views và không override catalog.

Profile ID là ASCII `^[a-z][a-zA-Z0-9_-]{0,63}$`, không trùng 6 builtin của `couple` hoặc 10 builtin của `natalDomains`, không trùng nhau và không dùng `constructor`, `prototype`, `__proto__`. Version mặc định `1.0`, 1–32 ký tự theo `^[a-zA-Z0-9][a-zA-Z0-9._-]{0,31}$`. Houses unique 1–12, bodies unique 10 IDs được hỗ trợ, angles unique 4 IDs; ít nhất một selector tại profile và mỗi section. Mỗi section có unique ID cùng cú pháp, không có version/nested sections. Unknown fields và explicit null bị từ chối. `1`, `1.0`, `1e0` normalize cùng nhà; fractions thực không bị làm tròn.

## Metadata và lỗi

Success/error envelope giữ `schemaVersion`, `engineVersion`, `calculation`, `data`, `warnings`, `errors`. Calculation metadata ghi `scope: couple-synastry-data`, `chartKind: coupleSynastry`, provider/ephemeris/time models và hai `subjectCalculations` gồm TT/UT1 Julian days cùng house system. Selection profile `sevenmlabs-couple-selection` và report profile `sevenmlabs-couple-report` đều version `1.0`. Mỗi `data.subjects.A/B` giữ full calculation riêng; UTC/location nằm trong natal của người đó. Không dùng một metadata natal để đại diện hai epochs/địa điểm khác nhau.

Sai request/birth/options trả `INVALID_INPUT`; provider house failure trả `CALCULATION_FAILED`, không âm thầm đổi hệ nhà. Response lỗi có `data: null`; không trả report cặp đôi một phần khi một người không tính được. Node/Python/.NET high-level API throw lỗi có code; raw-JSON API, Rust và C giữ error envelope để caller kiểm tra.

## Gọi từ năm ngôn ngữ

Lưu request thành `couple-request.json`, cài package theo [packages.md](packages.md). SDK chỉ gọi local core; không cần service phụ.

```js
const fs = require('node:fs');
const { calculate } = require('@7mlabs/astrology');
const request = JSON.parse(fs.readFileSync('couple-request.json', 'utf8'));
const result = calculate(request);
console.log(result.data);
```

```python
import json
from pathlib import Path
from sevenmlabs_astrology import calculate

request = json.loads(Path('couple-request.json').read_text(encoding='utf-8'))
result = calculate(request)
print(result['data'])
```

```csharp
using System.Text.Json;
using SevenMLabs.Astrology;

using var result = JsonDocument.Parse(
    Engine.CalculateJson(File.ReadAllText("couple-request.json")));
if (result.RootElement.GetProperty("errors").GetArrayLength() != 0)
    throw new Exception(result.RootElement.GetProperty("errors").GetRawText());
Console.WriteLine(result.RootElement.GetProperty("data"));
```

```rust
fn main() {
    let input = std::fs::read_to_string("couple-request.json").unwrap();
    println!("{}", astro_core::calculate_json(&input));
}
```

```c
#include "astro_engine.h"
#include <stdio.h>
int main(int argc, char **argv) {
    if (argc != 2 || astro_abi_version() != 1) return 1;
    char *result = astro_calculate_json(argv[1]);
    if (!result) return 2;
    puts(result);
    astro_free_string(result);
    return 0;
}
```

Rust/C examples trả envelope gồm lỗi; application phải parse `errors` trước khi dùng data. C input phải UTF-8 NUL-terminated hợp lệ, không free output bằng C `free`. Các source samples [Node](../examples/node/couple.cjs), [Python](../examples/python/couple.py), [.NET `--couple`](../examples/dotnet/Program.cs), [Rust](../examples/rust/src/main.rs), [C](../examples/c/main.c) cùng nhận contract này. Rust/C sample sẵn có nhận JSON qua argument.

## Composite tùy chọn

Từ `0.7.0-alpha.1`, thêm `composite: {}` ở request root để dựng chart C từ hai natal đã tính. Child nhận `houseMethod`, `antipodalPolicy`, `domains`, `customProfiles`; rulership và aspect policy dùng cùng root. Child domains là 10 IDs giống catalog cá nhân; root domains vẫn là 6 IDs cặp đôi. C trả ở `data.composite`, metadata ở `calculation.composite`, `data.chartCount: 3`. Không gửi child thì payload cặp đôi giữ nguyên shape cũ.

C có nhà/góc chiếu/chủ tinh/advanced facts tính lại từ midpoint, 10 lĩnh vực/30 sections và provenance riêng; không có ngày sinh, địa điểm hay tốc độ vật lý giả. Nếu không dựng được midpoint/houses, toàn request trả error envelope `data: null`. Xem [composite.md](composite.md) và [request kết hợp](../examples/couple-composite-request.json) cho contract đầy đủ.

## Migration và giới hạn

Từ `0.5.0-alpha.1`, chuyển nhu cầu so sánh hai người sang operation mới `couple`. Không thêm `personB` vào `natalDomains`; request đó vẫn bị từ chối. IDs lĩnh vực cặp đôi khác IDs cá nhân và namespace references khác local natal IDs; luôn join trong cùng response, không giữ indices giữa các request hoặc cộng contact counts từ nhiều views.

Provider/phạm vi thiên văn giữ Sun–Pluto, tropical geocentric, Gregorian UTC 1800–2399, Placidus/Whole Sign. Chưa có Chiron/Juno/nodes/Lilith/lots/fixed stars, sidereal, declination contacts hoặc mixed-person aspect patterns. Aspect patterns/advanced facts mỗi người vẫn thuộc natal người đó; không gọi chúng là cấu hình chung của cặp đôi.

Midpoint composite C đã có, xem [composite.md](composite.md). Chưa có forecasting cho cặp đôi, Davison, progression, return charts hoặc narrative/scoring. Transit và exact events cho cá nhân đã có qua [forecast.md](forecast.md). Package chưa publish registry; đã có [mẫu MCP local/frontend worker](integrations.md), còn frontend UI/browser WASM là phase riêng. Xem [frontend cặp đôi](frontend.md) và [kết quả kiểm thử thực](testing.md). Parity giữa bindings kiểm tra serialization/integration; độ chính xác natal đến từ provider references, không từ tên lĩnh vực.

## Kiểm tra contract khi phát triển

```bash
node scripts/check-couple-schema.cjs examples/couple-result.json examples/custom-couple-result.json
# Trong consumer đã cài package, dùng TypeScript compiler của môi trường build:
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs couple-types.ts
```

Schema validator dùng Ajv2020 strict, đăng ký schemas bằng URNs từ file local, không tải schema online. Có thể chỉ định module build bằng `AJV_2020_MODULE`. Kiểm tra cấu trúc được bổ sung bằng semantic joins: namespace/subject/direction, aspect/overlay/ruler IDs, section closure và advanced refs về đúng natal. Validator là dev tool, không là dependency runtime của package. Source TypeScript check nằm trong [couple-types.ts](../examples/node/couple-types.ts); kết quả chạy thực nằm trong [testing.md](testing.md).
