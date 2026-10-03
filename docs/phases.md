# Plan chuyển đổi theo phase

Plan chuyển source Python và .NET hiện tại sang engine trung lập chạy cục bộ. Mỗi phase có tiêu chí hoàn thành; chỉ phát hành phạm vi đã đạt tiêu chí. Việc một package cài được chưa chứng minh độ chính xác của ephemeris.

## Phase 0 Chốt phạm vi và contract

**Đầu ra:** architecture decision, contract input/output, quy ước đơn vị, danh mục hỗ trợ và mapping source legacy.

- Loại scoring/interpretation khỏi phạm vi engine.
- Chốt UTF-8 JSON, longitude `[0,360)`, speed degrees/day, orb degrees, null cho dữ liệu không biết.
- Phân biệt positions-input với birth-input; chưa có provider thì không nhận natal theo ngày sinh.
- Lưu baseline source gốc, test hiện có và các lỗi đã biết.

**Gate:** mọi feature có định nghĩa toán học và ví dụ input/output; schema/binding dùng cùng tên trường. Trạng thái: contract geometry và quyết định kiến trúc đã có; contract natal UTC/tọa độ và provider metadata đã có.

## Phase 1 Lõi geometry và mẫu package

**Đầu ra:** Rust core, C ABI, Node/Python/.NET package, Rust/C samples, script build/test và docs cài đặt.

- Chuẩn hóa vị trí, sign, house placement từ cusps được cung cấp.
- Tính aspects, applying khi biết tốc độ, midpoints, harmonics và synastry overlays.
- Cài tgz/wheel/nupkg vào consumer riêng; không import source của binding trực tiếp.
- Test lỗi, UTF-8, repeated/concurrent calls và parity mọi binding.

**Gate:** unit tests qua; mọi binding trả cùng payload cho fixtures; fresh consumer chạy offline không có localhost/HTTP. Trạng thái: source mẫu đã triển khai; kết quả thực chạy ghi trong `testing.md`. Đây là alpha geometry, chưa là full astrology engine.

## Phase 2 Ephemeris và natal cơ bản

**Đầu ra hiện có:** Swiss Ephemeris native 2.10.03/Moshier, UTC/TT/UT1 conversion, 10 bodies, Placidus/Whole Sign, ASC/MC/DSC/IC, signs/house placements và major aspects. Source provider được pin và notices nằm trong cả ba package. Không cần ephemeris data pack ở mode tích hợp này.

- Input UTC Gregorian 1800–2399; caller xử lý timezone trước khi gọi.
- Tropical geocentric apparent coordinates; không thêm sidereal/topocentric/asteroids.
- Placidus polar failure trả lỗi, không âm thầm đổi nhà.
- Kiểm tra calendar, tọa độ 0, UTC/time sensitivity, endpoints, provider concurrency và install/parity.
- References: PySwissEph cho positions/speed/houses/time; Skyfield/JPL DE421 cho vị trí độc lập ở cùng TT epoch. Tolerance và kết quả trong `testing.md`.

**Gate natal alpha:** tính thật, không runtime network, các tests natal cơ bản trên host qua. **Gate full release chưa đạt:** license toàn sản phẩm, OS/arch matrix, lịch sử timezone và coverage thiên văn rộng. Phần mở rộng lĩnh vực nằm trong phase 2 tiếp theo; calendar/transit events đã được triển khai ở phase 3 đầu tiên bên dưới. MCP đã có mẫu stdio local ở 0.9; frontend UI/WASM và public release vẫn là backlog.

## Phase 2 tiếp theo Payload theo lĩnh vực

**Đầu ra:** operation `natalDomains`, full natal/context và views `career`, `love`, `relationships`, `family`, `finance`, trong `0.3.0-alpha.1`.

- Công bố selection profile `1.0`, traditional/modern rulership tables và preset major/extended hoặc custom aspect rules.
- Giữ 10 bodies, 4 góc và 12 cusps H1–H12 thành 26 semantic points; trả đủ 325 pair relations và mọi aspect match.
- Mỗi lĩnh vực chọn focus bodies/angles/houses, mở rộng occupants/rulers và giữ contacts có ít nhất một selected endpoint.
- Trả selection reasons, references/indexes và metadata coverage. Không port scoring hoặc interpretation strings của legacy.
- Giữ operation `natal` cơ bản và geometry APIs tương thích; Whole Sign dùng actual MC riêng H10 cusp; unknown angle/cusp speeds giữ null.
- Mọi nhóm trong operation này mô tả **một cá nhân**. `love` mô tả natal của người đó; `relationships` mô tả dữ liệu quan hệ của người đó, không so sánh hai lá số.

