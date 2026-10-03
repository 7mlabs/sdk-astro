# Catalog lĩnh vực và profiles tùy chỉnh

`natalDomains` trong `0.9.0-alpha.1` tổ chức dữ liệu **một natal cá nhân** bằng selection profile `sevenmlabs-domain-selection` phiên bản `2.0`. Catalog có 10 lĩnh vực builtin; dev cũng có thể gửi `customProfiles` để tạo views theo selectors riêng. Mọi view dùng chung một natal, context, phép tính và advanced facts trong Rust core. SDK chỉ truyền request, gọi core cục bộ và trả JSON; không có công thức domain riêng trong Node.js, Python hoặc .NET.

Profiles là quy tắc tổ chức dữ liệu của dự án hoặc caller. Việc đặt tên `identity`, `marriage` hay `dailyLife` không biến các facts hình học thành kết luận tính cách, tương hợp, dự đoán hoặc chẩn đoán. Ứng dụng dựng nội dung báo cáo trên những facts và phải quản lý quy tắc diễn giải của mình.

## Catalog 10 lĩnh vực

Selectors của năm nhóm cũ được giữ nguyên. Năm nhóm bổ sung tổ chức các chủ đề cá nhân trước đây chưa có view riêng; không bổ sung thiên thể, house system hoặc kỹ thuật dự báo mới.

| ID | Chủ đề | Houses | Bodies | Angles |
|---|---|---|---|---|
| `career` | Công việc, sự nghiệp | H2, H6, H10 | Sun, Mercury, Jupiter, Saturn | MC |
| `love` | Tình cảm | H5, H7, H8 | Moon, Venus, Mars | DSC |
| `relationships` | Giao tiếp, đối tác, quan hệ xã hội | H3, H7, H11 | Moon, Mercury, Venus | ASC, DSC |
| `family` | Gốc rễ, gia đình, nhà ở, con cái | H3, H4, H5 | Sun, Moon, Saturn | IC |
| `finance` | Tài chính, nguồn lực cá nhân và chung | H2, H8, H10, H11 | Venus, Jupiter, Saturn | MC |
| `identity` | Tổng quan bản thân và biểu đạt | H1 | Sun, Moon | ASC |
| `learning` | Tư duy, học tập, khám phá | H3, H9 | Mercury, Jupiter | — |
| `creativity` | Sáng tạo, sở thích, biểu đạt | H5 | Sun, Venus, Mercury | — |
| `innerLife` | Nội tâm, gốc rễ cảm xúc, chiêm nghiệm | H4, H8, H12 | Moon, Saturn, Neptune | IC |
| `dailyLife` | Thói quen, nhịp sinh hoạt, chăm sóc bản thân | H1, H6, H12 | Sun, Moon, Mars, Saturn | ASC |

Bảng này là project policy, không phải mapping duy nhất của mọi trường phái. `identity` có Sun/Moon/ASC làm selectors; phân bố toàn natal vẫn đọc từ `context.advanced.distributions`. `dailyLife` không tính bệnh lý, tuổi thọ hoặc đưa lời khuyên y khoa.

### 30 mục báo cáo con

Mỗi builtin definition có ba sections. Trong bảng dưới, tên chỉ chủ đề tổ chức báo cáo, không là kết luận đã tính sẵn. Output giữ selector IDs chuẩn bằng chữ thường của bodies và `ascendant`/`midheaven`/`descendant`/`imumCoeli`.

| Profile | Section ID | Houses | Bodies | Angles |
|---|---|---|---|---|
| `career` | `profession` | H10 | Sun, Jupiter, Saturn | MC |
| `career` | `workHabits` | H6 | Mercury, Saturn | — |
| `career` | `resources` | H2 | Venus, Jupiter | — |
| `love` | `romance` | H5 | Moon, Venus, Mars | — |
| `love` | `marriage` | H7 | Moon, Venus, Saturn | DSC |
| `love` | `intimacy` | H8 | Venus, Mars, Pluto | — |
| `relationships` | `communication` | H3 | Moon, Mercury | — |
| `relationships` | `partnerships` | H7 | Mercury, Venus | DSC |
| `relationships` | `friendships` | H11 | Mercury, Venus, Jupiter | — |
| `family` | `roots` | H4 | Moon, Saturn | IC |
| `family` | `home` | H4 | Moon, Venus | IC |
| `family` | `parenting` | H5 | Sun, Moon, Saturn | — |
| `finance` | `personalResources` | H2 | Venus, Jupiter | — |
| `finance` | `sharedResources` | H8 | Venus, Saturn, Pluto | — |
| `finance` | `income` | H10, H11 | Venus, Jupiter, Saturn | MC |
| `identity` | `coreIdentity` | — | Sun | ASC |
| `identity` | `emotionalStyle` | — | Moon | — |
| `identity` | `personalPresentation` | H1 | Sun, Moon | ASC |
| `learning` | `thinking` | H3 | Mercury | — |
| `learning` | `education` | H9 | Mercury, Jupiter | — |
| `learning` | `exploration` | H9 | Jupiter | — |
| `creativity` | `selfExpression` | H5 | Sun, Venus | — |
| `creativity` | `creativeThinking` | H3, H5 | Mercury, Venus | — |
| `creativity` | `play` | H5 | Sun, Moon, Venus | — |
| `innerLife` | `emotionalRoots` | H4 | Moon, Saturn | IC |
| `innerLife` | `transformation` | H8 | Moon, Saturn, Pluto | — |
| `innerLife` | `reflection` | H12 | Moon, Saturn, Neptune | — |
| `dailyLife` | `routines` | H6 | Sun, Moon, Saturn | — |
| `dailyLife` | `selfCare` | H1, H6 | Sun, Moon, Mars | ASC |
| `dailyLife` | `rest` | H12 | Moon, Saturn, Neptune | — |

