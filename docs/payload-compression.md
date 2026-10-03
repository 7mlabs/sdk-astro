# Chuẩn bị payload cho LLM

Từ **0.10.0-alpha.1**, engine có bộ nén riêng, chạy offline trong Rust core. Bộ nén nhận **engine response envelope đã tính**, tạo context JSON có version, giữ bảng tham chiếu và công bố phạm vi dữ liệu còn lại. Không cần server hoặc LLM để nén.

Đây là bước sau tính toán. Mười operation chiêm tinh hiện có vẫn là `chart`, `harmonic`, `synastry`, `natal`, `natalDomains`, `couple`, `composite`, `events`, `forecast`, `query`. `calculate()` vẫn trả payload gốc; compression không trở thành operation tính toán thứ 11 và không thay đổi kết quả thiên văn.

## Ba cách dùng

| Mode | Dữ liệu giữ lại | Khi dùng |
| --- | --- | --- |
| `compact` — mặc định | Toàn bộ giá trị JSON của envelope gốc; thay cách biểu diễn để giảm lặp | LLM cần toàn bộ kết quả hoặc lưu context có thể khôi phục |
| `focused` | Domain/section yêu cầu cùng toàn bộ dữ kiện hỗ trợ đã có trong payload | Báo cáo một khía cạnh cụ thể |
| `budgeted` | Cùng quy tắc chọn phạm vi; thêm kiểm tra ngân sách | Ứng dụng cần biết context có vượt giới hạn không |

`compact` không chấp nhận selector `domains`/`sections` khác rỗng. `focused`/`budgeted` không có selector vẫn giữ toàn bộ dữ liệu. Budget không tự loại bỏ fact tùy tiện để ép vừa giới hạn. Nếu phạm vi được chọn vẫn quá lớn, engine giữ phạm vi đó, đặt `budget.exceeded: true` và warning `COMPRESSION_BUDGET_EXCEEDED`. Ứng dụng có thể chọn domain/section hẹp hơn hoặc tách thành nhiều lượt; luôn kiểm tra trường này trước khi gửi đến LLM.

Nén biểu diễn không bảo đảm payload nhỏ hơn ở mọi input: natal cơ bản hoặc query ngắn có thể bị metadata làm lớn hơn. Đo `metrics` trên payload thực tế; không suy ra tỷ lệ giảm token từ một payload khác.

## Node.js

```js
const {
  calculate, compressPayload, compressPayloadJson,
  expandContext, calculateWithContext
} = require('@7mlabs/astrology');

const request = {
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['career', 'love']
};

const result = calculate(request);
const context = compressPayload(result); // compact, không lược dữ liệu
const restored = expandContext(context); // JSON-value equality với result

const combined = calculateWithContext(request, {
  mode: 'budgeted', domains: ['career'], maxBytes: 64 * 1024
});
// combined.result: payload gốc, dùng cho UI/audit
// combined.context: context được chuẩn bị để đưa vào prompt
if (combined.context.data.budget.exceeded) {
  // Chọn phạm vi hẹp hơn hoặc chia lượt trước khi gửi.
}

// Raw API giữ JSON string qua ranh giới native, trả envelope lỗi thay vì throw.
const rawContext = compressPayloadJson(JSON.stringify(result),
  '{"mode":"focused","domains":["career"]}');
```

`expandContext()` nhận **toàn bộ context envelope**, không nhận riêng `data.payload`. Khi compression thành công, `compact` khôi phục cùng giá trị JSON; thứ tự object keys và whitespace không được bảo toàn. Với `focused`/`budgeted`, chỉ khôi phục envelope chứa dữ liệu đã giữ lại. Không thể khôi phục những phần nằm trong `omitted`. Input có thể bị từ chối nếu độ sâu sau encoding vượt giới hạn, kể cả khi raw input ban đầu vẫn parse được.

Mẫu có assertions: [examples/node/compression.cjs](../examples/node/compression.cjs).

## Python

```python
from sevenmlabs_astrology import (
    calculate, compress_payload, compress_payload_json,
    expand_context, calculate_with_context,
)

result = calculate(request)
context = compress_payload(result, {"mode": "compact"})
assert expand_context(context) == result

combined = calculate_with_context(request, {
    "mode": "focused", "domains": ["career"],
})
original = combined["result"]
context = combined["context"]
raw_context = compress_payload_json(raw_engine_response, '{"mode":"compact"}')
```

Request giống ví dụ Node ở trên, dùng Python dict. Chạy [examples/python/compression.py](../examples/python/compression.py) sau khi cài wheel phù hợp platform.

## .NET, Rust và C