**Gate:** validation options và schema đồng bộ; invariants 26/325; references hợp lệ; custom/disabled aspects; rulership/occupant expansion; endpoint ngoài nhóm; Whole Sign MC; fresh package install/parity các ngôn ngữ. Trạng thái thực nghiệm xem `testing.md`. Độ chính xác provider không được suy ra chỉ từ các tests selection.

## Phase 2 tiếp theo Natal cá nhân phục vụ báo cáo chuyên sâu

**Đầu ra `0.4.0-alpha.1`:** một birth input → một natal → dữ liệu nâng cao chung → các phần báo cáo `career`, `love`, `relationships`, `family`, `finance`; report/facts rules version `1.0`, selection profile `1.1`. Xem [contract báo cáo cá nhân](individual-reports.md). Không thêm chart đôi hoặc dự báo theo thời gian trong phase này.

- Giữ đủ H1–H12, 10 bodies, 4 angles, 325 quan hệ và mọi aspect match theo rules; lĩnh vực cần tham chiếu lại context đầy đủ.
- Bổ sung derived facts về element/modality/polarity, house type, motion, traditional dignity và solar proximity có định nghĩa/version công khai.
- Trả thống kê phân bố cùng counts, denominator và ties; không biến số đếm thành điểm tốt/xấu.
- Tính chuỗi dispositor, điểm kết thúc hoặc chu kỳ, mutual domicile reception theo rulership mà caller chọn.
- Nhận diện cấu hình body-only được hỗ trợ: Grand Trine, T-square, Yod, Grand Cross và Kite; trả bodies/aspects tham gia làm bằng chứng.
- Domain reports tổ chức houses/rulers/occupants, placements/conditions, ruler chains, aspects và configurations. Tham chiếu hợp lệ và không mất endpoint ngoài nhóm.
- Công bố coverage/limitations để bộ dựng báo cáo biết kỹ thuật nào được tính và kỹ thuật nào chưa có; `[]` nghĩa không tìm thấy trong phạm vi tính, không có nghĩa mọi trường phái đã được kiểm tra.

**Gate:** fixtures kiểm chứng dignity/solar thresholds/distribution ties, self-loop/cycle/chain termination, từng cấu hình được hỗ trợ, aspect rules disabled/custom và evidence references; regression natal/domain; cài lại package và parity mọi binding. Kết quả chạy thực ghi trong `testing.md`; frontend mới là định hướng cho đến khi có browser runtime đã kiểm chứng.

## Phase 2 tiếp theo Catalog đầy đủ và profiles tùy chỉnh

**Đầu ra `0.5.0-alpha.1`:** mở rộng catalog từ năm lên 10 lĩnh vực cá nhân, selection profile `2.0`, report structure `1.1`. Xem [profiles.md](profiles.md). Facts rules và provider giữ phạm vi đã công bố; thêm profile không tự thêm phép tính thiên văn mới.

- Giữ nguyên selectors năm nhóm cũ; thêm `identity`, `learning`, `creativity`, `innerLife`, `dailyLife`.
- Bỏ `domains` mặc định trả cả 10 builtin views. Caller cần phạm vi năm nhóm cũ truyền rõ năm IDs; profile catalog vẫn công bố đủ 10 định nghĩa.
- Tổ chức các chủ đề hôn nhân, thân mật, bạn bè, nhà ở, con cái và các chủ đề nhỏ khác thành `report.sections`; mọi section vẫn dùng một natal.
- Cung cấp `customProfiles`: tối đa 8 định nghĩa/request, selectors houses/bodies/angles và tối đa 8 sections/profile; không thêm công thức vào SDK.
- Tách custom views vào `data.customDomains`, giữ `data.domains` cho builtin views; output khai báo `origin`, `definition` để giải thích nguồn quy tắc.
- Sections giữ evidence references tới facts/context, bao gồm facts ngoài focus của parent khi selectors section yêu cầu.
- Cập nhật schema, TypeScript, docs, source mẫu và package versions. Chọn từng nhóm để hạn chế payload lớn do nhiều reports dùng lại cùng facts.

