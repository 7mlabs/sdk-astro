# Tích hợp SDK vào MCP và frontend

Engine chạy native trong tiến trình của ứng dụng. SDK Node.js, Python, .NET, Rust và C đều gọi cùng core; không phải HTTP client và không cần dịch vụ tính toán do 7MLabs vận hành. MCP và giao diện là hai adapter tùy chọn nằm ngoài dependency của engine.

Từ `0.10.0-alpha.1`, [bộ nén payload](payload-compression.md) có API riêng và helper `calculateWithContext`. Adapter MCP có thêm `astro_compress_payload` và `astro_calculate_context`, tổng cộng 7 tools. Tool tính context chỉ gửi context đã nén; SDK helper giữ cả payload gốc để ứng dụng dùng cho UI. Renderer nhận `result` nguyên bản; LLM nhận toàn bộ `context` gồm dictionary và coverage.

## API theo nhóm tính toán

Từ `0.9.0-alpha.1`, operation `query` hỗ trợ truy vấn nhỏ theo `group`/`action`. SDK cung cấp các namespace tương ứng; các phương thức nhận một object options và trả nguyên envelope, dữ liệu ở `.data`.

| Nhóm | Phương thức | Đầu vào chính | Nhu cầu |
|---|---|---|---|
| `geometry` | `normalize` | `longitude` | Chuẩn hóa kinh độ, cung và độ trong cung |
| `geometry` | `separation` | `longitude1`, `longitude2` | Độ lệch có dấu và khoảng cách góc |
| `geometry` | `midpoint` | Hai longitude, `antipodalPolicy?` | Trung điểm trên vòng tròn, xử lý đối đỉnh rõ ràng |
| `aspects` | `between` | `positions` đúng hai điểm, `rule`, `motionMode?` | Kiểm tra một góc chiếu/orb; chọn rõ chế độ motion |
| `aspects` | `inspect` | `birth`, `pointIds?`, aspect options | Contacts giữa các điểm natal được chọn |
| `houses` | `locate` | `longitude`, `houseCusps` đúng 12 cusp | Định vị điểm trong nhà từ cusps đã có |
| `houses` | `inspect` | `birth`, `houseNumbers?`, options | Nhà natal, chủ tinh và facts hỗ trợ |
| `points` | `inspect` | `birth`, `pointIds?`, options | Hành tinh, angles, cusp và facts liên quan |

Inspection options gồm `rulership` và `aspectPreset` hoặc `aspectRules`; không gửi đồng thời hai loại aspect settings. Birth chứa `utc`, `location`, `houseSystem?`. Native `inspect` tính natal tại birth đã cung cấp; geometry, `aspects.between` và `houses.locate` dùng trực tiếp số/vị trí/cusps đầu vào. Mỗi call độc lập; SDK chưa có cache chart hoặc session giữ chart ngầm.

```js
const { geometry, aspects, houses, points } = require('@7mlabs/astrology');

const position = geometry.normalize({ longitude: -10 });
console.log(position.data.longitude); // 350

const contact = aspects.between({
  positions: [{ id: 'sun', longitude: 0, speed: 1 },
              { id: 'moon', longitude: 91, speed: 0.5 }],
  rule: { angle: 90, maxOrb: 8 },
  motionMode: 'relative'
});

const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  houseSystem: 'wholeSign'
};
const selectedPoints = points.inspect({ birth, pointIds: ['sun', 'ascendant', 'H1'] });
const selectedHouses = houses.inspect({ birth, houseNumbers: [1, 7] });
```

Node/Python grouped methods và .NET `Engine.Geometry`/`Engine.Aspects`/`Engine.Houses`/`Engine.Points` là wrapper tiện dụng. Rust và C dùng cùng request JSON:

```json
{
  "operation": "query",
  "group": "geometry",
  "action": "separation",
  "longitude1": 350,
  "longitude2": 10
}
```

`calculateJson` trả error envelope; Node `calculate` và grouped wrappers throw error có `code`/`result` khi core báo lỗi. Không thay lỗi bằng dữ liệu mẫu. Các operation tạo chart đầy đủ, cặp đôi, composite và lịch vẫn dùng API chung `calculate`/`calculateJson`. Contract: [query request](../schemas/query-request.schema.json), [query response](../schemas/query-response.schema.json).

## MCP local qua stdio