Các SDK đều gọi chung Rust core, không tự triển khai một thuật toán nén khác. .NET có các API `Engine.CompressPayload<T>(payload, options)`, `Engine.CompressPayloadJson(payloadJson, optionsJson)`, `Engine.ExpandContext<T>(context)`, `Engine.ExpandContextJson(contextJson)` và `Engine.CalculateWithContext<T>(request, options)`. Object options dùng tên JSON `mode`, `domains`, `sections`, `maxBytes`, `maxTokens`, giống Node/Python. Helper cuối trả record `CalculationWithContext` với `Result`/`Context` là `JsonElement`. Xem [bindings/dotnet/Engine.cs](../bindings/dotnet/Engine.cs) để chọn overload phù hợp.

Rust dùng `astro_core::compress_json()` với JSON wrapper sau:

```json
{
  "payload": {
    "schemaVersion": "1.0", "engineVersion": "0.10.0-alpha.1",
    "data": {}, "warnings": [], "errors": []
  },
  "options": { "mode": "compact" }
}
```

`astro_core::expand_context_json()` nhận context envelope và trả retained engine envelope. C ABI bổ sung `astro_compress_json` và `astro_expand_context_json`; gọi `astro_free_string` đúng một lần để giải phóng string trả về. ABI hiện hành vẫn tương thích các hàm tính toán trước đó. Các hàm raw dùng engine error envelope để báo lỗi native; SDK vẫn có thể throw khi sai kiểu tham số, chứa NUL hoặc vượt giới hạn đầu vào.

## Options và phạm vi

| Field | Ý nghĩa |
| --- | --- |
| `mode` | `compact`, `focused`, `budgeted`; mặc định `compact` |
| `domains` | Mảng ID domain/custom domain đã thực sự tồn tại trong payload |
| `sections` | Mảng section ID đã thực sự tồn tại trong những domain được giữ lại |
| `maxBytes` | Số nguyên từ 1 đến 268435456, kiểm tra tổng byte UTF-8 của **native context envelope dạng minified** |
| `maxTokens` | Số nguyên từ 1 đến 268435456, kiểm tra `estimatedTokens` theo phương pháp khai báo dưới đây |

Selector không khớp gây `COMPRESSION_SELECTOR_INVALID`, không tạo context trống và không dùng heuristic đoán keyword. Các ID cho natal/composite/forecast có thể gồm `career`, `love`, `relationships`, `family`, `finance`, `identity`, `learning`, `creativity`, `innerLife`, `dailyLife`. Couple dùng các ID của profile couple, ví dụ `communication`; custom domain dùng ID do request tạo ra. Đọc `profileCatalog`, `domains`, `customDomains` và `report.sections` của kết quả gốc để biết selector hợp lệ.

Selector domain được áp dụng vào các container domain thực sự tồn tại, kể cả composite lồng trong couple. Cùng ID có thể xuất hiện ở nhiều container; engine chọn từng container riêng và giữ nguyên namespace. Section ID chỉ chọn mục báo cáo; parent report và những dữ kiện hỗ trợ vẫn được giữ đầy đủ. Khi chỉ chọn section mà không chọn domain, section được lọc ở các domain đã có; section không xuất hiện trong một domain có thể khiến danh sách sections của domain đó rỗng và được ghi rõ trong omissions.

Phiên bản này chủ động giữ toàn bộ shared `context`, natal/subjects, snapshots và events khi chọn domain/section. Điều này tránh làm đứt `aspectIndexes`, `houseRulerRelations`, dispositor chains, receptions, pattern references hoặc nhầm dữ liệu của A/B/composite. Domain khác và phần tử `profileCatalog` không liên quan có thể bị bỏ, nhưng không xóa hàng trong global context khiến index thay nghĩa. Cơ chế này bảo toàn dữ kiện hỗ trợ, đổi lại mức giảm kích thước khi chọn section có thể hạn chế.

## Context response

Response là engine envelope, `data.format` bằng `astro-context/1`:

```json
{
  "schemaVersion": "1.0",
  "engineVersion": "0.10.0-alpha.1",
  "data": {
    "format": "astro-context/1",
    "mode": "focused",
    "payload": {},
    "dictionary": {},
    "coverage": {
      "complete": false,
      "retainedPaths": ["", "/data/domains/career"],
      "scope": {"domains": ["career"], "sections": []}
    },
    "omitted": [{"path": "/data/domains/love", "reason": "domainOutsideScope"}],
    "metrics": {
      "inputBytes": 100000,
      "outputBytes": 50000,
      "estimatedTokens": 50000,
      "tokenEstimateMethod": "utf8-bytes-conservative"
    },
    "budget": {"maxBytes": null, "maxTokens": null, "exceeded": false}
  },
  "warnings": [],
  "errors": []
}
```

