# Changelog

## 0.10.0-alpha.1 — candidate, chưa phát hành registry

- Engine Rust chạy offline, provider Swiss Ephemeris/Moshier, Node-API và C ABI dùng chung.
- Natal, natal theo 10 lĩnh vực, synastry cặp đôi, midpoint composite, events, forecast, grouped query và geometry operations.
- Bộ nén payload riêng: compact roundtrip, focused selection, budgeted context và helpers calculate-with-context. Token metric là ước lượng bảo thủ theo UTF-8 bytes, chưa phải tokenizer cụ thể.
- Bindings và examples Node.js/TypeScript, Python, .NET, Rust, C; mẫu MCP local và UI renderer có bridge được inject.
- Schema JSON, reference fixtures, parity/fresh-install tests và docs.
- Chuẩn bị repo `7mlabs/sdk-astro`, metadata repository và CI candidate có artifact checksums/evidence. CI không publish registry.

Nền tảng có bằng chứng local: macOS ARM64. Linux x64 là CI candidate, cần kết quả runner xác nhận. Windows và final multi-platform registry packages chưa hoàn thiện. Owner đã chọn AGPL-3.0-only cho engine/SDK mới, giữ notices và license MIT riêng của UI. Quyền registry và kiểm tra platform vẫn là điều kiện trước registry release.

UI renderer giữ version riêng `0.9.0-alpha.1`, tương thích payload schema `1.0`.
