# Natal cá nhân phục vụ báo cáo chuyên sâu

`operation: "natalDomains"` nhận dữ liệu sinh của **một người** và trả một natal cùng dữ liệu theo 10 lĩnh vực cá nhân: `career`, `love`, `relationships`, `family`, `finance`, `identity`, `learning`, `creativity`, `innerLife`, `dailyLife`; hoặc theo `customProfiles` được dev khai báo. Tất cả các lĩnh vực dùng cùng lá số. `love` là dữ liệu natal liên quan tình cảm của cá nhân; `relationships` là dữ liệu natal liên quan cách cá nhân giao tiếp và tham gia quan hệ. Hai nhóm này không phải compatibility, synastry hoặc composite, và không nhận birth input của người thứ hai.

Mục tiêu là để ứng dụng dựng báo cáo chuyên sâu từ facts có thể kiểm tra lại: positions, đủ H1–H12, occupants/rulers, conditions, chuỗi chủ tinh, quan hệ góc và configurations được hỗ trợ. Engine không trả điểm tốt/xấu, kết luận tính cách, lời khuyên hoặc dự đoán. Ứng dụng có thể thêm templates diễn giải riêng và phải giữ nguồn/rules của phần đó tách khỏi facts của engine.

“Chuyên sâu” trong tài liệu này là phạm vi tính được công bố bên dưới. Không có cam kết bao phủ mọi kỹ thuật hoặc trường phái chiêm tinh; metadata và giới hạn cho biết chính xác phạm vi mỗi response.

## Dữ liệu gốc giữ đầy đủ

| Dữ liệu | Phạm vi |
|---|---|
| Natal | Một người, UTC Gregorian 1800–2399, tropical geocentric apparent |
| Thiên thể | Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto |
| Trục | ASC, MC, DSC, IC thực |
| Nhà | H1–H12, Placidus hoặc Whole Sign, cusp/sign, occupants và ruler placements |
| Quan hệ góc | 26 semantic points, đủ 325 cặp; signed delta và separation cả khi không match aspect |
| Aspects | Mọi match theo preset hoặc custom rules của request; orb, maxOrb, applying khi biết tốc độ |

`data.natal` và `data.context` là dữ liệu chung để join tất cả các phần báo cáo. Chọn riêng một lĩnh vực không bỏ các nhà hoặc thiên thể ngoài lĩnh vực khỏi context. House placement vẫn theo longitude ecliptic, không phải house position 3D theo latitude. Trong Whole Sign, actual MC và H10 cusp là hai điểm riêng.

Các góc/cusps chưa có tốc độ nên `applying` cho contacts có chúng là null. Null là chưa xác định, không phải false. Hai semantic points có thể trùng longitude; ASC/H1 hoặc MC/H10 không được đếm như hai thiên thể độc lập khi dựng báo cáo.

Chi tiết input, presets, sign ruler table và selection profile nằm trong [domains.md](domains.md). Các dữ liệu dẫn xuất bên dưới dùng cùng positions, nhà và aspect rules; không gọi thêm server hoặc dùng lại scoring legacy.

## Contract response

Package `0.9.0-alpha.1` dùng report structure version `1.1`, facts rules version `1.0` và domain selection profile `2.0`:

```text
data.chartKind = "individualNatal"
data.subjectCount = 1
data.natal
data.context
  ├─ houses                     # đủ H1–H12
  ├─ houseRulerRelations        # đủ 66 cặp nhà
  ├─ points / relations / aspects
  └─ advanced
      ├─ rules
      ├─ bodyStates
      ├─ distributions
      ├─ dispositorChains
      ├─ receptions
      └─ aspectPatterns
data.profileCatalog                     # định nghĩa 10 builtin profiles
data.domains.<id> / data.customDomains.<id>
  ├─ origin / definition                # nguồn và selectors của profile
  ├─ houses / bodies / angles / aspects # view trọng tâm
  └─ report                             # dữ liệu mở rộng để dựng báo cáo
      └─ sections                       # mục con cùng evidence references
```