Ví dụ trên minh họa shape, các số không phải benchmark. `payload` là retained source envelope đã encode; không phải object chart trần. Source `schemaVersion`, `engineVersion`, `calculation`, `warnings` và `errors` nằm trong envelope đó và không bị xóa. Outer envelope mô tả bước compression; một input chứa lỗi tính toán có thể được nén thành công và vẫn giữ nguyên lỗi tính toán bên trong. `coverage.complete` bằng `true` khi không có semantic omission; `false` nghĩa là có phần bị loại, kể cả khi phạm vi được yêu cầu vẫn đủ dữ kiện hỗ trợ.

`retainedPaths` dùng JSON Pointer. Chuỗi rỗng `""` chỉ root envelope, **không có nghĩa toàn bộ source chưa bị lược**; luôn đọc cùng `omitted`. Reason hiện tại: `domainOutsideScope`, `sectionOutsideScope`, `profileOutsideScope`.

## Cách đọc encoding

Ba marker có nghĩa theo format `astro-context/1`:

1. `{"$astroTable":{"columns":["id","longitude","applying"],"rows":[["sun",12.345,null],["moon",34.567,false]]}}` tương đương mảng hai object, ghép mỗi ô theo vị trí của `columns`. `null` vẫn là unknown, khác `false`.
2. `{"$astroRef":"d1"}` trỏ đến encoded value ở `dictionary.d1`. Không coi `d1` là point ID hoặc index chiêm tinh. Lookup lại dictionary trước khi đọc fact. Những object trùng hoàn toàn có thể chia sẻ biểu diễn nhưng không hợp nhất namespace hoặc nghĩa của point/house.
3. `{"$astroLiteral":[["$astroRef","literal source value"],["otherKey",42]]}` phục hồi original object có reserved key; marker ban đầu của source không bị hiểu nhầm thành reference.

Engine không làm tròn góc/orb/speed, không thay `null` bằng trạng thái suy đoán, không thêm diễn giải và không tính lại chart. SDK expansion cũng dùng native core. Send **cả context envelope** để LLM thấy dictionary, coverage, omitted và warnings. Gửi riêng `payload` sẽ thiếu nội dung tham chiếu.

Có thể thêm instruction này vào prompt:

> Read astro-context/1 by resolving $astroRef against dictionary and reconstructing $astroTable rows from columns. $astroLiteral preserves original object keys. Preserve source versions, warnings, null/unknown values, and subject/snapshot reference scopes. State when omitted data prevents answering the question. budget.exceeded is a delivery limit warning, not permission to invent missing facts.

Bộ nén chỉ bảo đảm phép biểu diễn và phạm vi đã công bố; nó không chứng minh rằng mọi câu hỏi của LLM đều có đủ dữ kiện. Ứng dụng cần chọn selector theo câu hỏi và kiểm tra omissions trước khi viết báo cáo.

## Byte và token budget

`metrics.inputBytes` đếm byte UTF-8 của source envelope serialize dạng minified theo core, không tính whitespace của raw input hoặc wrapper options. `metrics.outputBytes` đếm byte UTF-8 của context envelope native serialize, gồm dictionary/coverage/metrics/budget/warnings. `estimatedTokens` hiện bằng số byte đó, với `tokenEstimateMethod: "utf8-bytes-conservative"`. Đây là ước lượng thận trọng cho text, **không phải tokenizer của một model cụ thể** và không có tỷ lệ “4 ký tự = 1 token” giả định.

Ngân sách không tính system prompt, chat history, template instruction, MCP protocol wrapper hoặc trường bị host lặp lại. Python/.NET/Node có thể serialize envelope theo whitespace/escaping khác; dùng raw API khi cần giữ đúng kích thước native đo. Khi tích hợp provider cụ thể, tokenize **prompt cuối cùng** với tokenizer tương ứng và chừa chỗ cho output. Không dùng `maxTokens` của compressor làm bảo đảm tổng token API request.

Compression/expansion có cap **256 MiB UTF-8**, tính toàn bộ request native. Context output và payload expand cũng có cap 256 MiB; nếu việc thêm metadata/escaping làm output quá lớn, core trả `COMPRESSION_OUTPUT_TOO_LARGE`. Nếu encoded JSON vượt giới hạn nesting của parser 128 container hoặc giải mã vượt 128 bước, compression trả `COMPRESSION_OUTPUT_TOO_DEEP` trước khi báo thành công. Reserved-key escaping có thể tăng độ sâu, vì vậy source parse được vẫn có thể bị từ chối ở bước này.

Decoder yêu cầu đầy đủ metadata của `astro-context/1`, kiểm tra cả dictionary entries chưa được dùng, thiếu refs/cycles và giới hạn độ sâu. Không nhận context bị cắt chỉ còn `format`, `payload`, `dictionary`. Đây là các cap khác giới hạn request tính toán **4 MiB**, vì response report/forecast có thể lớn hơn nhiều. Raw input/options phải là JSON hợp lệ, với số hữu hạn. Dùng object tương thích JSON khi gọi SDK; các giá trị như NaN, Infinity hoặc circular references có thể bị serializer từ chối hoặc đổi giá trị trước khi đến core.

