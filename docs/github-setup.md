# Thiết lập 7mlabs/sdk-astro

## Trạng thái

Repo GitHub: [7mlabs/sdk-astro](https://github.com/7mlabs/sdk-astro). Source tách riêng engine/SDK, không chứa lịch sử hoặc ứng dụng legacy, giữ third-party notices. Owner đã chọn AGPL-3.0-only ngày 2026-10-03; xem [license.md](license.md). [@7mlabs/astrology `0.10.0-alpha.1`](https://www.npmjs.com/package/@7mlabs/astrology) đã phát hành npm với tag `alpha`. PyPI, NuGet, crates.io và UI renderer chưa phát hành registry.

Source đã được push lên nhánh `main` của repo công khai ngày 2026-10-03 bằng Git HTTPS qua credential helper sẵn có. Không cần gửi token trong chat hoặc commit credentials.

[CI run 37096125672 đã qua trên macOS ARM64 và Linux x64](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672), tại commit `a17850e01507485312e5cb584ed1eb82a786fcbc`: mỗi candidate job có 108 tests Rust, 339 cases parity giữa 5 ngôn ngữ, 106 compression checks, 13 fixture roundtrips, installed SDK helpers/TypeScript và license/checksum validation. Assembler ghép hai binary; bốn final consumer jobs kiểm tra cùng tarball trên Node 18/24 ở cả hai target. Artifacts candidate lưu 14 ngày, `npm-release` và các báo cáo final verification lưu 30 ngày. Bản registry npm đã kiểm tra integrity/tag và cài mới thực tế; SHA256 tarball là `d8dcfdbaea4e66070a75f9e5ef91d0e6302b33995b364dce40b6a1e28eba9748`.

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

CI `.github/workflows/neutral-engine.yml` chạy candidate builds trên `macos-15` (ARM64) và `ubuntu-24.04` (x64), thêm installed compression helpers/TypeScript, assemble npm release và final consumer matrix Node 18/24. Artifacts theo platform lưu riêng; workflow này không phát hành registry. `.github/workflows/npm-release.yml` gọi lại toàn bộ pipeline và chỉ publish khi tất cả gates qua. Linux wheel thường không thay thế manylinux wheel dành cho public PyPI.

`scripts/export-website-data.mjs` cần artifacts build/test thật và các báo cáo compression-schema/MCP integration được khai báo trong `scripts/website-catalog.mjs` trước khi export. Có thể xem các bước kiểm thử liên quan trong [payload-compression.md](payload-compression.md) và [integrations.md](integrations.md). Trong checkout SDK độc lập, truyền output rõ ràng thay vì dùng default sibling website:

```bash
node scripts/export-website-data.mjs --output artifacts/website-data
```

## Đưa source lên GitHub

License đã chốt là AGPL-3.0-only cho engine/SDK mới, dùng nhánh AGPL miễn phí của Swiss Ephemeris. Giữ toàn văn LICENSE, NOTICE và các notices trong provider/SDK packages; UI renderer có MIT riêng. Xem [distribution.md](distribution.md).

Khi GitHub credential có quyền ghi, checkout local có remote `origin` trỏ đến repo đích. Kiểm tra staged files, chạy repository checks và các tests ở trên, rồi commit/push. Không dùng force push hoặc import history legacy. Sau khi push, theo dõi toàn bộ jobs và cập nhật trạng thái thực trong README/docs.

## Cấu hình phát hành public

1. npm scope `7mlabs` và package `@7mlabs/astrology` đã xác minh; lần phát hành đầu hoàn tất bằng tài khoản có 2FA. Giữ root LICENSE AGPL, package metadata và third-party provenance đã chốt. Xác minh các tên PyPI/NuGet/crates.io khi phát triển tiếp những registry đó.
2. npm đã có một tarball chứa hai addon và loader chọn binary theo host. Giữ final fresh-install tests trên Node 18/24 và hai target. NuGet còn cần một nupkg tập hợp runtime assets; Python cần wheels được build/repair trên nền tảng tương ứng.
3. npm trusted publisher đã cấu hình theo repository `7mlabs/sdk-astro`, workflow filename `npm-release.yml`, environment `npm-release`. Job publish có OIDC riêng; chưa có lần publish thực tế bằng OIDC. Cần xác nhận lần phát hành tự động tiếp theo qua trước khi coi đường publish này đã được kiểm chứng. Không cần gửi token trong chat.
4. Khi cập nhật, đồng bộ version engine/SDK và các version/filename của builder theo [checklist phát hành](node-release.md), rồi chạy toàn bộ gate; push tag alpha khớp version, ví dụ `v0.10.0-alpha.2`. Workflow kiểm tra HEAD/tag/provenance/checksum/bốn consumer reports trước publish. Không push tag để publish lại `0.10.0-alpha.1`; version registry là bất biến.
5. Sau mỗi release, kiểm tra registry integrity/dist-tag và cài trong project mới rồi cập nhật website với version/platform thực tế. Lần đầu đã hoàn tất các kiểm tra này trên macOS ARM64; Linux được kiểm chứng với cùng tarball trong CI.

Nguồn cấu hình chính thức:

- [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)
- [PyPI trusted publishing](https://docs.pypi.org/trusted-publishers/)
- [NuGet trusted publishing](https://learn.microsoft.com/en-us/nuget/nuget-org/trusted-publishing)
- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)

Owner kiểm soát tài khoản/publisher và lựa chọn license. Source build, metadata, CI tests, package assembly và docs được duy trì trong repo; engine chạy cục bộ, không cần server tính toán.
