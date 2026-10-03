# GitHub công khai và phân phối package

Repo công khai là [7mlabs/sdk-astro](https://github.com/7mlabs/sdk-astro), đã upload source engine/SDK riêng. Owner đã chọn nhánh license miễn phí AGPL ngày 2026-10-03. [@7mlabs/astrology `0.10.0-alpha.1`](https://www.npmjs.com/package/@7mlabs/astrology) đã phát hành public trên npm với tag `alpha`; Python/.NET/Rust/UI renderer chưa phát hành registry. Xem [thiết lập GitHub và phát hành](github-setup.md).

## Điều kiện trước khi public

Repo mới không import ứng dụng/history legacy. Source mới và engine packages dùng `AGPL-3.0-only`, có toàn văn LICENSE/NOTICE và provenance của third-party. Renderer độc lập giữ MIT. Source npm giữ `private: true`; assembler chỉ bỏ guard trong tarball public đã qua release gate. Cargo giữ `publish = false`. Python/NuGet vẫn có thể upload về mặt công cụ, nhưng workflow hiện không có bước publish chúng.

Swiss Ephemeris có license kép AGPL hoặc Professional theo [tài liệu upstream](https://www.astro.com/swisseph-download/doc/swephprg.pdf); dự án chọn AGPL miễn phí. Provider được vendor theo commit đã pin, có bản vá chặn file ngoài ngày 2026-10-02 và notices đi kèm. Phần mềm tích hợp engine cần tuân thủ các nghĩa vụ AGPL tương ứng; xem [license.md](license.md).

## Tổ chức public repository

Public repo nên bao gồm core, bindings, examples, schema, docs, tests và workflows. Không commit build cache, virtualenv, node_modules, secrets, logs, dữ liệu sinh cá nhân hoặc toàn bộ publish binaries legacy. Binary release qua registry/GitHub Releases kèm checksum và platform manifest.

Checkout mới chỉ chứa `neutral-engine/`, `bindings/`, `schemas/`, `tests/`, `examples/`, `scripts/`, `docs/` và `.github/`. Không import lịch sử Git hoặc ứng dụng Python/.NET/sandbox cũ. Docs legacy chỉ lưu thông tin lịch sử. Giữ vendor licenses và third-party notices; bản scan theo pattern không thay thế việc xác nhận provenance.

## Package naming và version

Tên npm `@7mlabs/astrology` và scope `7mlabs` đã xác minh quyền sở hữu và phát hành. `sevenmlabs-astrology`, `SevenMLabs.Astrology` và tên crate public còn cần xác minh registry tương ứng. `0.10.0-alpha.1` trong npm/NuGet/Rust tương ứng `0.10.0a1` Python; metadata engine ghi version core. UI renderer có version riêng `0.9.0-alpha.1`, dùng payload schema `1.0`. Version artifact không được tái sử dụng cho bytes khác khi public. ABI/schema có version riêng; neutral engine không có scoring.

## Cài và cập nhật thuận tiện

Mỗi runtime có một package; engine native được đóng gói hoặc chọn tự động theo OS/CPU. SDK và core cùng version. Bản npm đầu dùng một tarball chứa hai addon macOS ARM64 và Linux x64, loader chọn đúng binary; chưa cần optional platform packages. Có thể tách platform packages khi matrix lớn hơn. Python cần platform wheels; NuGet cần một nupkg chứa runtime assets của các nền tảng được hỗ trợ. Consumer cần runtime và package; Rust/C source build cần compiler.

```sh
npm install @7mlabs/astrology@alpha
```

Cập nhật bằng cách chạy lại lệnh trên. Pin bằng `npm install --save-exact @7mlabs/astrology@0.10.0-alpha.1` và commit lockfile. Package chưa có stable `latest`; nền tảng và libc requirements nằm trong [hướng dẫn Node.js](node-release.md).

Candidate build một platform mỗi lần, tạo tgz/nupkg cùng tên giữa hai job. npm assembler đã ghép hai candidate vào một tarball duy nhất trước release; không publish riêng hai candidate cùng package/version. NuGet vẫn cần assembly nhiều runtime trước public. Linux wheel hiện có tag Linux thường, cần manylinux build/repair và kiểm thử môi trường sạch trước public PyPI. Windows chưa được builder hỗ trợ.

## CI và phát hành

CI candidate build/test macOS ARM64 và Linux x64, assemble npm tarball rồi fresh-install tarball cuối trên Node 18/24 ở mỗi target; workflow này không publish registry. [Run 37096125672](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672) đã qua tại commit `a17850e01507485312e5cb584ed1eb82a786fcbc`: 108 tests Rust, 339 parity cases giữa năm ngôn ngữ và bốn final npm consumer jobs. SHA256 bản npm đã publish: `d8dcfdbaea4e66070a75f9e5ef91d0e6302b33995b364dce40b6a1e28eba9748`. Registry integrity/tag và fresh registry install trên macOS ARM64 đã được kiểm tra. Bằng chứng Linux giới hạn ở Ubuntu 24.04 x64 và không xác nhận tính portable của wheel trên các distro khác.

Workflow `npm-release.yml` tách quyền publish, chạy lại gates rồi dùng tag alpha/version bất biến và kiểm tra registry checksum/dist-tag. Trusted publisher đã cấu hình cho `7mlabs/sdk-astro`, workflow `npm-release.yml`, environment `npm-release`; lần phát hành đầu dùng tài khoản npm có 2FA, chưa có publish thực tế qua OIDC. Bản tiếp theo phải tăng version rồi dùng tag tương ứng. Các publisher PyPI/NuGet/crates.io được xử lý sau; xem [hướng dẫn setup](github-setup.md).

## Không duy trì server

Bạn duy trì source, issue/PR, build matrix và releases. Registry/GitHub chứa source và artifacts. Người dùng tính trong process của họ; không có server uptime, database hoặc account của engine. MCP cũng chạy local. Cần bảo trì dependency và dữ liệu khi upstream đổi, không phải vận hành backend.
