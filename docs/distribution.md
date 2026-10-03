# GitHub công khai và phân phối package

Repo đích là [7mlabs/sdk-astro](https://github.com/7mlabs/sdk-astro), đang public và trống khi kiểm tra ngày 2026-10-03. Source engine/SDK được chuẩn bị trong checkout local riêng; chưa push hoặc publish registry. Connector chỉ có quyền đọc và CLI chưa đăng nhập, nhưng Git HTTPS qua credential helper đã xác thực bằng dry run thành công. Bước push thật còn chờ owner chọn license. Xem [thiết lập GitHub và phát hành](github-setup.md).

## Điều kiện trước khi public

Source clone ban đầu chưa có LICENSE ở gốc. Repo mới không import ứng dụng/history legacy; owner vẫn cần xác định license cho source mới và ghi nhận provenance. Không tự gán MIT/Apache cho toàn bộ source. Giữ npm `private: true` và Cargo `publish = false` đến khi gate đạt. Python/NuGet vẫn có thể upload về mặt công cụ, nhưng workflow hiện không có bước publish.

Swiss Ephemeris có license kép AGPL hoặc Professional theo [tài liệu upstream](https://www.astro.com/swisseph-download/doc/swephprg.pdf). Provider hiện đã được vendor để build/test local; source pin commit, có bản vá chặn file ngoài và notices được đóng gói. Không publish trước khi quyết định mô hình phù hợp. Việc tách provider không mặc nhiên giải quyết nghĩa vụ license của sản phẩm tích hợp. Cần giữ notices của source và data đi kèm.

## Tổ chức public repository

Public repo nên bao gồm core, bindings, examples, schema, docs, tests và workflows. Không commit build cache, virtualenv, node_modules, secrets, logs, dữ liệu sinh cá nhân hoặc toàn bộ publish binaries legacy. Binary release qua registry/GitHub Releases kèm checksum và platform manifest.

Checkout mới chỉ chứa `neutral-engine/`, `bindings/`, `schemas/`, `tests/`, `examples/`, `scripts/`, `docs/` và `.github/`. Không import lịch sử Git hoặc ứng dụng Python/.NET/sandbox cũ. Docs legacy chỉ lưu thông tin lịch sử. Giữ vendor licenses và third-party notices; bản scan theo pattern không thay thế việc xác nhận provenance.

## Package naming và version

Tên dự kiến `@7mlabs/astrology`, `sevenmlabs-astrology`, `SevenMLabs.Astrology`; quyền sở hữu tên và scope chưa được xác nhận, tên crate public sẽ xác minh sau. `0.10.0-alpha.1` trong npm/NuGet/Rust tương ứng `0.10.0a1` Python; metadata engine ghi version core. UI renderer có version riêng `0.9.0-alpha.1`, dùng payload schema `1.0`. Version artifact không được tái sử dụng cho bytes khác khi public. ABI/schema có version riêng; neutral engine không có scoring.

## Cài và cập nhật thuận tiện

Mỗi runtime có một package; engine native được đóng gói hoặc chọn tự động theo OS/CPU. SDK và core cùng version. Node cần entry package cùng optional platform packages pin đúng version; Python cần platform wheels; NuGet cần một nupkg chứa runtime assets của các nền tảng được hỗ trợ. Consumer Node/Python/.NET chỉ cần runtime và package; Rust/C source build cần compiler.

Candidate hiện build một platform mỗi lần, tạo tgz/nupkg cùng tên giữa hai job. Không publish các output đó thành cùng package version: phải assemble multi-platform hoặc tách optional platform packages trước. Linux wheel hiện có tag Linux thường, cần manylinux build/repair và kiểm thử môi trường sạch trước public PyPI. Windows chưa được builder hỗ trợ.

## CI và phát hành

CI candidate build/test macOS ARM64 và Linux x64, upload artifact phục vụ kiểm tra; không tạo release hoặc publish registry. Kết quả local chỉ xác nhận macOS ARM64. Release tương lai: tag → build matrix → fresh-install/parity → assemble → checksum/notices → publish → registry fresh-install verification → cập nhật docs. Owner cần cấu hình quyền GitHub và publisher registry, ưu tiên OIDC theo [hướng dẫn setup](github-setup.md).

## Không duy trì server

Bạn duy trì source, issue/PR, build matrix và releases. Registry/GitHub chứa source và artifacts. Người dùng tính trong process của họ; không có server uptime, database hoặc account của engine. MCP cũng chạy local. Cần bảo trì dependency và dữ liệu khi upstream đổi, không phải vận hành backend.