`houseRulerRelations` liên kết mỗi cặp nhà bằng `house1`, `house2`, `ruler1`, `ruler2`, `sameRuler`, `relationId`, `aspectIndexes`. Nếu hai nhà cùng ruler, `sameRuler: true`, `relationId: null` và `aspectIndexes: []`; không tạo self-aspect giả. Khi rulers khác nhau, `relationId` join sang `context.relations` và indexes join sang `context.aspects`. Vì vậy nhà không có occupants vẫn có ruler placement/contacts để đọc.

Mỗi `data.domains.<id>.report` và `data.customDomains.<id>.report` có:

| Field | Nội dung |
|---|---|
| `version`, `chartKind` | `"1.1"`, `"individualNatal"` |
| `selectionPolicy` | `"primaryProfileWithTraceableSupport"` |
| `focusHouseIds` | Nhà trọng tâm theo profile |
| `houseIds`, `houses` | Nhà trọng tâm và nhà hỗ trợ cùng evidence lựa chọn |
| `pointIds`, `points` | Điểm cần đọc cho report, gồm cả hai endpoint của mọi report aspect |
| `aspects` | Global aspect facts, thêm `primaryContact` và `relatedHouseIds` |
| `bodyFacts` | Các body states của thiên thể trong report |
| `dispositorChains` | Chuỗi cho các thiên thể trong report; tra nodes bằng context chung |
| `receptions`, `aspectPatterns` | Facts có ít nhất một primary body trong lĩnh vực |
| `sections` | Mục báo cáo con với selectors và tham chiếu evidence trong cùng response |
| `coverage` | Counts và quy tắc mở rộng/support thực tế |

`report.houses[]` giữ toàn bộ house fact: cusp/sign, occupants, ruler id và placement. Nó thêm `roles`, `evidence`, `aspectIndexes`, `rulerPlacementHouseId`, `rulerDispositorChainId`. Các indexes chỉ mọi contact trên cusp, occupants hoặc ruler của nhà; chain ID join sang `context.advanced.dispositorChains` hoặc `report.dispositorChains`. `rulerPlacementHouseId` join sang `context.houses`: nhà đích có thể ngoài `report.houses` vì support expansion hữu hạn. Một nhà có thể có nhiều roles và nhiều bằng chứng.

### Mở rộng nhà hỗ trợ có giới hạn

Từ view trọng tâm, report chọn các nhà bằng:

1. Nhà trọng tâm của profile.
2. Nhà đặt các primary bodies và nhà do chúng làm ruler.
3. Nhà đặt mỗi body trong toàn bộ dispositor paths của primary bodies và nhà do những bodies đó làm ruler.
4. Nhà đặt focus angle thực; quan trọng khi Whole Sign MC khác H10 cusp.
5. Với mỗi primary aspect: nhà đặt các endpoints và nhà do endpoint bodies làm ruler; cusp endpoint chọn chính nhà của cusp.

Sau khi chọn nhà, report giữ mọi aspect có contact với **cusp, occupants hoặc ruler** của từng nhà đã chọn và thêm đầy đủ endpoints vào report points. `primaryContact` cho biết contact đã thuộc view trọng tâm hay được thêm để giải thích nhà hỗ trợ. Việc thêm contact này **không chọn tiếp nhà mới một cách đệ quy**; `coverage.recursiveHouseExpansion` là false. Quy tắc hữu hạn giúp lý do chọn dữ liệu vẫn truy vết được.

View trọng tâm `domain.aspects` giữ selection cũ; để dựng báo cáo sâu, đọc `domain.report` và context chung. `report.houses` không mặc định luôn có 12 nhà: nó có các nhà liên quan theo policy, còn `context.houses` luôn đủ H1–H12. Bộ dựng báo cáo có thể thêm tổng quan 12 nhà từ context.

## Mục báo cáo con

Mỗi builtin profile có ba sections; catalog/selector definitions đầy đủ ở [profiles.md](profiles.md). Ví dụ `love` có `romance`, `marriage`, `intimacy`; `family` có `roots`, `home`, `parenting`. Đây vẫn là facts của cùng một người: không có chart đối tác hoặc con cái được tính từ một natal.

Report `1.1` tổ chức sections bằng tham chiếu compact, không lồng một full report mới vào mỗi section:

