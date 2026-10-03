# @7mlabs/astrology-ui

UI độc lập framework cho payload của `@7mlabs/astrology` `0.9.0-alpha.1`. Package ESM này không chứa native engine, không gọi dịch vụ bên ngoài và không yêu cầu React/Vue. Host truyền hàm `calculate` hoặc một engine envelope đã tính. npm tarball hiện là artifact alpha cục bộ, chưa publish registry.

```sh
npm install /absolute/path/7mlabs-astrology-ui-0.9.0-alpha.1.tgz
```

Với bundler hỗ trợ CSS imports:

```js
import { createAstroUI, mountAstroResult } from '@7mlabs/astrology-ui';
import '@7mlabs/astrology-ui/styles.css';

const ui = createAstroUI({
  element: document.querySelector('#result'),
  // Electron preload, Tauri command, or your application's JSON bridge:
  calculate: request => host.asyncCalculate(request)
});
await ui.calculateRequest({
  operation: 'query', group: 'geometry', action: 'normalize', longitude: -10
});
// Dispose the viewer; the host owns worker/engine shutdown.
ui.close();
```

Browser không import native addon. Node/Electron host, Rust/Tauri, Python hoặc .NET có thể tính payload rồi đưa cùng envelope cho widget. Web frontend cần một host bridge hoặc bản engine WASM; engine WASM chưa được cung cấp. Ví dụ HTTP loopback trong thư mục cha là một lựa chọn chạy local cho demo, không là server bắt buộc của engine.

Nếu đã có kết quả:

```js
const view = mountAstroResult(element, envelope, {
  initialTab: 'chart',
  aspectLimit: 40,
  jsonPreviewLength: 24000,
  onDownload: originalEnvelope => downloadJSON(originalEnvelope)
});
view.selectTab('planets');
view.destroy();
```

## API

- `createAstroUI({element, calculate, ...options})`: `calculate` là async function nhận object hoặc raw JSON string, trả **nguyên engine envelope**. `calculateRequest(request)` chuyển nguyên request cho bridge và return envelope; `render(envelope)` render dữ liệu đã tính; `close()` giải phóng DOM. Nếu nhiều Promise cạnh tranh, chỉ response mới nhất được render. Host errors reject; core errors vẫn resolve và hiển thị mã/message thực.
- `mountAstroResult(element, envelope, options?)`: return controller `destroy()`, `selectChart(key)`, `selectDomain(id, sectionId?)`, `selectTab(key)`, `getState()`.
- Options: `initialTab`, `initialChart`, `initialDomain`, `aspectLimit` (mặc định 40 đường body–body trên vòng), `jsonPreviewLength` (mặc định 24.000 ký tự), `onDownload(envelope)` (optional).
- Tab keys: `chart`, `planets`, `houses`, `aspects`, `domains`, `events`, `json`. Tabs chỉ hiện khi payload có dữ liệu tương ứng. Chart keys tùy payload: `natal`, `synastry`, `A`, `B`, `C`, `transits`, `N`, `T`, `snapshot`, `query`.
- `@7mlabs/astrology-ui/data` export các adapters `getAstroViews`, `getDomainFacts`, `resolveLocalAdvanced` và helpers format. Đây là phép đọc/chọn dữ liệu, không tính vị trí thiên văn.
- Stylesheet export: `@7mlabs/astrology-ui/styles.css`; static server có thể copy cùng 5 module files và dùng `<link rel="stylesheet" href="/astro-ui/styles.css">`, import từ `/astro-ui/index.mjs`.

## Dữ liệu được hiển thị

Vòng SVG dùng kinh độ/cusp thực, đặt ASC bên trái khi có. Nhãn hành tinh có leader line để tránh va chạm; marker giữ kinh độ gốc. Vòng giới hạn đường body–body cho dễ đọc; bảng giữ toàn bộ contacts, gồm cusp/góc trục khi context cung cấp. Bảng dài render 80 hàng một lượt, nút Xem thêm tăng số hàng; JSON và evidence chỉ stringify khi mở, preview rút gọn được ghi rõ. Caller giữ toàn bộ payload và quyết định tải file.

- Natal: vòng, 10 hành tinh, góc trục, nhà và aspects.
- Natal domains: selectors/tiểu mục, nhà trọng tâm, trạng thái hành tinh, contacts và tham chiếu chuỗi chủ tinh/receptions/patterns.
- Couple: hai lớp A/B, riêng natal A/B, domain evidence có namespace, directional overlays và ruler links. Advanced facts resolve về **natal của từng subject**.
- Composite: chart C, provenance nằm nguyên trong envelope; speed/Rx/applying null hiển thị `—`.
- Forecast: natal N/transit T tại snapshot; sự kiện, overview và domain references. Contacts của snapshot và contacts tại event-time không bị ghép indexes với nhau.
- Events: snapshot bầu trời và danh sách sự kiện kèm thông số/độ chính xác thực. Không tạo nhà từ event snapshot.
- Query và supplied-position chart/harmonic/synastry: giá trị/hình học được engine cung cấp. Không tạo góc trục/nhà còn thiếu, không sinh điểm số hay báo cáo văn bản.

CSS scope `.astro-result` và kế thừa tokens `--bg`, `--surface`, `--text`, `--muted`, `--line`, `--accent`, `--font`, `--mono` khi host có. Tất cả labels/IDs/errors được đưa vào text nodes, không dùng `innerHTML`.

Xem `../README.md` để chạy standalone host và kiểm tra package/native bridge.