## Chọn builtin hoặc custom

Bỏ `domains` tạo cả 10 builtin views. Muốn giảm số phần báo cáo, truyền danh sách builtin IDs cần dùng. `customProfiles` luôn được tính khi có trong request:

| Request | Views trả về |
|---|---|
| Không có `domains` hoặc `customProfiles` | 10 builtin views |
| `domains: ["career", "identity"]` | Hai builtin views |
| `domains: ["career"]` cùng custom profiles | Một builtin và các custom views |
| `domains: []` cùng custom profiles không rỗng | Chỉ các custom views |
| `domains: []` không có custom profiles | `INVALID_INPUT` |

`data.profileCatalog` là mảng definitions đã normalize (gồm `id`, `version`, `houses`, `bodies`, `angles`, `sections`), vẫn giữ định nghĩa của cả 10 builtin profiles, kể cả khi chỉ yêu cầu một nhóm. `data.domains` chỉ chứa builtin views được chọn; `data.customDomains` chứa custom views. Mỗi view trả `origin`, `definition`, versions, selection reasons và report. Các catalog definitions/sections mô tả selectors, không phải thêm một natal hay một bộ positions mới.

## Định nghĩa custom profile

```json
{
  "operation": "natalDomains",
  "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
  "location": { "latitude": 10.8231, "longitude": 106.6297 },
  "domains": [],
  "customProfiles": [
    {
      "id": "personalGrowth",
      "version": "1.0",
      "houses": [1, 9],
      "bodies": ["sun", "moon", "mercury"],
      "angles": ["ascendant"],
      "sections": [
        {
          "id": "study",
          "houses": [3, 9],
          "bodies": ["mercury"]
        }
      ]
    }
  ]
}
```

Custom profile và section dùng cùng selectors houses/bodies/angles. Nhà được truyền bằng số, output tham chiếu nhà bằng `H1`…`H12`. Mỗi definition cần ít nhất một selector; không có ngầm định “lấy tất cả” khi ba mảng đều rỗng. Section có thể chọn selectors ngoài profile cha: ví dụ `study` ở trên chọn H3 mặc dù parent chỉ chọn H1/H9. Mọi houses/contacts/facts của section được hợp nhất vào report cha. Evidence references của section join được qua report cha hoặc context chung; không cần tính lại dữ liệu.

Profile áp dụng cùng cơ chế mở rộng như builtin: focus bodies cộng occupants/rulers của focus houses; contacts giữ cả endpoint ngoài nhóm; report mở rộng các nhà hỗ trợ bằng policy có evidence. Caller không truyền công thức, script, điểm số, ephemeris provider hoặc tập body mới trong custom profile. Chi tiết facts/report trong [individual-reports.md](individual-reports.md).

## Validation

| Trường | Quy tắc |
|---|---|
| `customProfiles` | Khi có, mảng 1–8 definitions; không nhận `[]` hoặc null |
| Profile `id` | ASCII, 1–64 ký tự, regex `^[a-z][a-zA-Z0-9_-]{0,63}$`; duy nhất trong request |
| Reserved profile IDs | Không dùng 10 builtin IDs, `constructor`, `prototype`, `__proto__` |
| Profile `version` | Bỏ trường dùng `"1.0"`; 1–32 ký tự ASCII theo `^[a-zA-Z0-9][a-zA-Z0-9._-]{0,31}$` |
| `houses` | Bỏ trường dùng `[]`; tối đa 12 số nguyên giá trị 1–12, không lặp |
| `bodies` | Bỏ trường dùng `[]`; tối đa 10 body IDs được hỗ trợ, không lặp |
| `angles` | Bỏ trường dùng `[]`; tối đa 4 angle IDs được hỗ trợ, không lặp |
| `sections` | Bỏ trường dùng `[]`; tối đa 8 definitions; section IDs duy nhất trong profile |
| Section `id` | Cùng cú pháp ASCII như profile; được dùng tên builtin; các tên `constructor`, `prototype`, `__proto__` vẫn bị từ chối |
| Section selectors | Houses/bodies/angles theo cùng quy tắc; ít nhất một selector |
| Unknown fields/null | Bị từ chối, không thay cho giá trị mặc định hoặc bỏ trường |