| Section field | Cách đọc |
|---|---|
| `id` | ID duy nhất trong profile; tương ứng `definition.sections[].id` |
| `selectionRules` | Selectors và chính sách chọn contacts của section |
| `primaryPointIds` | Những points trọng tâm sau khi mở rộng occupants/rulers |
| `focusHouseIds`, `relatedHouseIds` | Nhà selectors và toàn bộ nhà hỗ trợ của section |
| `aspectIndexes` | Global indexes của contacts, join sang `report.aspects` hoặc `context.aspects` |
| `bodyFactIds` | Body IDs để join `report.bodyFacts` hoặc `context.advanced.bodyStates` |
| `dispositorChainIds` | Chain IDs để join `report.dispositorChains` hoặc advanced context |
| `receptionIds`, `aspectPatternIds` | IDs tương ứng trong report hoặc advanced context |

Mỗi section chọn và mở rộng độc lập theo cùng policy support hữu hạn, cùng natal/rulership/aspect rules. Selectors của section có thể vượt ngoài profile cha: `creativity.creativeThinking` thêm H3; `love.intimacy` thêm Pluto. Report cha hợp nhất houses/contacts/facts từ selection cha và mọi section để references giải được, giữ đầy đủ endpoints và nodes của chain/configuration.

`report.aspects[].sectionIds` chỉ những sections giữ contact đó. `primaryContact` vẫn cho biết contact thuộc **selection của parent**, không đổi sang true chỉ vì một section chọn nó. `relatedHouseIds` giữ hợp của các nhà hỗ trợ có contact với fact. Houses có thể thêm roles `sectionFocus`/`sectionSupport`; evidence có `sectionId` khi phát sinh từ section. Một house/aspect có thể thuộc nhiều sections nhưng chỉ có một record trong parent report.

`report.coverage.sectionCount` ghi số sections và `sectionSelection: "independentSelectorsSharedNatal"` công bố policy. Parent report có thể rộng hơn bản `1.0` dù selectors parent không đổi, vì nay sections tham gia hợp nhất. View trọng tâm `domain.houses/bodies/aspects` vẫn dùng selectors parent; đọc `report.sections` và report chung để dựng các mục nhỏ.

Ví dụ join facts cho section `marriage`, sau khi có `result` từ `calculate`:

```js
const report = result.data.domains.love.report;
const section = report.sections.find(section => section.id === 'marriage');
const houses = new Map(report.houses.map(house => [house.id, house]));
const aspects = new Map(report.aspects.map(aspect => [aspect.index, aspect]));
const bodyFacts = new Map(report.bodyFacts.map(fact => [fact.bodyId, fact]));
console.log(section.relatedHouseIds.map(id => houses.get(id)));
console.log(section.aspectIndexes.map(index => aspects.get(index)));
console.log(section.bodyFactIds.map(id => bodyFacts.get(id)));
```

Bộ dựng báo cáo nên dùng mỗi section làm outline, tra houses/rulers/occupants, placements/conditions/chains, aspects/patterns có trong references, rồi ghi metadata/options/coverage. Sections không cung cấp lời diễn giải, điểm số hoặc xác nhận một sự kiện xảy ra.

## Facts nâng cao và quy tắc

Rules được công bố bằng `sevenmlabs-natal-facts` version `1.0`. Những trạng thái này là phân loại theo quy tắc phần mềm; tên truyền thống không phải bằng chứng dự đoán hay khẳng định thực nghiệm về cá nhân.

### Body states

Mỗi body có element, modality và polarity lấy từ sign; house type lấy từ house number:

| House type | Nhà |
|---|---|
| `angular` | H1, H4, H7, H10 |
| `succedent` | H2, H5, H8, H11 |
| `cadent` | H3, H6, H9, H12 |

`motion` lấy từ longitude speed tức thời đã tính: speed < 0 là `retrograde`, > 0 là `direct`, đúng 0 là `stationary`. Không có epsilon và không tìm/predict thời điểm station. Dignity dùng bảng **traditional**, theo sign, cho bảy thiên thể Sun–Saturn: domicile, detriment, exaltation, fall. Đây không phải essential dignity tổng hợp; không có triplicity, bounds/terms, faces/decans, degree of exaltation hoặc điểm dignity. Uranus/Neptune/Pluto có `supported: false` và các flags null. Việc request chọn modern rulership không đổi bảng dignity traditional.

