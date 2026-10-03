# Changelog

## 0.10.0 — stable target, đang chờ xác minh phát hành

- Chuyển package npm `@7mlabs/astrology` sang version `0.10.0` và kênh ổn định `latest`; lệnh cài chính là `npm install @7mlabs/astrology`.
- Đồng bộ version core và SDK Node/Python/.NET thành `0.10.0`; Python và .NET tiếp tục dùng artifacts local, chưa phát hành registry.
- Release tooling xác thực version và chọn dist-tag từ metadata: stable dùng `latest`, prerelease giữ kênh riêng. Source npm tiếp tục có `private: true`; chỉ tarball đã qua gate được publish.
- Phạm vi binary giữ macOS ARM64 và Linux x64/glibc 2.38+; UI renderer giữ version độc lập `0.9.0-alpha.1`.
- Bằng chứng CI, checksum tarball và xác minh registry/OIDC sẽ được ghi sau khi hoàn tất release gate và publish.

## 0.10.0-alpha.1 — npm alpha, 2026-10-03

- Engine Rust chạy offline, provider Swiss Ephemeris/Moshier, Node-API và C ABI dùng chung.
- Natal, natal theo 10 lĩnh vực, synastry cặp đôi, midpoint composite, events, forecast, grouped query và geometry operations.
- Bộ nén payload riêng: compact roundtrip, focused selection, budgeted context và helpers calculate-with-context. Token metric là ước lượng bảo thủ theo UTF-8 bytes, chưa phải tokenizer cụ thể.
- Bindings và examples Node.js/TypeScript, Python, .NET, Rust, C; mẫu MCP local và UI renderer có bridge được inject.
- Schema JSON, reference fixtures, parity/fresh-install tests và docs.
- Chuẩn bị repo `7mlabs/sdk-astro`, metadata repository và CI candidate có artifact checksums/evidence. CI không publish registry.
- Build provider C99 trên Linux khai báo `_GNU_SOURCE` để có đủ libc file-offset/dl declarations; không đổi thuật toán thiên văn.
- Root refinement kiểm tra hữu hạn mốc JD lân cận khi số thực không thể chia bracket tiếp; vẫn yêu cầu residual thực và bracket đổi dấu đạt cùng tolerance.

Tarball npm alpha đã phát hành public với hai native binaries macOS ARM64 và Linux x64 (glibc 2.38+). CI run `37096125672` xác nhận 108 Rust tests, 339 cross-language conformance cases và consumer matrix Node 18/24; registry integrity/tag và cài mới trên macOS ARM64 đã được kiểm tra. Python/.NET vẫn là artifacts local, chưa phát hành PyPI/NuGet. Windows, macOS Intel, Linux ARM64 và Alpine/musl chưa có binary. Engine/SDK dùng AGPL-3.0-only; UI giữ MIT riêng.

UI renderer giữ version riêng `0.9.0-alpha.1`, tương thích payload schema `1.0`.
