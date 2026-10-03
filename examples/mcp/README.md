# MCP stdio adapter tùy chọn

Ví dụ này dùng official `@modelcontextprotocol/sdk` **1.29.0** và native `@7mlabs/astrology` **0.10.0-alpha.1**. Nó không thêm dependency MCP vào engine và không mở HTTP port. MCP host launch process này và sở hữu vòng đời process.

## Cài và chạy

Tại thư mục `examples/mcp`, cài package native đã build cùng dependency adapter:

```sh
npm install ../../artifacts/packages/7mlabs-astrology-0.10.0-alpha.1.tgz
npm start
```

Native tarball và MCP example còn ở local, chưa publish registry. `npm install` lấy official SDK/Ajv từ registry nếu máy chưa có cache; quá trình **chạy và tính toán** không gọi mạng. Cần binary đúng platform/architecture. Node phải hỗ trợ package native hiện tại; kiểm thử trực tiếp trên macOS ARM64.

`npm start` chờ messages trên stdin. Không gửi log lên stdout, không thử server bằng `echo` thông thường. Chạy smoke bằng official client:

```sh
npm test
```

Adapter đọc canonical schemas từ `../../schemas`. Khi sao chép adapter ra project riêng, mang theo `query-request.schema.json`, `query-response.schema.json`, `compression-request.schema.json`, `compression-response.schema.json` và đặt `ASTRO_SCHEMA_DIR` trỏ tới thư mục đó. `ASTRO_ENGINE_MODULE` tùy chọn trỏ tới package native đã cài ở một project khác; thông thường chỉ cần install `@7mlabs/astrology` cùng project.

## Cấu hình MCP host

Host cần hỗ trợ transport stdio. Mẫu cấu hình phổ biến dưới đây; chỉnh theo format host thực tế, dùng absolute paths:

```json
{
  "mcpServers": {
    "astrology": {
      "command": "/absolute/path/to/node",
      "args": ["/absolute/path/to/astrology/examples/mcp/server.cjs"]
    }
  }
}
```

Không dùng `npx` tự tải package mỗi lần host khởi động. Adapter mẫu đã có dependencies local thì chỉ chạy `node server.cjs`. Không cần token, credentials hoặc URL server.

## Tools

- `astro_geometry`: `action: normalize | separation | midpoint`.
- `astro_aspects`: `action: between | inspect`.
- `astro_houses`: `action: locate | inspect`.
- `astro_points`: `action: inspect`.
- `astro_calculate`: `requestJson` chứa bất kỳ engine request đã serialize.
- `astro_compress_payload`: `payloadJson` chứa envelope đã tính; `options` chọn mode/domain/section/budget.
- `astro_calculate_context`: `requestJson` chứa request tính toán; `options` chuẩn bị context và chỉ trả context.

Grouped tool arguments có `action` và fields của action, không có `operation` hoặc `group`; routing do adapter inject. Ví dụ gọi `astro_geometry`:

```json
{ "action": "normalize", "longitude": -10 }
```

Ví dụ gọi `astro_points`:

```json
{
  "action": "inspect",
  "birth": {
    "utc": { "year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0 },
    "location": { "latitude": 10.8231, "longitude": 106.6297 },
    "houseSystem": "wholeSign"
  },
  "pointIds": ["sun", "ascendant", "H1"]
}
```

Các request chi tiết theo [query request schema](../../schemas/query-request.schema.json). Group tools được derive từ schema này, không tạo bộ validation tính toán riêng. Generic tool dùng raw string để giữ nguyên decimal/exponent literals của input; native core quyết định hợp lệ.

Tool returns nguyên engine envelope ở `structuredContent` và JSON text trong `content`. Core errors giữ nguyên và đặt `isError: true`; lỗi schema/unknown tool là protocol errors. Metadata readOnly/idempotent mô tả tool chỉ tính dữ liệu. Adapter không lưu dữ liệu, không diễn giải, không cho điểm và không gọi mạng.

