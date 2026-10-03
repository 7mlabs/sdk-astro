# Kiến trúc engine chạy cục bộ

Engine hướng đến GitHub công khai và package cài vào dự án, không yêu cầu server tính toán. Quyết định kiến trúc là một lõi Rust, giao diện C ABI, Node binding qua Node-API, Python qua ctypes và .NET qua P/Invoke. Alpha đã có provider Swiss native với Moshier tích hợp: Rust gọi source C được pin, không nhúng Python. Natal, couple, composite, events/forecast và geometry cùng được phân phối trong package mỗi ngôn ngữ.

## Vì sao chọn phương án này

| Phương án | Phân phối đa ngôn ngữ | Tận dụng source hiện tại | Hạn chế |
|---|---|---|---|
| Python và .NET hiện tại | Cần đóng gói hai runtime hoặc gọi process | Cao | Phụ thuộc runtime ngoài ngôn ngữ của dev |
| Một bản engine riêng mỗi ngôn ngữ | Dễ import trong từng runtime | Trung bình | Dễ lệch công thức và tăng chi phí bảo trì |
| C hoặc C++ chung | Binding native tốt | Tận dụng provider C | Quản lý bộ nhớ và lỗi thủ công nhiều hơn |
| Rust và C ABI | Một bộ thuật toán, nhiều binding | Chuyển dần và dùng legacy đối chiếu | Cần port, build binary theo nền tảng và kiểm chứng FFI |

Rust là lựa chọn tối ưu cho mục tiêu dài hạn trong dự án này, không phải ngôn ngữ mặc định tối ưu cho mọi engine. Nó không tự tăng độ chính xác thiên văn. Độ chính xác đến từ provider, thuật toán, hệ tọa độ, time scale và dữ liệu kiểm chứng. Bản mẫu chưa chứng minh hiệu năng của toàn bộ engine hay so sánh mọi ngôn ngữ bằng benchmark.

## Cấu trúc được triển khai

```text
neutral-engine/
  crates/astro-core/       # Contract, natal, individual/couple/composite/forecast contexts, event search và geometry
  crates/astro-provider-swiss/ # Swiss C, UTC/TT/UT1, Moshier, houses, state lock
  crates/astro-ffi/        # C ABI, quyền sở hữu output và containment panic
  crates/astro-node/       # Node-API gọi cùng astro-core
  include/astro_engine.h  # Header C/C++
bindings/node/            # Object/JSON API, TypeScript types, binary .node
bindings/python/          # Python wheel, ctypes wrapper, binary native
bindings/dotnet/          # NuGet, P/Invoke wrapper, binary theo RID
examples/                 # Consumer Node, Python, .NET, Rust và C
tests/conformance/        # Fixture đầu vào đối chiếu mọi binding
scripts/                  # Build, đóng gói, cài và kiểm tra consumer
docs/                     # Architecture, API, migration, release, frontend
```

Không chia thêm nhiều crate/package ngay khi chưa có nhu cầu. Provider hiện nằm trong `astro-provider-swiss`; chỉ thêm interface crate khi có provider thứ hai; tách `astro-core` thành types/math/features chỉ khi dependency và kích thước thực tế cần điều đó. Public package vẫn là một package tiện dụng cho mỗi ngôn ngữ.

## Ranh giới module

`astro-core/src/compression.rs` xử lý engine response sau tính toán, độc lập với 10 operation chiêm tinh. Cùng một bộ nén/giải nén phục vụ Rust, C ABI, Node-API và các bindings. Tables giảm tên trường lặp; dictionary chia sẻ các giá trị trùng nhau. Domain/section selection giữ context chứng cứ và ghi omissions. JSON parser bật `float_roundtrip` để bảo toàn các số thực của payload engine qua nén/giải nén. Bộ nén không gọi provider hoặc dịch vụ bên ngoài; SDK helper thực hiện tính toán trước rồi gọi bước xử lý này.

Core nhận positions hoặc UTC và tọa độ qua provider. Nó trả số liệu: longitude, speed, sign, houses, aspect angle, separation, orb, midpoint và overlays. `natalDomains` tổ chức các facts của một natal thành 10 built-in views và profiles tùy chỉnh theo profile versioned. Views khai báo quy tắc chọn houses/bodies/angles, mở rộng theo occupants/rulers, giữ contacts có ít nhất một selected endpoint và trả selection reasons.