Adapter mẫu ở [examples/mcp](../examples/mcp/README.md) dùng official `@modelcontextprotocol/sdk` **1.29.0**, tách khỏi package tính toán. MCP host khởi chạy một process Node và trao đổi stdin/stdout; host sở hữu vòng đời process. Đây là transport local chuẩn của MCP và không cần HTTP endpoint hay daemon chạy thường trực. [MCP stdio specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).

Adapter có năm tools:

| Tool | Arguments | Native request |
|---|---|---|
| `astro_geometry` | `action` và arguments của geometry | Inject `operation: "query"`, `group: "geometry"` |
| `astro_aspects` | `action` và arguments của aspects | Inject group `aspects` |
| `astro_houses` | `action` và arguments của houses | Inject group `houses` |
| `astro_points` | `action: "inspect"` và arguments của points | Inject group `points` |
| `astro_calculate` | `requestJson` là JSON string | Forward nguyên chuỗi tới native `calculateJson` |

Schema của bốn group tools được tạo trực tiếp từ query request schema: lọc group, bỏ routing keys do adapter sở hữu, giữ `action` và validation của từng operation. Client không được override `operation` hoặc `group`. Tool tổng quát nhận JSON string để giữ nguyên numeric literals và hỗ trợ mọi operation; native vẫn kiểm tra toàn bộ payload.

