# Thiết lập 7mlabs/sdk-astro

## Trạng thái

Repo GitHub: https://github.com/7mlabs/sdk-astro. Source tách riêng engine/SDK, không chứa lịch sử hoặc ứng dụng legacy, giữ third-party notices. Owner đã chọn AGPL-3.0-only ngày 2026-10-03; xem [license.md](license.md). Package chưa được phát hành trên npm, PyPI, NuGet hoặc crates.io.

Source đã được push lên nhánh `main` của repo công khai ngày 2026-10-03 bằng Git HTTPS qua credential helper sẵn có. Không cần gửi token trong chat hoặc commit credentials.

[Candidate CI đã qua trên macOS ARM64 và Linux x64](https://github.com/7mlabs/sdk-astro/actions/runs/37093556052), tại commit `2e1f53a0f786f3d0ad6309b8c659d6911717acda`: mỗi job có 107 tests Rust, 339 cases parity giữa 5 ngôn ngữ, 106 compression checks, 13 fixture roundtrips, installed SDK helpers/TypeScript và kiểm tra license/checksum trong package. Artifacts `candidate-darwin-arm64` và `candidate-linux-x64` lưu 14 ngày; chúng phục vụ kiểm tra candidate, chưa là bản phát hành registry.

## Kiểm tra source và build candidate

Chạy từ root repo với Rust 1.83+, Node 24, Python 3.13 và .NET 10 SDK. Native provider cần compiler C. macOS ARM64 đã được kiểm thử local; CI xác nhận candidate trên macOS 15 ARM64 và Ubuntu 24.04 x64. Windows chưa được builder hỗ trợ.

```bash
python3 scripts/check-repository.py
cargo test --manifest-path neutral-engine/Cargo.toml --locked
python3 -m venv artifacts/python-build
artifacts/python-build/bin/python -m pip install setuptools wheel
artifacts/python-build/bin/python scripts/build-packages.py
artifacts/python-build/bin/python scripts/test-packages.py
artifacts/python-build/bin/python scripts/test-compression.py
python3 scripts/check-repository.py --artifacts
```

CI `.github/workflows/neutral-engine.yml` chạy candidate builds trên `macos-15` (ARM64) và `ubuntu-24.04` (x64), thêm kiểm tra installed compression helpers/TypeScript. Artifacts theo platform lưu riêng; không phát hành registry. Linux wheel thường không thay thế manylinux wheel dành cho public PyPI.

`scripts/export-website-data.mjs` cần artifacts build/test thật và các báo cáo compression-schema/MCP integration được khai báo trong `scripts/website-catalog.mjs` trước khi export. Có thể xem các bước kiểm thử liên quan trong [payload-compression.md](payload-compression.md) và [integrations.md](integrations.md). Trong checkout SDK độc lập, truyền output rõ ràng thay vì dùng default sibling website:

```bash
node scripts/export-website-data.mjs --output artifacts/website-data
```

## Đưa source lên GitHub

License đã chốt là AGPL-3.0-only cho engine/SDK mới, dùng nhánh AGPL miễn phí của Swiss Ephemeris. Giữ toàn văn LICENSE, NOTICE và các notices trong provider/SDK packages; UI renderer có MIT riêng. Xem [distribution.md](distribution.md).

Khi GitHub credential có quyền ghi, checkout local có remote `origin` trỏ đến repo đích. Kiểm tra staged files, chạy repository checks và các tests ở trên, rồi commit/push. Không dùng force push hoặc import history legacy. Sau khi push, theo dõi cả hai jobs và cập nhật trạng thái thực trong README/docs.

## Cấu hình phát hành public

1. Xác nhận quyền sở hữu scope npm `@7mlabs` và các tên package PyPI/NuGet/crates.io. Giữ root LICENSE AGPL, package metadata và third-party provenance đã chốt.
2. Hoàn thiện native platform matrix và fresh install tests. Node cần resolver/optional platform packages; NuGet cần một nupkg tập hợp runtime assets; Python cần wheels được build/repair trên nền tảng tương ứng.
3. Cấu hình GitHub release environment và trusted publisher trên từng registry theo đúng owner `7mlabs`, repo `sdk-astro`, tên workflow và environment. Workflow publish sẽ được thêm khi gates đạt; hiện chưa tồn tại.
4. Thêm tag-based release workflow, đồng bộ version engine/SDK, xuất checksum/notices. Dùng alpha/prerelease trước stable, không ghi đè version đã phát hành.
5. Kiểm thử cài từ registry trong project mới rồi cập nhật website với version và platform đã xác nhận.

Nguồn cấu hình chính thức:

- [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)
- [PyPI trusted publishing](https://docs.pypi.org/trusted-publishers/)
- [NuGet trusted publishing](https://learn.microsoft.com/en-us/nuget/nuget-org/trusted-publishing)
- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)

Owner kiểm soát tài khoản/publisher và lựa chọn license. Source build, metadata, CI tests, package assembly và docs được duy trì trong repo; engine chạy cục bộ, không cần server tính toán.
