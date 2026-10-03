# Astro UI và native runtime local

Frontend này có thể dùng độc lập hoặc nhúng vào website. Browser gửi JSON cho **Node worker local**, worker gọi SDK native đã cài và trả nguyên engine envelope. Cả 10 operation hoạt động với dữ liệu người dùng: `chart`, `harmonic`, `synastry`, `natal`, `natalDomains`, `couple`, `composite`, `events`, `forecast`, `query`.

Không cần backend hosted. Host chỉ lắng nghe `127.0.0.1`, chạy khi bạn mở ứng dụng và dừng bằng `Ctrl+C`. Bản website static có thể xem tài liệu/payload mẫu; muốn tính input mới bằng native engine, mở UI từ host local. Đây chưa phải browser WASM.

## Chạy ngay trong checkout

Yêu cầu Node.js 18+; môi trường đã kiểm tra dùng Node.js 24.19.0 / macOS ARM64. Từ thư mục **astrology/examples/frontend**:

```sh
node host.cjs
```

Mở **http://127.0.0.1:4174**. Đổi port bằng `PORT=4175 node host.cjs`. Lệnh không tải package hoặc khởi động daemon ngầm.

Host ưu tiên `ASTRO_ENGINE_MODULE`, sau đó package `@7mlabs/astrology` đã cài. Trong checkout có artifacts, host tìm consumer install tương ứng manifest hiện tại: kiểm tra platform/arch, SHA256 artifact và npm SHA512 integrity. Không dùng hash thư mục cố định, source binding hoặc package chưa cài. Nếu thiếu SDK, UI vẫn mở và báo runtime chưa kết nối.

Nếu `node` chưa nằm trong `PATH`, dùng đường dẫn đầy đủ tới executable Node. Với runtime đã có trong môi trường Codex của checkout này:

```sh
export ASTRO_NODE_BIN="$HOME/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
"$ASTRO_NODE_BIN" host.cjs
```

Chạy lệnh trong `astrology/examples/frontend`; không chạy `host.cjs` từ thư mục website. Đường dẫn workspace hiện tại có dấu cách ở cuối tên thư mục cha, vì vậy luôn đặt path trong dấu nháy khi `cd`.

## Cài SDK khi dùng frontend riêng

Với Node/npm đã cài, từ thư mục frontend:

```sh
npm install --offline --ignore-scripts --no-audit --no-fund ../../artifacts/packages/7mlabs-astrology-0.9.0-alpha.1.tgz
npm start
```

Artifact alpha còn local, chưa publish registry. Cần binary đúng platform/architecture. Nếu SDK đã cài ở nơi khác:

```sh
export ASTRO_ENGINE_MODULE="/absolute/path/to/node_modules/@7mlabs/astrology"
node host.cjs
```

## HTTP adapter để ghép với một local website

`local-http.cjs` không tự mở port. Nó trả handler để gắn vào server hiện có:

```js
const { createLocalEngineHost } = require('./local-http.cjs');
const engine = createLocalEngineHost({
  // engineModule: '/absolute/path/to/installed/package',
  // engineDirectory: '/absolute/path/to/astrology',
  // baseDirectory: '/absolute/path/to/website',
  maxPendingRequests: 8
});

// Trong request callback của server:
if (await engine.handle(req, res)) return;
// Tiếp tục phục vụ static assets ở đây.

// Khi đóng ứng dụng/server:
await engine.close();
```

Server ghép cần lắng nghe `127.0.0.1`. Handler chỉ nhận loopback Host đúng port hiện tại, Origin cùng trang local và Fetch Metadata phù hợp. Không bật CORS cho website khác. `GET /api/astro/status` thực hiện probe `query.geometry.normalize` thật; response có `available`, `engineVersion`, `runtime`, `operations`, giới hạn input và số request đang xử lý. SDK thiếu/bị lỗi trả status 503.

`POST /api/astro/calculate` nhận `Content-Type: application/json`, body là **raw engine request**, không thêm wrapper `{request:...}`. Giới hạn 4 MiB và tối đa 8 request pending, gồm cả request đang đọc body. Hết chỗ trả 429; body quá lớn trả 413; sai Origin/Host trả 403. Không lưu birth input/payload, không gửi dữ liệu ra mạng.

Response 200 là nguyên envelope: `schemaVersion`, `engineVersion`, `calculation` khi thành công, `data`, `warnings`, `errors`. Engine validation errors cũng trả 200 với `data: null`; transport/runtime errors trả non-2xx và `{error:{code,message}}`. Không biến payload mẫu thành kết quả tính thực.

Raw string giữ nguyên literal cho exact integer normalization, ví dụ `2000.0000000000000001` bị Rust từ chối đúng. Nếu caller truyền object JavaScript thì `JSON.stringify` xảy ra trước HTTP, vì vậy dùng raw string khi cần kiểm tra literal số có nhiều chữ số.

## Nhúng widget browser

Module UI chỉ dùng DOM và hàm tính được inject; không import addon native:

```js
import { createAstroUI } from './ui/index.mjs';

createAstroUI({
  element: document.querySelector('#astro'),
  calculate: async input => {
    const response = await fetch('/api/astro/calculate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: typeof input === 'string' ? input : JSON.stringify(input)
    });
    const result = await response.json();
    if (!response.ok) throw new Error(result.error?.message || 'Runtime unavailable');
    return result;
  }
});
```

Nạp `ui/styles.css`. Standalone `ui/example.html`/`example.mjs` ghép widget với cùng API này. Widget có thể nhận adapter desktop IPC khác; website marketing không phải dependency runtime.

Widget cũng đã đóng gói độc lập thành **`@7mlabs/astrology-ui` `0.9.0-alpha.1`**. Không có native dependency trong UI package; host vẫn inject `calculate` trả nguyên engine envelope.

```sh
npm install /absolute/path/7mlabs-astrology-ui-0.9.0-alpha.1.tgz
```

```js
import { createAstroUI, mountAstroResult } from '@7mlabs/astrology-ui';
import '@7mlabs/astrology-ui/styles.css';

const ui = createAstroUI({ element, calculate: request => host.asyncCalculate(request) });
await ui.calculateRequest(request);
ui.close();
// Nếu đã tính ở host/CLI khác: mountAstroResult(element, envelope).
```

Tarball alpha nằm trong `artifacts/packages/7mlabs-astrology-ui-0.9.0-alpha.1.tgz`; SHA-256 và exports ở manifest riêng `artifacts/ui/manifest.json`. Không thêm UI vào manifest của native engine. [UI package README](ui/README.md) ghi đầy đủ API, chart/tab keys, CSS tokens và field mapping. Package chứa 5 module/stylesheet files, README và package metadata, không chứa standalone server, SDK binary hoặc request fixtures.

UI có vòng SVG/bảng hành tinh/nhà/góc chiếu, natal A/B/composite C, domain selectors và section evidence, forecast/events overview, scalar queries và JSON preview. Vị trí/cusp/contacts lấy từ payload; `null` không bị chuyển thành motion/applying giả. JSON/evidence mở theo yêu cầu, preview có giới hạn rõ ràng; payload gốc giữ nguyên cho export. Form standalone có presets cho đủ 10 operation và chuyển raw JSON trực tiếp tới native worker.

## Dùng bridge trong desktop host

```js
const { createEngineBridge } = require('./bridge.cjs');
const bridge = createEngineBridge({ engineModule: '/absolute/path/to/installed/package' });
const result = await bridge.asyncCalculate({
  operation: 'query', group: 'geometry', action: 'normalize', longitude: -10
});
console.log(result.data.longitude); // 350
await bridge.close();
```

`asyncCalculate` nhận object serializable hoặc raw JSON string. Core errors resolve nguyên envelope; lỗi serialize/worker/host reject Promise. Một worker persistent xử lý lần lượt, nên native synchronous call không khóa main/UI. `close()` terminate worker và reject các request còn pending; chưa có cancel riêng cho từng native job.

## Kiểm tra thật

```sh
node smoke.cjs
node http-smoke.cjs
node ui-smoke.mjs
# Hoặc npm test sau khi Node/npm và SDK đã cài.
```

Bridge smoke: 11 checks, gồm 4 requests concurrent/queued. HTTP smoke: 44 checks, gọi trực tiếp cả 10 operation qua SDK đã cài, so sánh nguyên payload với SDK; kiểm tra HTML/ESM/CSS thật, raw numeric literal, malformed JSON, midpoint error, queue, UTF-8, Host/Origin/Fetch Metadata, methods, input limits, SDK thiếu và path traversal. Đã chạy với engine `0.9.0-alpha.1` cài từ artifact local, không dùng source binding.

UI payload adapter smoke: **11 checks đạt**, chạy cả source module và UI package đã cài bằng npm từ tarball. Native browser QA: **26 checks đạt** ở `http://127.0.0.1:4174`, viewports 1536×1024 và 390×844: đủ 10 presets, tabs, A/B/C, domain/tiểu mục, forecast snapshot contacts, sự kiện, scalar query, error envelope và không overflow mobile. Chrome console không có app errors. Test consumer DOM riêng sau npm install: **11 checks đạt**, gồm actual native calculation, raw numeric literal rejection, keyboard tab navigation, truncated JSON giữ nguyên full data, subject-local evidence, text-only arbitrary IDs và dispose. UI package `0.9.0-alpha.1` không có dependencies; browser QA dùng Playwright 1.62.1 + Chrome cài sẵn do IAB không có trong phiên subagent. Chưa kiểm thử Safari/Firefox hoặc đóng gói app Electron/Tauri.

WASM cần build cả Rust core và Swiss C cho browser, chọn C runtime/headers, wrapper ABI và kiểm thử parity. Checkout hiện không có target/toolchain/artifact browser đó. Local worker host là runtime đang hoạt động; browser WASM là bước phát triển riêng.