**Gate:** 10 catalog definitions hợp lệ; old-five selector regressions; custom-only/mixed requests; boundary/unknown/duplicate/null validation; sections có references giải được và endpoints/chain nodes đầy đủ; schema/runtime parity; fresh package install/parity năm ngôn ngữ. Kết quả thực nghiệm được ghi trong [testing.md](testing.md).

Không có paired chart, forecasting, medical diagnosis hoặc narrative/scoring trong phase này. Frontend mới vẫn là plan; public release còn phụ thuộc gates ở phase 4.

## Phase 2 tiếp theo Lá số cặp đôi

**Đầu ra `0.6.0-alpha.1`:** operation `couple` nhận UTC/location của hai người, tính hai natal độc lập rồi trả payload thô so sánh. Operation cá nhân `natalDomains` giữ nguyên. Xem [couple.md](couple.md).

- Giữ natal/full context và advanced facts từng người, có metadata và house system riêng.
- Global cross context gồm 52 qualified points, đủ 676 cross relations, matched aspects với applying null, 20 directional body overlays và 144 cross house-ruler relations.
- Catalog cặp đôi riêng có 6 lĩnh vực/18 sections; custom profiles dùng selectors đối xứng trên hai natal.
- Namespaced IDs và index references phải resolve đúng chiều A/B. Domain selection giữ contacts khi ít nhất một endpoint được chọn.
- Không có scoring/narrative, composite, Davison hoặc forecasting. Frontend tiếp tục là plan.

**Gate:** validation/schema/runtime đồng bộ, coverage và direction invariants, references của domains/sections, disabled/custom aspects, mixed houses/rulership, fresh install/parity 5 ngôn ngữ và repeated/concurrent calls. Kết quả thực nghiệm nằm trong [testing.md](testing.md).

## Phase 2 tiếp theo Lá số thứ ba

**Đầu ra `0.7.0-alpha.1`:** midpoint composite C từ hai natal, 10 lĩnh vực/30 sections, custom profiles và provenance. Có operation `composite` riêng và opt-in `couple.composite` dùng lại hai natal. Xem [composite.md](composite.md).

- Dựng bodies/ASC/MC theo circular shortest-arc midpoint; DSC/IC giữ đối đỉnh ASC/MC.
- Công bố chính sách đối đỉnh, kiểm tra midpoint cusps và lựa chọn Whole Sign rõ ràng.
- Tính lại nhà, góc chiếu, chủ tinh, derived facts, report evidence cho C; không tạo birth time/location/motion giả.
- Giữ natal A/B và output cá nhân/synastry cũ tương thích.
- Gate: midpoint invariants/swap/self charts, invalid houses, no partial success, reference closure, schema/TypeScript và fresh installed-package parity. Kết quả thực ở [testing.md](testing.md).

## Phase 3 Các module trung lập nâng cao

**Đầu ra:** transit, exact event search, retrograde stations, solar/planetary returns, Davison/reference-place composite và progression theo định nghĩa được công bố.

- Port từng thuật toán sau khi có ephemeris phase 2.
- Event search cần kiểm tra bracket/root; hỗ trợ nhiều lần đi qua do retrograde, không chỉ một ngày mẫu.
- Phân biệt timeline lấy mẫu với exact event.
- Harmonic phải tính lại mọi trường phụ thuộc hoặc loại khỏi output, không giữ nội dung natal cũ.
- Fixed stars/asteroids phải thể hiện coverage và dữ liệu thiếu.

**Gate:** fixtures thực, tolerance thời gian/tọa độ, cancellation và giới hạn scan; không có nội dung diễn giải hoặc scoring phụ thuộc ứng dụng.

## Phase 3 triển khai đầu tiên Calendar và transit theo natal

**Đầu ra `0.8.0-alpha.1`:** events day/month/year và forecast cá nhân, global eclipse maxima thật, exact transit-natal hits, house/ruler/contact evidence, 10 views/30 sections và raw messageContext. Xem [forecast.md](forecast.md).