`astro-core/src/natal.rs` thực hiện birth validation và ghép natal từ provider; `astro-core/src/domains.rs` dựng context 26 points, 325 pair relations, rulership/dispositors và views. `astro-core/src/advanced.rs` tính body states, distributions, chuỗi dispositor/receptions và aspect patterns. `astro-core/src/reports.rs` dựng nhà hỗ trợ có evidence và giữ mọi contact của cusp/occupants/ruler trên các nhà liên quan. `astro-core/src/profiles.rs` khai báo catalog 10 lĩnh vực/30 sections và validation cho custom profiles. Domain selection dùng cùng hàm cho builtin/custom/section; `reports::merge_sections` hợp nhất evidence và giữ section references mà không nhân bản nested reports. Provider chỉ được gọi để tính một lá số, không tính lại riêng cho từng lĩnh vực. Output khai báo `chartKind: individualNatal`, `subjectCount: 1`; xem [contract báo cáo cá nhân](individual-reports.md). Profile lựa chọn là policy công khai của dự án, tách với số liệu thiên văn; thay selection policy không làm thay tọa độ của cùng birth input. Không trả compatibility score, attachment style, lời khuyên, kết luận tình yêu hoặc AI narrative.

Profile lựa chọn có version `2.0`; rulership giữ version `1.0`. Traditional và modern là hai bảng chọn một ruler/cung, được khai báo trong output. House assignment giữ ecliptic longitude giữa cusps; chưa tính house position 3D theo body latitude. Góc ASC/MC/DSC/IC và cusp H1–H12 giữ IDs riêng; Whole Sign không lấy cusp H10 làm MC. Các nhà/góc có thể trùng vị trí nhưng không biến thành các bodies độc lập.

`Astro.Core` .NET từ lần tách trước là thư viện legacy, khác với `neutral-engine/crates/astro-core`. Nó không phải backend của các package mới. Các engine scoring love/work/social được giữ trong legacy để bảo toàn source; không được port vào neutral engine.

`astro-core/src/couple.rs` nhận hai birth inputs qua cùng natal validator/provider rồi xây context so sánh từ hai natal và advanced facts của chúng. Có 52 namespaced points, 676 cross relations, 20 body overlays và 144 cặp chủ tinh nhà A/B; views/sections dùng selectors đối xứng trên hai natal. Cross contacts giữ `applying: null`, không suy chuyển động từ hai natal epochs. Hai subjects giữ UTC/location trong natal và time/house system trong calculation riêng; provider không bị gọi lại cho mỗi lĩnh vực. Xem [couple.md](couple.md).

Synastry trung lập là phép đối chiếu hai chart. Operation `synastry` từ positions vẫn giữ contract geometry cũ; `couple` là operation birth-input mới. `astro-core/src/composite.rs` dựng chart C bằng circular midpoint của hai natal, kiểm tra nhà và lưu provenance. Builder không gọi provider; nhánh `couple.composite` dùng lại hai natal đã tính. Pipeline `domains`/`reports`/`advanced` nhận chart kind để tính lại facts cho C với motion không áp dụng, đồng thời giữ output natal cá nhân. C dùng catalog 10 lĩnh vực/30 sections, khác catalog 6 lĩnh vực synastry. Xem [composite.md](composite.md). Điểm chất lượng quan hệ thuộc ứng dụng bên ngoài.

`astro-core/src/events.rs` tìm crossings/stations/phases/exact aspects và gọi Swiss global eclipse APIs; `astro-core/src/forecast.rs` giữ một natal/context, chuyển events thành personal house/ruler/contact evidence, rồi tổ chức views theo profiles. Snapshot và root events tách phạm vi/indexes; không sao chép natal reports ở mỗi event. Provider session giữ cùng lock/time/ephemeris configuration trong một scan, tính selected bodies trực tiếp theo UT1 JD, không tính nhà ở mỗi tick. `natal` cũ giữ nguyên implementation/output. [forecast.md](forecast.md) công bố search budgets, tolerances, half-open intervals, offset/leap-second rules và highlight policy.

Swiss C được build với optimization level 3 ở cả Rust debug và release để giữ phép tính số học đồng nhất giữa crate mẫu và các native packages. Scanner xác nhận station bằng dấu tốc độ ở hai đầu bracket; không dùng tốc độ gần zero trong khoảng một giây để loại bỏ một nghiệm đã có bằng chứng đổi dấu.

## Provider và dữ liệu

Provider đầu tiên là Swiss Ephemeris 2.10.03, source C pin commit trong `crates/astro-provider-swiss/PROVENANCE.md`. Mode cố định Moshier, không cần file ephemeris. Source C được compile và link static vào native binary của mỗi binding. Public licensing vẫn là gate release. Skyfield hiện tại giữ vai trò nguồn tham chiếu đối chiếu trong quá trình chuyển đổi, không là dependency bắt buộc của package mới.

