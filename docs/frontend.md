# Định hướng frontend cho engine trung lập

Frontend nên là playground kỹ thuật và công cụ khám phá dữ liệu, cùng contract với package. Engine hiện có báo cáo **một cá nhân** từ `natalDomains` và dữ liệu **hai người** từ `couple`, cùng chart C từ `composite` hoặc `couple.composite`; `events`/`forecast` đã có calendar/transit cá nhân; frontend mới chưa được triển khai. Có [mẫu host worker bridge](integrations.md) để nối UI desktop với package native. `love`/`relationships` của cá nhân đọc cùng một natal; form cặp đôi dùng hai birth inputs và catalog riêng. Engine trả facts và bằng chứng; nội dung diễn giải thuộc bộ dựng báo cáo của ứng dụng. Mục tiêu cuối là app tĩnh chạy offline bằng WASM. Native natal/couple/composite đã tính được nhưng Node addon/C library chưa chạy trong browser; provider WASM cần spike riêng.

## MVP sau khi browser runtime sẵn sàng

| Chức năng | Mục đích | Điều kiện |
|---|---|---|
| Nhập hoặc import positions JSON | Dùng ngay geometry không cần ephemeris | Core WASM geometry |
| Nhập ngày giờ, timezone, tọa độ | Tạo positions/chart từ thời gian | Provider và data WASM |
| Vòng chart và bảng placements | Xem longitude, sign, house, speed, Rx | Renderer chỉ đọc output |
| Chọn 10 lĩnh vực của cá nhân | Các tab công việc, tình cảm, quan hệ, gia đình, tài chính, bản thân, học tập, sáng tạo, nội tâm và đời sống hằng ngày | Native natalDomains; browser vẫn cần WASM |
| Form cặp đôi A/B | Hai UTC/location/house system độc lập, import/export hoặc đổi vị trí A/B | Native couple; browser vẫn cần provider WASM |
| Vòng synastry và overlays | Hai vòng chart, placements của A trong nhà B và chiều ngược lại | Đọc qualified points và directional overlays |
| Sáu tab cặp đôi | Thu hút, giao tiếp, kết nối cảm xúc, dài hạn, nguồn lực chung, tổ ấm/gia đình; 18 sections và custom profiles | Theo [couple.md](couple.md), không dựng điểm tương hợp giả |
| Chart C và 10 tab composite | Một vòng C; placements, đủ H1–H12, rulers/aspects/advanced facts và 30 sections như natal; chuyển A/B/synastry/C | Theo [composite.md](composite.md); namespace C local, không join nhầm với cross context |
| Composite options và provenance | Chọn midpoint cusps hoặc Whole Sign từ ASC C; antipodal policy, nguồn từng điểm và cảnh báo dựng chart | Không hiển thị speed/Rx/applying giả; houses invalid cần chọn phương pháp rõ ràng |
| Calendar ngày/tháng/năm | Timeline, 6 families global + exact natal hits, highlight filter, body/domain selector | Đọc [forecast.md](forecast.md), phân biệt snapshot với exact event search |
| Event drawer theo natal | Transit trong N:Hn, contacts/rulers/natal facts/evidence, precision và eclipse contacts | Indexes của snapshot/event/natal riêng; global eclipse không có local visibility |
| Daily/month/year report panels | messageContext và overview, 10 lĩnh vực/30 sections, refs sang lịch | Narrative templates thuộc ứng dụng; scan dài trong worker |
| Cross-aspect matrix | Hàng A/cột B, lọc body/angle/cusp, tách pairs thô với matches | Hiển thị applying là không xác định với cross contacts |
| Mục báo cáo con | Mở hôn nhân, thân mật, bạn bè, con cái, nhà ở và các sections của từng profile; tra evidence thay vì tính lại | Đọc `report.sections` và context |
| Profile editor | Chọn houses/bodies/angles, tạo sections và preview request; export/import định nghĩa | Cùng validation của [profiles.md](profiles.md); giới hạn 8 custom profiles/request |
| Bảng natal nâng cao | Xem conditions, distributions, dispositor chains/cycles và cấu hình góc chiếu | Chỉ render derived facts được engine hỗ trợ |
| Khung báo cáo cá nhân | Tổ chức từng phần với facts, evidence links, options và coverage | Theo [individual-reports.md](individual-reports.md); templates ứng dụng riêng |
| Query inspector theo nhóm | Normalize/separation/midpoint, custom aspect angle, chọn H1–H12/body/angle/cusp | Dùng [query.md](query.md), giữ primary/support points và indexes local |
| Aspect table và matrix | Xem angle, orb, applying, filter bodies | Cùng aspectRules với core |
| Harmonic explorer | Đổi factor và so chart gốc/biến đổi | Không giả nhà vật lý |
| JSON inspector và export | Xem payload thật, download/import, copy | Không giấu warnings/errors |
| Code examples | Copy Node/Python/.NET call tương đương | Generated từ request đang chọn |
| Data manager | Xem provider version, coverage, file thiếu | Manifest và checksum |

## Luồng sử dụng

Chọn positions hoặc birth input → chọn options được engine hỗ trợ → chạy trong worker → xem chart/table/JSON → sửa options và so kết quả → export request/response. Với `natalDomains`, một birth input tạo một hồ sơ cá nhân; chọn một hoặc nhiều lĩnh vực hoặc custom profiles chỉ đổi phần tổ chức dữ liệu, không thêm chart thứ hai. Omit `domains` tạo 10 reports; UI nên gửi đúng IDs được chọn để giảm kích thước payload. `domains: []` cùng custom profiles chỉ tạo custom views. Không dùng dữ liệu mẫu mà không dán nhãn; không tạo output giả khi thiếu provider.

