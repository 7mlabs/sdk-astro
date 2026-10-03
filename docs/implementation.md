# Plan triển khai và kiểm soát phát hành

Plan này mô tả công việc cụ thể sau bản mẫu, từ chuyển source đến phân phối package. Cần duy trì CI và registry releases; không cần vận hành API server cho người dùng.

## Chuyển source hiện tại

| Source legacy | Đích | Hành động |
|---|---|---|
| `Astro.Physics/main.py` | Provider và app service Rust | Tách ephemeris, houses, time search khỏi FastAPI; đối chiếu thuật toán |
| `Astro.Physics/angles.py` | Geometry Rust | Đối chiếu wrap/speed conventions |
| `AstrologyEngineService.cs` | Geometry/features Rust | Tách I/O, themes và công thức; port các phần trung lập |
| `AstrologyEngineService.Features.cs` | Module features | Port lần lượt midpoint/harmonic/bounds và kiểm chứng |
| `AstrologyEngineService.Synastry.cs` | Synastry/composite | Giữ dữ liệu so sánh, bỏ relationship metrics mang ý nghĩa ứng dụng |
| `*CompatScoringEngine.cs`, `LifeDomains.cs`, `Themes.json` | Legacy/application layer | Không port scoring/narrative; domain selection mới có policy/version riêng |
| `SvgRenderService.cs` | Rendering tùy chọn | Tách khỏi toán học; có thể thay bằng renderer browser sau |
| `astro-sandbox` | Legacy sandbox | Giữ để đối chiếu; không dùng làm bằng chứng frontend offline |

Không xóa legacy cho đến khi module mới đạt coverage, parity và tài liệu. Chuyển sang Rust không giữ mọi hành vi sai của source cũ. Mỗi thay đổi kết quả có migration note giải thích nguyên nhân.

## Điểm dừng hiện tại

Đã có raw UTC/location → natal JSON cơ bản trong `astro-core/src/natal.rs` và `astro-provider-swiss`. Local package `0.9.0-alpha.1` tích hợp Swiss C/Moshier và bổ sung `astro-core/src/domains.rs` để tổ chức facts thành payload theo lĩnh vực. Các module `advanced.rs` và `reports.rs` bổ sung phép tính natal nâng cao và context phục vụ báo cáo cá nhân; contract nằm trong [individual-reports.md](individual-reports.md). Operation `couple` bổ sung hai birth inputs → hai natal → cross context và 6 lĩnh vực/18 sections; xem [couple.md](couple.md). Operation `composite` trong `composite.rs` dựng C từ hai natal, tính lại shared facts và 10 lĩnh vực/30 sections; `couple` có opt-in để dùng lại hai natal, xem [composite.md](composite.md). `events.rs` bổ sung bounded scanner/provider session và global eclipse APIs; `forecast.rs` gắn các mốc với natal/context và 10 lĩnh vực/30 sections. Xem [forecast.md](forecast.md). `query.rs` bổ sung grouped on-demand calculations/selections; có mẫu MCP local và frontend worker bridge, xem [query.md](query.md) và [integrations.md](integrations.md). Davison, progression/returns, browser WASM, frontend UI và public release vẫn là backlog.

## Triển khai payload lĩnh vực

1. Giữ shared birth validation/provider; dispatch riêng `natal` và `natalDomains` để options lĩnh vực không lọt vào API cơ bản.
2. Công bố profile `2.0` cho 10 lĩnh vực, 30 sections, catalog và custom profiles; chọn houses/bodies/angles và rulership traditional/modern rõ ràng.
3. Dựng full context từ một natal: 26 points, 12 enriched houses, 10 dispositors, 325 relations, flat aspects với global index.
4. Dựng views từ occupants/rulers/focus objects; include contacts có ít nhất một selected endpoint, giữ selection reasons và join references.
5. Kiểm tra schema/runtime options, preset/custom exclusions, disabled matching, structural pairs, Whole Sign MC và parity packages.
6. Dựng derived facts: body states, distributions, dispositor chains, mutual reception và 5 body-only aspect pattern types; công bố rules version và các giới hạn.
7. Dựng report từ primary profile và nhà hỗ trợ có evidence; giữ toàn bộ contacts trên cusp/occupants/ruler, không mở rộng nhà đệ quy. Liên kết đủ 66 cặp nhà qua các chủ tinh.
8. Chuẩn hóa custom profile IDs/version/selectors/limits; section chọn độc lập và merge report facts qua references. Test custom-only/mixed, exact decimal house numbers và scope một người.
9. Cập nhật Node types, request/response schemas, samples/ngôn ngữ, docs API và kết quả test thực sau khi build/install lại artifacts.