## Context đã nén

`astro_calculate_context` gọi tính toán và bộ nén chung trong native core. Ví dụ arguments:

```json
{
  "requestJson": "{\"operation\":\"natalDomains\",\"utc\":{\"year\":2000,\"month\":1,\"day\":1,\"hour\":12,\"minute\":0},\"location\":{\"latitude\":10.8231,\"longitude\":106.6297},\"domains\":[\"career\",\"love\"]}",
  "options": {"mode": "budgeted", "domains": ["career"], "maxBytes": 65536}
}
```

Kết quả thành công có `data.format: "astro-context/1"`, encoded source `payload`, `dictionary`, `coverage`, `omitted`, `metrics`, `budget`. Tool này không trả thêm payload gốc. Khi tính toán thất bại, nó giữ nguyên error envelope `data: null`, đặt `isError: true` và không nén để che lỗi. `astro_compress_payload` có cùng options nhưng nhận engine response envelope thay vì request. Năm tool trước vẫn trả dữ liệu gốc.

Đọc `$astroTable` theo columns/rows và `$astroRef` theo dictionary; `$astroLiteral` giữ source object có reserved keys. Gửi cả context envelope vào prompt để có dictionary và metadata. Host có thể chọn một representation giữa JSON text và structuredContent để tránh lặp. Ngân sách chỉ đo context native minified, không tính MCP transport/prompt/history.

`compact` giữ tất cả JSON values, `focused` chọn domain/section và giữ shared supporting facts, `budgeted` thêm kiểm tra giới hạn. Nếu dữ kiện còn lớn hơn budget, `budget.exceeded: true` và warning `COMPRESSION_BUDGET_EXCEEDED`; adapter không xóa fact để ép vừa và không coi warning này là protocol error. `maxTokens` dùng estimate `utf8-bytes-conservative`, không đếm bằng tokenizer model. Xem [payload compression docs](../../docs/payload-compression.md) trước khi dùng cho báo cáo chuyên sâu.

## Kiểm thử thực tế

`smoke.cjs` dùng official client qua stdio, kiểm tra handshake/listing, tám actions, structured/text parity, native errors, generic chart/couple/forecast, routing override và unknown tool. `smoke-compression.cjs` bổ sung compact roundtrip trên natal/couple/composite/forecast, context-only combined tool, focused selector, unknown domain và budget quá nhỏ. `npm test` chạy cả hai script. Native module có thể lấy từ package đã cài qua `ASTRO_ENGINE_MODULE`, không dùng source binding.

Đã chạy trên macOS ARM64 / Node.js 24.19.0 với package native `0.10.0-alpha.1` cài local: **7 tools, 16 calls regression và 15 calls compression đạt**. Native errors của ngày sinh sai và integer literal có phần lẻ được giữ nguyên trong tool `astro_calculate_context`. Các dependencies official MCP SDK 1.29.0 và Ajv 8.20.0 được load từ cache local; tính toán không gọi mạng.

Lệnh kiểm thử có thể chạy từ repo root với các paths tương ứng trên máy:

```sh
ASTRO_ENGINE_MODULE="/absolute/path/to/consumer/node_modules/@7mlabs/astrology" \
NODE_PATH="/absolute/path/to/cached/node_modules:/absolute/path/to/cached/node_modules/@modelcontextprotocol/sdk/node_modules" \
node examples/mcp/smoke.cjs
```

Hoặc cài dependencies trong `examples/mcp` bằng lệnh phía trên rồi chạy `npm test`; khi đó không cần `NODE_PATH`.

Xem [integration docs](../../docs/integrations.md) để ghép SDK với MCP hoặc frontend desktop/browser đúng runtime. MCP stdio hỗ trợ local host có process access; một client chỉ nhận URL HTTP cần adapter remote riêng.

Primary references: [MCP stdio](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [MCP tools/results](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [official TypeScript SDK v1](https://ts.sdk.modelcontextprotocol.io/server).