Solar condition dùng **khoảng cách longitude ecliptic ngắn nhất** đến Sun, tính bằng độ:

| Condition | Separation |
|---|---|
| `cazimi` | ≤ 17/60° |
| `combust` | > 17/60° và ≤ 8.5° |
| `underSunbeams` | > 8.5° và ≤ 17° |
| `free` | > 17° |
| `notApplicable` | Sun; separation null |

Các ngưỡng có tolerance floating point 1e-12° ở biên. Đây là một convention công khai, không phải angular distance 3D trên thiên cầu và không bao phủ mọi cách định nghĩa solar condition. Mỗi body trả separation cùng condition để ứng dụng kiểm tra được phân loại.

### Distributions

Phân bố elements, modalities, polarities và house types trả các nhóm `{id, count, bodyIds}`. Population là toàn bộ 10 natal bodies, mỗi body một đơn vị; giữ cả nhóm có count 0. Counts là số thiên thể, không phải trọng số; angles/cusps không được cộng vào body distribution. Tổng counts trong mỗi loại là 10; ứng dụng dùng denominator này khi tính tỷ lệ. Nếu nhiều nhóm có cùng count, ứng dụng phải giữ ties thay vì tự chọn một nhóm làm dominant. Không đổi tỷ lệ phân bố thành mức độ thuận lợi cho lĩnh vực.

### Dispositor chains và receptions

Một chuỗi bắt đầu từ body, đi theo ruler của sign tại vị trí body, sử dụng đúng `rulership` được caller chọn. `path` giữ mỗi body một lần. Engine dừng khi gặp self-ruler hoặc body đã đi qua:

- `termination: "selfRuler"`: `terminalBodyId` chỉ body tự làm ruler của sign mình; `cycleBodyIds` rỗng.
- `termination: "cycle"`: `terminalBodyId` null; `cycleBodyIds` chỉ các bodies thuộc chu kỳ, không gồm phần đường dẫn trước chu kỳ.

Self-ruler được xem là điểm kết thúc, không báo thành chu kỳ dài một body. Các paths có thể hội tụ về cùng một terminal hoặc cùng một cycle. Không suy ra một “final dispositor duy nhất” cho toàn natal khi cấu trúc có nhiều terminals/cycles.

Mutual domicile reception chỉ xuất hiện khi hai bodies là dispositor trực tiếp của nhau theo bảng rulership đang chọn. Record giữ `bodyIds`, `type: "mutualDomicile"` và `rulership`. Không tính reception bằng exaltation, triplicity hoặc quan hệ ba body.

### Aspect patterns

Nhận diện chỉ dùng **body–body** aspects thực sự match rules của request. Angles, cusps và các điểm chưa hỗ trợ không tham gia pattern detection. Không tạo thêm orb riêng để match một pattern.

| Type | Cạnh góc bắt buộc |
|---|---|
| `grandTrine` | Ba bodies, ba góc 120° |
| `tSquare` | Ba bodies: một opposition 180° và hai squares 90° |
| `yod` | Ba bodies: một sextile 60° và hai quincunxes 150° |
| `grandCross` | Bốn bodies: bốn squares 90° và hai oppositions 180° |
| `kite` | Grand Trine cộng một body có opposition tới một đỉnh và sextiles tới hai đỉnh còn lại |

`bodyIds` và `aspectIndexes` là evidence của pattern, không phải lời luận giải. `apexBodyId` là đỉnh của T-square/Yod, body thứ tư ngoài Grand Trine của Kite, và null với Grand Trine/Grand Cross. Engine giữ một edge canonical theo pair và angle, chọn global index thấp nhất nếu gặp duplicate. ID pattern duy nhất theo type, body IDs và apex; các aspect match bổ sung không loại bỏ pattern.