Bodies được hỗ trợ: `sun`, `moon`, `mercury`, `venus`, `mars`, `jupiter`, `saturn`, `uranus`, `neptune`, `pluto`. Angles: `ascendant`, `midheaven`, `descendant`, `imumCoeli`. Cú pháp IDs phân biệt chữ hoa/thường; `innerLife` và `innerlife` không là cùng một builtin ID.

Houses dùng cùng normalization số nguyên chính xác của engine: `1`, `1.0`, `1e0` đều biểu diễn nhà 1; số có phần thập phân thực, strings/bools hoặc ngoài 1–12 bị từ chối. Một profile chỉ có sections nhưng không có selector ở cấp profile vẫn không hợp lệ. Selectors không được lặp ID trong cùng một mảng. Section chỉ nhận `id`, `houses`, `bodies`, `angles`; không có trường `version` hoặc nested sections.

[Request mẫu](../examples/custom-profiles-request.json) và [kết quả thực tế](../examples/custom-profiles-result.json) dùng profile `personalGrowth` ở trên.

## Dùng cùng payload qua từng ngôn ngữ

Sau khi [cài package](packages.md), lưu request JSON trên vào `request.json`. Node.js/Python/.NET có thể đọc cùng file và truyền nguyên payload xuống core:

```js
const fs = require('node:fs');
const { calculate } = require('@7mlabs/astrology');
const request = JSON.parse(fs.readFileSync('request.json', 'utf8'));
const result = calculate(request);
console.log(result.data.customDomains.personalGrowth.report.sections);
```

```python
import json
from pathlib import Path
from sevenmlabs_astrology import calculate

request = json.loads(Path("request.json").read_text(encoding="utf-8"))
result = calculate(request)
print(result["data"]["customDomains"]["personalGrowth"]["report"]["sections"])
```

```csharp
using System.Text.Json;
using SevenMLabs.Astrology;

var json = Engine.CalculateJson(File.ReadAllText("request.json"));
using var result = JsonDocument.Parse(json);
Console.WriteLine(result.RootElement.GetProperty("data")
    .GetProperty("customDomains").GetProperty("personalGrowth")
    .GetProperty("report").GetProperty("sections"));
```

.NET `CalculateJson` và Rust/C giữ nguyên error envelope; cần kiểm tra `errors` khi xử lý input không tin cậy. Node/Python high-level `calculate` throw lỗi khi core trả errors. Các ví dụ trên chỉ minh họa đọc JSON, không sinh lại facts trong SDK.

Rust gọi `astro_core::calculate_json(&request_json)`; C gọi `astro_calculate_json(request_json)` rồi giải phóng kết quả bằng `astro_free_string`. Source mẫu của hai ngôn ngữ đã nhận JSON qua argument, nên cùng request chạy qua chúng mà không đổi operation hoặc viết thêm module domain. Chuẩn bị build theo [packages.md](packages.md).

## Migration từ 0.4.0-alpha.1

- Mặc định `domains` tăng từ năm lên 10 nhóm. Giữ phạm vi cũ bằng `domains: ["career", "love", "relationships", "family", "finance"]`.
- Selectors của năm nhóm cũ được giữ nguyên. Metadata selection profile tăng `1.1` → `2.0` vì catalog/default contract mở rộng.
- Report structure tăng `1.0` → `1.1`; thêm `sections`, hợp nhất evidence của sections vào parent report. Vì vậy report có thể rộng hơn trước dù view trọng tâm giữ nguyên. Facts rules giữ `1.0`, provider/phạm vi thiên văn vẫn như trước.
- `data.profileCatalog`, `data.customDomains` và view `origin`/`definition` là fields mới; caller không nên giả response chỉ có các keys cũ.
- `domains: []` giờ hợp lệ khi có custom profiles không rỗng; vẫn trả lỗi nếu không chọn bất kỳ profile nào.

Full context giữ 26 points/325 pairs/12 houses cho mọi selection. Nhiều reports/sections có thể dùng lại cùng evidence và làm response lớn; chọn đúng builtin IDs và custom definitions cần dùng. Không cộng counts/aspects của các nhóm để suy ra tổng hoặc tạo điểm tốt/xấu. IDs/indexes chỉ được join trong cùng response, không tái sử dụng giữa những lần tính có options khác.

Frontend có thể đọc catalog để tạo tab/filter và form editor, còn package vẫn chạy độc lập. Forecast đã có operation riêng với cùng catalog và selectors, xem [forecast.md](forecast.md). Browser WASM, MCP và public registry release chưa được triển khai. Lá số cặp đôi có operation riêng, không thay contract profiles cá nhân.

Lá số hai người dùng operation `couple` và catalog riêng; xem [contract cặp đôi](couple.md). Contract một người của tài liệu này được giữ nguyên.