Form birth cần ngày/giờ, offset hoặc timezone, lat/lon và chế độ không biết giờ sinh. Tra cứu địa điểm online là tùy chọn; nhập tọa độ thủ công phải hoạt động offline. Timezone IANA lịch sử cần database riêng; không coi UTC+7 cố định là mọi địa điểm.

Với `couple`, mỗi người có một form và metadata riêng. Chọn domain/section → mở cross aspect hoặc overlay → xem qualified endpoint A/B, vị trí natal, nhà nhận và evidence liên quan. Cung cấp tab toàn cảnh gồm hai natal, 676 cross pairs và 144 house-ruler pairs để mọi dữ liệu đều truy cập được. Đổi A/B cần tính lại request, giữ chiều overlays đúng và không chỉ đổi labels. Khi một người sai dữ liệu, hiện lỗi tại subject tương ứng; không render một report cặp đôi từ dữ liệu còn lại. Đánh dấu hai giờ sinh phải biết khi chart có angles/houses; chưa có contract unknown-time thì không tự tạo cusp giả.

Với composite, dùng cùng hai birth inputs và chọn gọi riêng hoặc bật `composite` trong `couple`. Sau khi tính, chuyển giữa natal A, natal B, synastry và C; 10 tab C dùng `data.composite.domains`, khác 6 tab synastry. Evidence/index C chỉ resolve trong context/report C. Hiển thị method, nguồn A/B của midpoint và dữ liệu motion là không áp dụng. Nếu midpoint houses lỗi, cho user chỉnh options rồi tính lại; không tự chọn Whole Sign. Export report C ghi construction/provenance và hai natal nguồn, không ghi một birth time/location giả.

## Chạy không server

Browser package chứa WASM runtime và loader data; UI được build thành static files. GitHub Pages chỉ phục vụ file, không tính chart trên server. Sau khi assets/data được tải hoặc cài đủ, offline caching cho phép mở lại theo browser policy. Việc host static demo là tùy chọn, engine package vẫn chạy độc lập.

Nếu provider chưa compile WASM, có thể tạo desktop app gọi native package. Không dựng frontend gọi localhost rồi gọi đó là browser engine offline. Tauri/Electron cũng là lựa chọn riêng, không đưa vào dependency bắt buộc của core.

## Thiết kế và kiểm thử

UI có ba vùng: input/options, chart/table, JSON/metadata. Ưu tiên bảng và biểu đồ, các control có label rõ, keyboard/focus, responsive và độ tương phản. Chart là dữ liệu vector, không là ảnh sinh AI thay cho kết quả.

Mỗi tab lĩnh vực hiển thị các sections dưới report, với danh sách selectors và evidence links. Subsection có thể dùng selectors vượt focus của profile cha; Parent report hợp nhất facts của các sections; UI cần join đúng evidence qua report/context, không suy từ danh sách primary points của cha. Mỗi tab nên có bốn lớp: nhà trọng tâm và chủ tinh; thiên thể/conditions/chuỗi chủ tinh; bảng contacts và configurations; metadata cùng giới hạn. Người đọc mở một fact sẽ thấy vị trí gốc, houses/aspects liên quan và quy tắc tính. Bảng tổng quát vẫn phải cho truy cập đủ H1–H12 để không khiến houses ngoài nhóm bị hiểu là không có dữ liệu. Các contacts body/cusp hoặc angle/cusp cần label riêng; điểm trùng longitude có thể tạo conjunction hình học và không phải hai thiên thể độc lập.

Tách export dữ liệu JSON với export báo cáo HTML/PDF. Báo cáo phải ghi provider/engine/profile versions, UTC/location, house system, rulership, aspect rules và coverage thực. Nếu ứng dụng thêm đoạn văn, ghi rõ nguồn/rules của template; không trình bày văn bản đó như output tính toán có sẵn của engine. Không tạo chế độ “không biết giờ sinh” có nhà/góc giả: khi chưa có contract phù hợp phải giải thích yêu cầu giờ sinh.

Test import/export roundtrip, malformed JSON, unknown speed, house thiếu, request cancel, data thiếu, desktop/mobile, screen reader, worker error và memory limit. So sánh output browser với native fixtures; chưa làm browser/native parity thì không công bố cùng độ chính xác.

## Phase frontend

F1: WASM geometry spike và positions playground. F2: chart/table/JSON explorer hoàn chỉnh. F3: provider WASM, data manager, birth form và các tab báo cáo cá nhân với evidence navigation/export. F3b: form A/B, synastry wheel/matrix, overlays và sáu tab cặp đôi từ contract `couple`, kiểm tra browser/native parity. F3c: vòng C, 10 tab/30 sections composite, custom profiles, provenance explorer và options nhà/đối đỉnh; kiểm tra local joins và browser/native parity. F4: static demo, offline caching, docs và UI components. F3d: calendar ngày/tháng/năm, daily messageContext, monthly/year overview, exact-event drawer và natal impact evidence từ forecast/events; worker và browser/native parity bắt buộc trước browser release. UI kit React/Vue là tùy chọn; engine không phụ thuộc framework frontend.