Các góc trong cột trên là `angle` của configured rule, không yêu cầu separation thực exact: separation được match bằng orb của rule đó. Custom angle `120.0001` không thay thế rule 120° để nhận diện Grand Trine. Khi custom rules không có các góc cần thiết, pattern đó không thể được tìm thấy; `aspectRules: []` trả không có patterns dù raw relations vẫn đủ 325 cặp. Một pattern rỗng có nghĩa không tìm thấy trong phạm vi/rules hiện tại, không có nghĩa mọi configuration khác đã được kiểm tra.

## Đọc dữ liệu để dựng báo cáo

Mọi builtin và custom profile mở rộng focus bodies bằng occupants và rulers của focus houses. Giữ mọi contact có ít nhất một selected endpoint, kể cả endpoint ngoài nhóm. Báo cáo cần lấy thông tin cả hai đầu từ context; không bỏ endpoint ngoài nhóm chỉ vì nó không có trong danh sách body trọng tâm.

Một phần báo cáo nên có houses/cusps/rulers/occupants; body placements và conditions; ruler placements/chains; aspects với angle/orb/applying; configurations và toàn bộ bodies/aspects làm bằng chứng; options/rule versions và coverage. Đủ H1–H12 vẫn sẵn trong context để giải thích liên kết giữa nhà trọng tâm với nhà khác.

Ví dụ sau chỉ đọc payload, không tự tính lại công thức:

```js
const { calculate } = require('@7mlabs/astrology');
const result = calculate({
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['career', 'love', 'relationships', 'family', 'finance'],
  aspectPreset: 'extended',
  rulership: 'traditional'
});

const career = result.data.domains.career.report;
console.log(result.data.chartKind, result.data.subjectCount);
console.log(career.houses, career.bodyFacts, career.aspectPatterns);
console.log(result.data.domains.love.report.sections);
console.log(result.data.context.advanced.distributions);
console.log(result.data.context.houseRulerRelations);
```

Cài SDK theo [packages.md](packages.md). Output mẫu đầy đủ ở [natal-domains-result.json](../examples/natal-domains-result.json).

Profiles và custom selectors được mô tả trong [profiles.md](profiles.md). Các mục con cũng mô tả **cùng một cá nhân**; ví dụ `marriage` tổ chức facts của natal liên quan chủ đề hôn nhân, không nhận dữ liệu của người phối ngẫu. `dailyLife` tổ chức dữ liệu nhà/thói quen theo profile của dự án, không trả chẩn đoán y tế.

IDs và indexes dùng để join trong **cùng response**. Domain views có thể cùng sử dụng một fact; không cộng số aspects của nhiều lĩnh vực thành tổng số contact của natal. Custom aspect rules có thể match nhiều góc cho cùng cặp nên cần giữ đúng `aspectIndexes`, không giả mỗi pair luôn chỉ có một aspect.

## Giới hạn cần thể hiện trong báo cáo

- Chỉ 10 bodies và 2 house systems hiện tại; chưa có Nodes, Chiron, Juno, Lilith, Lots, asteroids hoặc fixed stars.
- Chưa có declination parallels/contra-parallels, house position 3D, sidereal, topocentric hoặc các bảng dignity khác.
- Body-only patterns chỉ bao phủ năm loại được liệt kê, không toàn bộ configurations.
- Không có second-person chart, synastry/composite, transit, progression, returns hoặc tìm thời điểm sự kiện trong operation này.
- Input cần giờ UTC/tọa độ hợp lệ. Caller xử lý timezone lịch sử; chưa có chế độ bỏ giờ sinh mà vẫn trả houses/angles đáng tin cậy.
- Các facts dẫn xuất không mở rộng kiểm chứng thiên văn của provider; phạm vi/tolerance provider được ghi riêng trong [testing.md](testing.md).

Frontend đề xuất trong [frontend.md](frontend.md) dùng cùng payload để hiện bảng, evidence navigation, JSON export và templates báo cáo cá nhân. Browser runtime/WASM vẫn cần kiểm chứng trước khi công bố natal chạy trực tiếp trong browser.

Lá số hai người dùng operation `couple` và catalog riêng; xem [contract cặp đôi](couple.md). Contract một người của tài liệu này được giữ nguyên.