- Provider scan session, 6h grid, relative-velocity extrema partition, circular roots và refinement; giữ repeated retrograde hits.
- Global ingress/stations/primary Moon phases/planetary aspects/solar-lunar eclipse; eclipse contacts và maximum có scope riêng.
- Fixed civil offset, UTC/TT/UT1 metadata, `[start,end)`, leap-second output và request validation trước provider.
- Một natal/context dùng chung, representative snapshot và exact event list; không có scores/narrative.
- Gate: independent dated astronomical references, root residual/bracket checks, genuine double hits, full-year budgets, per-event joins, schema/types, fresh-install parity và regression natal/couple/composite. Trạng thái kiểm thử thực tại [testing.md](testing.md). Orb-entry/exit windows, progression/returns và các phương pháp forecast khác tiếp tục là backlog.

## On-demand queries: đầu ra 0.9

Operation query có 4 groups/8 actions, custom aspect angles, natal house/point selection và grouped helpers trong Node/Python/.NET; Rust/C dùng cùng raw JSON. Có strict schemas, typed TS và references tự đủ. Mẫu MCP local/stdio và frontend host worker bridge dùng cùng package. Xem [query.md](query.md), [integrations.md](integrations.md), [testing.md](testing.md).

## Phase 4 Phát hành package đa nền tảng

**Đầu ra:** GitHub public, release tags, packages npm/PyPI/NuGet và source crate khi tên/license đã được chốt.

- Build và cài thử binary trên từng OS/arch/runtime công bố hỗ trợ.
- npm chọn binary theo platform/arch; Python wheels đúng platform; NuGet theo RID.
- Người dùng được hỗ trợ không cần compiler hoặc runtime của ngôn ngữ khác.
- Sinh checksums, dependency/data manifest, changelog và hướng dẫn migration.
- Registry install tests dùng đúng version vừa release; rollback bằng version mới hoặc thu hồi theo policy registry, không thay bytes cùng version.

**Gate:** legal/provenance resolved; CI sạch; security/dependency review đạt; install test từ registry thực qua. Repo đã chọn là `7mlabs/sdk-astro`, source mới dùng AGPL-3.0-only theo lựa chọn miễn phí của owner. Public registry release còn chờ platform/ownership gates. Xem [github-setup.md](github-setup.md).

## Phase 5 CLI và MCP cục bộ

**Đầu ra:** CLI file/stdin → JSON và MCP tools read-only gọi engine trực tiếp. Đã có mẫu MCP stdio local trong [integrations.md](integrations.md); chưa publish adapter thành package riêng hoặc tích hợp cài đặt tự động vào host.

MCP tools đề xuất: calculate chart, synastry, transits, event search, capabilities. Stdout chỉ dùng protocol; stderr chứa log. Không đưa tài khoản hoặc remote endpoint thành dependency của engine. **Gate:** protocol tests, schema parity, timeout, cancellation và xử lý request quá lớn.

## Phase 6 WASM và frontend offline

**Đầu ra:** browser build, worker adapter, data pack browser và playground tĩnh.

- Thử provider C/ephemeris với WASM trước khi hứa natal chạy browser.
- Worker thực hiện tính toán; UI dùng cùng contract, không copy công thức.
- Browser/native parity, memory usage, offline reload, CSP, mobile và accessibility tests.
- Publish demo qua static hosting hoặc GitHub Pages; không có backend tính toán.

**Gate:** browser tính được thật với đủ data; không gọi native Node addon hay fake output. Frontend hiện mới được định hướng trong `frontend.md`.

## Thứ tự thực hiện

Phase 0–1 tạo nền móng; phase 2 có natal cơ bản, 10 payload lĩnh vực cá nhân, dữ liệu chuyên sâu và custom profiles; full ephemeris coverage còn cần mở rộng kiểm chứng. Phase 2 đã có synastry và midpoint composite C; các kỹ thuật theo thời gian và các phương pháp chart đôi khác nằm ở phase 3. Phase 4 chỉ public những module đã đạt gate. CLI cơ bản có thể làm sớm nhưng MCP và frontend cần runtime/provider tương ứng. Lịch các phase tương lai phụ thuộc dữ liệu đo về port, coverage và build matrix.
