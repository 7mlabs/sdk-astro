# Thiết lập 7mlabs/sdk-astro

## Trạng thái

Repo GitHub đích: https://github.com/7mlabs/sdk-astro. Bản source local tách riêng engine/SDK, không chứa lịch sử hoặc ứng dụng legacy, đã giữ third-party notices. Source chưa được push; package chưa được phát hành trên npm, PyPI, NuGet hoặc crates.io.

Kiểm tra quyền ngày 2026-10-03: connector GitHub có `pull: true`, `push: false`; `gh auth status` báo chưa đăng nhập. Cần cấp quyền ghi `7mlabs/sdk-astro` cho kết nối GitHub đang dùng, hoặc đăng nhập GitHub CLI bằng tài khoản có quyền ghi repo. Không gửi token trong chat hoặc commit credentials.

## Kiểm tra source và build candidate

Chạy từ root repo với Rust 1.83+, Node 24, Python 3.13 và .NET 10 SDK. Native provider cần compiler C. macOS ARM64 đã được kiểm thử local; Linux x64 sẽ cần GitHub Actions xác nhận. Windows chưa được builder hỗ trợ.

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

`scripts/export-website-data.mjs` cần artifacts build/test thật trước khi export. Trong checkout SDK độc lập, truyền output rõ ràng thay vì dùng default sibling website:

```bash
node scripts/export-website-data.mjs --output artifacts/website-data
```

## Đưa source lên GitHub

Trước push, owner xác định license cho source mới và mô hình phân phối Swiss Ephemeris; xem [distribution.md](distribution.md). Giữ các notices trong provider và SDK packages. Không tự gán MIT cho toàn bộ engine.

Khi GitHub credential có quyền ghi, checkout local có remote `origin` trỏ đến repo đích. Kiểm tra staged files, chạy repository checks và các tests ở trên, rồi commit/push. Không dùng force push hoặc import history legacy. Sau khi push, theo dõi cả hai jobs và cập nhật trạng thái thực trong README/docs.

## Cấu hình phát hành public

1. Xác nhận quyền sở hữu scope npm `@7mlabs` và các tên package PyPI/NuGet/crates.io. Chốt root LICENSE, metadata và third-party provenance.
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