Mọi provider phải khai báo nguồn ephemeris, phiên bản, khoảng ngày hỗ trợ, time scale, tọa độ geocentric/topocentric, reference frame, apparent/astrometric và chế độ fallback. Các lựa chọn chưa tồn tại trong bản mẫu không được quảng bá trong capability list.

Không tải dữ liệu lúc import hoặc tính. Basic data có thể đi kèm package; dữ liệu lớn phân phối qua pack riêng hoặc đường dẫn do dev cung cấp. Phải kiểm tra checksum/version và trả lỗi khi thiếu dữ liệu; không âm thầm đổi sang phương pháp thiên văn khác. Natal alpha giới hạn input Gregorian 1800–2399; kiểm thử endpoints và một số epochs, chưa bảo chứng toàn bộ khoảng ngày. Build `ASTRO_MOSHIER_ONLY` chặn đọc file ephemeris/time-table ngoài để kết quả không phụ thuộc thư mục hoặc SE_EPHE_PATH.

## Tương thích nền tảng

Bản mẫu đã thiết kế builder cho macOS/Linux; kết quả chạy trực tiếp phải đọc ở `testing.md`. Windows Node cần import library của Node khi link và chưa được triển khai. Không coi các nền tảng dự kiến là đã được hỗ trợ.

Phase release nhắm macOS ARM64/x64, Windows x64 và Linux glibc x64/ARM64. Alpine/musl là target riêng. Python wheel cần manylinux hoặc musllinux khi phát hành Linux, không chỉ gắn tên wheel Linux. .NET sample hiện target net10.0; mở rộng TFM cần kiểm tra riêng. Bun, Deno, Java, Go, PHP và Swift chưa có wrapper.

## JSON và C ABI

JSON là contract ở ranh giới ngôn ngữ. Native core sử dụng type Rust nội bộ, không parse JSON ở mỗi bước phép tính. Node/Python/.NET nhận và trả object phù hợp ngôn ngữ; `calculateJson` là API thấp tầng để giữ nguyên envelope lỗi.

C ABI v1 cung cấp `astro_abi_version`, `astro_calculate_json` và `astro_free_string`. Output cấp phát bởi Rust phải được giải phóng đúng một lần bằng hàm của Rust; không dùng free của C hoặc .NET. Input NUL-terminated phải là UTF-8 và có lifetime đủ cho lời gọi. Binding kiểm tra NUL và kích thước trước khi gọi. C caller có trách nhiệm bảo đảm pointer hợp lệ. Panic được giữ trong boundary, không unwind sang runtime khác.

Geometry và domain selection không có mutable global state. Swiss được bảo vệ bởi Mutex chung trong từng native library; toàn bộ time conversion, positions, houses và swe_close nằm trong một critical section. Test trộn Placidus/Whole Sign và các tọa độ qua nhiều threads đã được bổ sung. Provider dùng tropical/Moshier cố định; chưa có sidereal/path options. Các lời gọi natal trong cùng library được tuần tự hóa để bảo toàn trạng thái provider. Domain context/views được dựng từ natal facts sau khi provider trả về.

## Browser và MCP

Native Node package không chạy trong browser. Browser package tương lai dùng WASM và worker; cần port cả provider, data loading và filesystem abstraction. Không coi việc Rust biên dịch WASM là toàn bộ ephemeris đã chạy được trong browser.

MCP là process cục bộ gọi package/engine, dùng stdio và log stderr. Dev không phải duy trì một server public. Phần MCP còn là kế hoạch, chưa có adapter được triển khai.

## Tài liệu tham chiếu

- [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [Node-API và ABI stability](https://nodejs.org/api/n-api.html)
- [Python ctypes](https://docs.python.org/3.13/library/ctypes.html)
- [.NET native interop](https://learn.microsoft.com/dotnet/standard/native-interop/abi-support)
- [Swiss Ephemeris programming interface](https://www.astro.com/swisseph-download/doc/swephprg.pdf)

## On-demand queries và adapters

`query.rs` dùng các công thức/validators hiện có cho geometry và dựng selection từ natal/context dùng chung. Node/Python/.NET namespaces chỉ đặt operation/group/action và gọi cùng JSON API. Payload inspection chứa các điểm hỗ trợ và indexes local; metadata phân biệt pure geometry với full natal rồi selection. Xem [query.md](query.md).

[MCP và frontend host](integrations.md) là adapters tùy chọn ngoài core/native package. MCP dùng SDK protocol chính thức và stdio local; frontend host chuyển request qua worker. Browser WASM vẫn cần provider/browser build riêng, không tải native Node-API trong React/Vue browser.