Mỗi tool trả cả `structuredContent` chứa nguyên engine envelope và text chứa cùng JSON. Core errors được giữ nguyên trong `.errors`, `.data: null`, đồng thời đánh dấu MCP `isError: true`. Unknown tool hoặc arguments sai schema là MCP protocol errors. Schema output chung mô tả envelope; response của bốn query tools còn được adapter kiểm tra bằng canonical query response schema. [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [official TypeScript SDK v1](https://ts.sdk.modelcontextprotocol.io/server).

Các annotations là `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: false`. Adapter chỉ tính toán từ input, không đọc birth data từ tài khoản bên ngoài, không gọi mạng, không lưu hồ sơ và không cần credentials. stdout chỉ chứa MCP protocol; log dùng stderr. Không đưa MCP SDK vào dependency bắt buộc của native engine.

Cách cài local tarball, lệnh launch và cấu hình host xem [MCP README](../examples/mcp/README.md). SDK MCP được pin theo phiên bản đã kiểm tra; ví dụ này dùng API v1, không trộn imports của SDK v2 split packages. Nếu publish adapter thành package riêng, cần bundle canonical schemas, CLI entry point và platform binary dependency, thêm matrix test trên các OS trước khi phát hành.

## Frontend chạy được ở đâu?

| Môi trường UI | Cách gọi tính toán | Hiện trạng |
|---|---|---|
| Desktop Electron có React/Vue/HTML | Renderer → IPC → Node host/worker → native SDK | Có [worker host mẫu](../examples/frontend/README.md); cần đóng gói và kiểm thử Electron riêng |
| Desktop Tauri/webview | UI → host command → Rust/C ABI | Có core Rust/C để tích hợp; chưa có app Tauri mẫu được kiểm thử |
| Ứng dụng .NET hoặc Python có giao diện | Event handler/worker → SDK local | SDK có thể dùng trong host; chưa có app giao diện mẫu |
| React/Vue trong browser thông thường | WASM chạy trong Web Worker | **Chưa có package/provider WASM**; không import Node addon trực tiếp vào browser |
| Website xem JSON đã tính | Import JSON → render chart/table/report | Không cần native runtime cho việc hiển thị; phép tính được tạo ở host/CLI khác |
| Website gọi một backend của dev | Browser → API riêng → SDK trong backend | Có thể tích hợp nhưng dev phải vận hành backend; đây là lựa chọn của ứng dụng |

React/Vue là renderer dữ liệu, không quyết định runtime tính toán. Node addon yêu cầu Node/native host; các cài đặt Python/.NET/Rust/C cũng cần runtime tương ứng. Bundler frontend hoặc browser Web Worker không tự biến native addon thành WASM. Mục tiêu web app tĩnh offline cần port riêng provider/core, loader/data, kiểm thử native–browser parity; xem [frontend roadmap](frontend.md).

Một bridge desktop chỉ cần nhận request JSON và trả nguyên response JSON. Main host đăng ký channel chuyên biệt; preload cung cấp đúng hàm tính toán cho renderer. Electron hướng dẫn dùng `ipcRenderer.invoke`/`ipcMain.handle` qua `contextBridge` cho luồng hai chiều. [Electron IPC](https://www.electronjs.org/docs/latest/tutorial/ipc), [contextBridge](https://www.electronjs.org/docs/latest/api/context-bridge).

```js
// Electron main: phần tích hợp vào một app đã có BrowserWindow.
const { ipcMain } = require('electron');
const { createEngineBridge } = require('./engine-host/bridge.cjs');
const bridge = createEngineBridge(); // copy bridge.cjs + worker.cjs từ examples/frontend
ipcMain.handle('astrology:calculate', (_event, requestJson) => {
  if (typeof requestJson !== 'string') throw new TypeError('Expected request JSON');
  return bridge.asyncCalculate(requestJson);
});

// preload: giữ contextIsolation và expose đúng API của ứng dụng.
const { contextBridge, ipcRenderer } = require('electron');
contextBridge.exposeInMainWorld('astrology', {
  calculate: request => ipcRenderer.invoke('astrology:calculate', JSON.stringify(request))
});

// React/Vue/HTML renderer: không import native package trong browser bundle.
const result = await window.astrology.calculate({
  operation: 'query', group: 'geometry', action: 'normalize', longitude: -10
});
if (result.errors.length) {
  showErrors(result.errors);
} else {
  renderPosition(result.data);
}
```

Đây là đoạn hướng dẫn IPC, chưa phải Electron app đã đóng gói/test. [Worker host mẫu](../examples/frontend/README.md) có `asyncCalculate(requestOrRawJson)` và `close()`, chạy native SDK trong một worker persistent; các request concurrent được queued và trả đúng caller. Core error envelope vẫn resolve; serialize/worker/host errors reject Promise. Host gọi `close()` khi đóng ứng dụng. Chỉ truyền dữ liệu JSON qua bridge; không expose toàn bộ `ipcRenderer` hoặc Node APIs. Timeout/cancel request ở frontend không tự ngắt native synchronous call đang chạy; mẫu chỉ terminate cả worker khi close, chưa có pool hoặc cancellation từng job.

## UI nên lấy dữ liệu nào?

Renderer mẫu đã được đóng gói tùy chọn thành **`@7mlabs/astrology-ui` `0.9.0-alpha.1`**: ESM browser modules, stylesheet, không có native dependency. Dev cài tarball alpha rồi import `createAstroUI`/`mountAstroResult` và `@7mlabs/astrology-ui/styles.css`; truyền `calculate: request => bridge.asyncCalculate(request)`. Host Rust/C/Python/.NET cũng có thể trả cùng JSON envelope qua bridge của ứng dụng để dùng renderer này. Cài đặt/API ở [UI package README](../examples/frontend/ui/README.md); standalone local demo và handler nhúng ở [frontend example](../examples/frontend/README.md). UI package chỉ hiển thị/select dữ liệu; native SDK/host vẫn là nơi tính toán.

Dùng query nhỏ cho input preview, tìm điểm/nhà và kiểm tra contact cụ thể. Dùng payload chart/report đầy đủ cho vòng natal, synastry/composite, bảng aspect, các tab lĩnh vực, section evidence và JSON inspector. Dùng forecast/events cho calendar/timeline và precision của event. Mỗi renderer join theo namespace/context của payload: natal facts, A/B cross contacts, chart C, snapshot và exact-event indexes không thay thế nhau.

UI giữ `schemaVersion`, `engineVersion`, calculation options, warnings và errors trong export/report. Form chuyển thời gian địa phương thành UTC trước khi gọi engine; tra địa điểm/timezone là chức năng ứng dụng riêng. Nội dung diễn giải, template báo cáo, storage, authentication và đồng bộ không thuộc core. Frontend chi tiết được định hướng ở [frontend.md](frontend.md).

## Xác minh adapter

`examples/mcp/smoke.cjs` dùng official MCP Client để spawn adapter qua stdio, initialize, list tools, gọi đủ tám group actions, so toàn bộ structured/text result với installed native package, kiểm tra lỗi calendar/đối đỉnh/raw integer và thử generic natal/couple/forecast. Nó cũng kiểm tra việc từ chối routing override và unknown tool. Đã chạy với package `0.9.0-alpha.1` cài mới trong consumer `64430cbbc44e`: **5 tools / 16 MCP calls đạt**, dùng official SDK 1.29.0 và Ajv 8.20.0 có sẵn trên máy. `examples/frontend/smoke.cjs` cũng đạt **11 checks**, gồm **4 calls concurrent/queued**, raw JSON/errors và terminate worker. Cả hai chạy trên macOS ARM64 / Node.js 24.19.0; đây là kiểm thử native stdio/Node worker host, không phải browser WASM hoặc Electron UI. Lệnh và runtime setup được ghi trong README của từng ví dụ.