## MCP

Adapter tùy chọn có hai tool thêm vào năm tool cũ:

- `astro_compress_payload`: `{ "payloadJson": "<complete engine envelope>", "options": {"mode":"focused","domains":["career"]} }`.
- `astro_calculate_context`: `{ "requestJson": "<engine request>", "options": {"mode":"compact"} }`.

Tool thứ hai chỉ trả **context envelope** khi tính toán thành công; không trả `{result, context}` kèm payload gốc làm tăng context của LLM. Nếu tính toán thất bại, tool trả nguyên native error envelope với `data: null`, `isError: true`, không bọc lỗi đó thành một context thành công. Host cần chart gốc cho UI thì dùng SDK helper hoặc gọi calculation riêng. MCP dùng cùng native compressor và [canonical response schema](../schemas/compression-response.schema.json); lỗi native được giữ và đánh dấu `isError`, warnings quá budget không tự chuyển thành protocol error.

Tool result có JSON text và `structuredContent` theo adapter MCP. Host nên đưa một representation vào prompt; byte metrics không bao gồm sự lặp lại do host hay transport. Xem [MCP README](../examples/mcp/README.md) và test [smoke-compression.cjs](../examples/mcp/smoke-compression.cjs).

## Validation và kiểm thử

Canonical schemas: [compression request](../schemas/compression-request.schema.json), [compression response](../schemas/compression-response.schema.json). Lỗi của bước chuẩn bị context dùng namespace `COMPRESSION_*`, gồm JSON/payload/options/selector không hợp lệ, input quá lớn hoặc context không thể decode. Budget vượt giới hạn được báo bằng warning và `budget.exceeded`; facts bắt buộc vẫn còn.

Kiểm tra quan trọng khi tích hợp: compact roundtrip giữ JSON values; focused giữ reference closure; reserved keys không va chạm; source errors/warnings/version không bị xóa; small budget báo vượt rõ ràng; tất cả SDK dùng cùng core. Không dùng số ký tự làm benchmark token thật.

Các CLI mẫu Rust/C/.NET nhận JSON qua stdin với `--compress-json` (wrapper `payload`/`options`) và `--expand-context-json` (context envelope). Chạy `scripts/test-compression.py` sau `scripts/test-packages.py` để kiểm tra 5 ngôn ngữ và 13 fixture thật. `scripts/check-compression-bindings.py` kiểm tra installed SDK helpers cùng TypeScript contracts. Kiểm tra schema bằng SDK đã cài và Ajv development dependency:

```sh
ASTRO_ENGINE_MODULE="/absolute/path/to/consumer/node_modules/@7mlabs/astrology" \
NODE_PATH="/absolute/path/to/development/node_modules" \
node scripts/check-compression-schema.cjs
```

Script trên dùng 13 file `examples/*-result.json` mặc định, xác thực response, roundtrip và sự đồng bộ validation options giữa schema/native core; có thể truyền đường dẫn response fixtures để chọn tập khác. Fixtures nguồn khác version vẫn giữ đúng version của nguồn bên trong context mới. Các fixture hiện lưu trong repo được tính lại bằng installed package `0.10.0-alpha.1`; response gốc và compression cùng version trong snapshot website.

## Kết quả đo trên fixture thật

Đã kiểm tra schema và compact roundtrip trên **13 fixture** bằng package `0.10.0-alpha.1` cài local, macOS ARM64. Bảng dưới đo tổng **byte UTF-8**, gồm context metadata/dictionary; không đo token bằng tokenizer model. Các fixture dùng options cụ thể trong repo, không đại diện mọi chart hoặc mọi cấu hình forecast.

| Fixture | Source minified bytes | Context bytes | Giảm byte |
| --- | ---: | ---: | ---: |
| Natal | 7,147 | 5,278 | 26.15% |
| Natal domains | 2,129,952 | 1,291,085 | 39.38% |
| Couple | 2,101,342 | 1,101,538 | 47.58% |
| Composite | 2,206,657 | 1,374,855 | 37.70% |
| Forecast ngày | 306,138 | 158,582 | 48.20% |
| Forecast tháng | 4,296,721 | 2,195,083 | 48.91% |
| Events năm | 1,264,416 | 1,054,986 | 16.56% |

Tất cả các hàng trên giữ đầy đủ JSON values khi expand. Schema/native checks còn kiểm tra 27 validation cases, collision markers, source errors, metadata thiếu và reserved-key encoding quá sâu. MCP test sử dụng official stdio client với 7 tools: 16 calls regression tính toán và 15 calls compression đạt. Kết quả runtime chỉ xác nhận nền tảng đã thử; package cho OS/architecture khác vẫn cần binary và CI tương ứng.