Profile không copy các kết luận spouse/wealth/attachment/health trong legacy. Metadata `houseAssignment: eclipticLongitude` mô tả phương pháp đang dùng; mở rộng 3D house positions, extra bodies cần contract và reference tests riêng. Two-person relationships có operation `couple` riêng, không mở rộng ngầm `natalDomains`.

## Triển khai cặp đôi

1. Tái sử dụng birth validation và provider cho từng người; từ chối birth/options đặt sai cấp.
2. Tạo hai natal/full contexts cùng advanced facts; giữ metadata và house system riêng.
3. Gắn namespace A/B vào global point IDs, dựng đủ 26 × 26 cross relations và mọi matches theo rules.
4. Dựng 10 body overlays mỗi chiều và 12 × 12 cross house-ruler relations, giữ references tới points/relations/aspects.
5. Tổ chức 6 lĩnh vực/18 sections và custom selectors đối xứng, giữ endpoint ngoài focus và section evidence.
6. Test direction, options/calendar/namespace/reference closure, disabled aspects, concurrency và regression cá nhân/geometry.
7. Cập nhật schema/types/docs, build artifact, cài consumer và đối chiếu Node/Python/.NET/Rust/C. Kết quả thực ghi trong [testing.md](testing.md).

## Backlog tương lai

1. Review contract geometry, harden FFI, bổ sung property/fuzz tests và benchmark.
2. Xác minh license; chọn provider version/data policy và proof native natal.
3. Chuẩn hóa birth request, timezone helper, provider metadata và errors.
4. Port houses/aspects với fixtures; kiểm chứng frame trước khi so Skyfield/Swiss.
5. Nâng wrapper từ alpha thành public API; Node worker API cho scans dài, typed DTO .NET/Python, cancellation.
6. Build matrix macOS/Linux/Windows, prebuilt binary selection, package installation tests.
7. Tạo release candidate, test registry sandbox, rồi public đúng phiên bản đã kiểm tra.
8. Mở rộng orb-entry/exit windows, returns/progression, Davison/reference-place composite và các kỹ thuật synastry khác; sau đó MCP và browser runtime.

## Quy trình một thay đổi thuật toán

Ghi rõ input và kết quả trước/sau → thay core → test fixtures và invariants → build wrappers → parity consumers → cập nhật contract/docs/changelog. Binding không có công thức nghiệp vụ để sửa riêng. Khi thay shape/meaning JSON, bump schema hoặc engine major phù hợp; version ABI thay khi function signature/ownership thay.

## Quy trình build package

Builder hiện dùng `scripts/build-packages.py`, tạo binary rồi stage vào mỗi binding. Đây là mẫu build cho current host; chưa là cross-build framework hoàn chỉnh. Cần Python build venv, Rust compiler, Node/npm và .NET SDK trên máy người phát hành. Dev cài artifact được hỗ trợ chỉ cần runtime ngôn ngữ của mình.

- Node: cargo tạo addon, pack tgz có wrapper/types/native binary. Public release nên dùng platform-specific optional packages để tránh một tgz quá lớn.
- Python: wheel `py3-none-<platform>` chứa shared library và ctypes wrapper. Linux public wheel cần auditwheel/manylinux; macOS cần kiểm tra minimum deployment target; Windows cần dependency DLL policy.
- .NET: pack NuGet có managed assembly và `runtimes/<rid>/native/`; consumer restore phải tự copy/load đúng native library.
- Rust: crate phụ thuộc core/provider theo features; C/C++ nhận header và library từ GitHub Release. Bản mẫu Rust dùng path dependency local vì chưa publish crate.

## CI và release

CI candidate trong `.github/workflows/neutral-engine.yml` chỉ chạy core/package tests trên macOS/Linux, chưa publish. Nó không thay thế kết quả kiểm thử Windows hoặc Linux ARM64. Release workflow tương lai cần tag chính thức, GitHub environment và registry credential/trusted publishing được cấu hình bởi owner.

Các bước release: checkout sạch → test Rust → build current target → pack → cài artifacts trong consumer sạch → parity → tạo checksums/SBOM/notice → candidate review → publish version đã kiểm tra → registry install verification. Artifact thử nội bộ hiện chưa có license public đã giải quyết và không được publish.

## Definition of done

- Có input/output docs và sample chạy được cho từng language quảng bá.
- Package cài từ artifact/registry thật, không phụ thuộc source repo hoặc localhost.
- Binding cùng engineVersion và cùng JSON cho mọi fixture.
- Có OS/arch/runtime support matrix theo thực nghiệm.
- Không runtime network, không tải ephemeris bất ngờ, không bỏ qua dữ liệu thiếu.
- Có license, notices và quyền phân phối source/data trước public.
- Có benchmark thực trước khi đặt cam kết latency/throughput.
