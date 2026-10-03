# Kết quả cài đặt và kiểm thử trực tiếp

Bản mẫu được chạy trên macOS ARM64, Node.js 24.19.0, Python 3.13.3, .NET SDK 10.0.101 và Rust 1.83.0. Các test xác minh geometry từ positions, natal, báo cáo cá nhân, synastry, midpoint composite C, events và forecast cá nhân của bản `0.10.0-alpha.1`, gồm grouped queries và bộ nén payload; không xác minh toàn bộ source legacy.

## Bản npm đã phát hành: 0.10.0-alpha.1

[@7mlabs/astrology](https://www.npmjs.com/package/@7mlabs/astrology) đã phát hành public ngày 2026-10-03 với tag `alpha`. [CI run 37096125672](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672) kiểm tra commit `a17850e01507485312e5cb584ed1eb82a786fcbc` trên macOS 15 ARM64 và Ubuntu 24.04 x64:

- Mỗi candidate target qua **108 Rust tests**, **339 parity cases** giữa Node/Python/.NET/Rust/C, 106 compression checks, 13 real fixture roundtrips, SDK helpers/TypeScript và package license/checksum gates.
- Một tarball npm ghép cả hai native binaries được fresh-install offline vào consumer riêng trên **bốn tổ hợp** macOS ARM64/Linux x64 × Node **18/24**. Mỗi tổ hợp qua toàn bộ **339 cases** trong một process SDK, đối chiếu với Rust CLI mới, đủ 8 query helpers, query schema/semantic checks, 27 compression contract cases/13 roundtrips và TypeScript strict.
- Registry SHA512 integrity, legacy checksum và tag `alpha` trùng release đã kiểm tra. Fresh install từ npm trên macOS ARM64 xác minh native checksum rồi chạy natal, geometry và compression roundtrip; Linux registry smoke chưa chạy riêng, consumer CI dùng chính cùng tarball.

SHA256 `7mlabs-astrology-0.10.0-alpha.1.tgz`: `d8dcfdbaea4e66070a75f9e5ef91d0e6302b33995b364dce40b6a1e28eba9748`. npm trusted publisher đã cấu hình cho repository/workflow/environment tương ứng; lần phát hành này dùng tài khoản npm có 2FA. Chưa có lần publish thực tế bằng OIDC, nên không ghi cấu hình publisher là một test phát hành tự động đã qua. Python/.NET/Rust và UI renderer chưa phát hành registry.

Các fingerprints, counts và CI run cũ phía dưới là đối chứng của những đợt chạy được ghi rõ; không thay thế bằng chứng của tarball npm public này.

## Cài artifact thật

`scripts/build-packages.py` tạo tgz, wheel và nupkg chứa native binary. `scripts/test-packages.py` kiểm tra checksum, tạo consumer theo fingerprint artifact, rồi cài bằng npm offline, pip no-index và NuGet local feed. Mỗi fingerprint dùng cache riêng để không tái sử dụng nupkg cùng version từ lần build trước.

Node/Python/.NET samples chạy từ consumer, không import binding source và không cài compiler trong consumer. Rust sample dùng crate path local; C sample được compile và link native library. Đây là luồng kiểm tra artifacts local; cài npm public được kiểm chứng riêng ở phần bản phát hành phía trên.

## Compression: đối chứng trước lần phát hành npm

Ba artifacts native đã build và cài offline trên macOS ARM64 vào consumer `783f6cd94295`. Node/Python/.NET dùng binary trong package; Rust dùng path crate, C link native ABI. Đợt kiểm tra này diễn ra trước lần phát hành npm và trước các regression fixes bổ sung; các counts bên dưới thuộc fingerprint này.

- Rust workspace: **106 tests đạt** (80 core, 9 couple, 9 composite, 6 forecast, 1 FFI, 1 provider); 11 compression tests kiểm tra số thực, null, namespace, section evidence, escape collisions, budget decimal/exponent, malformed references và metadata/depth limits. Fmt/Clippy all-targets `-D warnings` đạt.
- Calculation regression: **339/339 cases** trả toàn bộ decoded JSON giống nhau qua Node/Python/.NET/Rust/C. Repeat/concurrent tests và bốn reference natal tiếp tục đạt.
- Compression: **106 checks** qua năm ngôn ngữ, compact roundtrip **13 payload thật**, focused scope giữ natal/shared context/subjects/events/snapshots; metrics tính đúng toàn bộ byte UTF-8 response. Byte giảm **16.56–51.03%** trên tập fixtures này; không phải benchmark token hoặc cam kết mọi input.
- SDK helper: Node/Python/.NET cài riêng cùng artifact trong consumer `783f6cd94295/helpers`, kiểm tra `calculateWithContext`, raw/object APIs, unsupported values, không mutate input, roundtrip và budget warning. TypeScript strict đạt; .NET kiểm tra object 60 tầng với parse/serializer depth phù hợp.
- Schema/native: **27 contract cases**, 13 fixture roundtrips, source-error preservation, metadata thiếu bị từ chối và reserved-key encoding quá sâu trả lỗi rõ ràng.
- MCP stdio: **7 tools**, **16 regression calls + 15 compression calls** đạt; lỗi tính toán của tool context giữ `isError`, helper MCP không trả thêm payload gốc.

Bằng chứng: `artifacts/test-results.json`, `artifacts/compression-test-results.json`, `artifacts/compression-binding-tests.json`, `artifacts/compression-schema-tests.log`, `artifacts/compression-mcp-tests.json`; checksum artifacts ở `artifacts/packages/manifest.json`. [Hướng dẫn bộ nén](payload-compression.md) có API, schemas, encoding và giới hạn. Không kiểm chứng lại stress forecast 136 MiB của 0.8 trong lượt này; cap 256 MiB là giới hạn contract, không là benchmark hiệu năng.

## Đối chứng tính toán trước compression

- Rust workspace: 95 tests qua (69 core, 9 couple integration, 9 composite integration, 6 forecast integration, 1 FFI và 1 provider). Bao gồm geometry/natal/couple/composite, event roots, stations, eclipses, fixed natal applying, period validation và reference closure. 13 event tests đã qua ở release profile trong 0.8; 8 query tests qua ở cả debug/release trong 0.9.
- Node: chạy package đã cài, validation errors, malformed JSON, type errors và 1.000 repeated calls.
- Python: 10 unittest gồm geometry, natal, reports, chart đôi và forecast; các tests repeated/concurrent calls qua.
- .NET: cài nupkg, build consumer không warning/error, validation và 1.000 lời gọi đồng thời.
- Parity: 339/339 cases qua Node/Python/.NET/Rust/C, đối chiếu toàn bộ decoded JSON; gồm 62 query cases mới và 277 regressions.
- Packaging: native asset phải đúng `runtimes/osx-arm64/native/libastro_engine.dylib` trên host này; manifest chứa SHA-256.
- Rust fmt/Clippy và syntax Python/JSON được kiểm tra trước giao.

Kết quả machine-readable sinh tại `artifacts/test-results.json`; checksum artifact tại `artifacts/packages/manifest.json`. Sau mỗi thay đổi binary/packaging, build lại và chạy lại consumer theo fingerprint mới.

Đối chứng 0.9: cài offline Node/Python/.NET `0.9.0-alpha.1` vào consumer fingerprint `64430cbbc44e`; 339/339 parity cases qua cả năm ngôn ngữ; 3 dedicated query examples Node/Python/.NET đối chiếu với Rust cũng qua. Python có 10 unittest qua; .NET build 0 warning/0 error; repeat/concurrent calls qua. Rust Clippy all-targets `-D warnings` và fmt check qua. Kết quả tại `artifacts/test-results.json`; MCP/frontend-host checks qua như bên dưới.

## On-demand query và integrations: kiểm chứng 0.9.0-alpha.1

- 8 query core tests kiểm tra normalize/wrap/separation, midpoint swap/antipodal policy, custom angle và 3 motion modes, zero speed/overflow/unknown rate, house boundaries/order, selected houses/rulers/occupants, selected point/contacts, complete pair sets, local indexes, raw decimal/exponent/tiny-fraction normalization và validation trước provider. Cả 8 qua debug/release; tổng native suite là 95.
- Node/Python/.NET thêm tests gọi đủ 8 grouped methods và đối chiếu raw query envelope, từ chối options/routing keys sai, thêm 20 query repeat/concurrent calls mỗi binding. Ba dedicated grouped examples (Node/Python/.NET --query) được đối chiếu với Rust trong runner.
- 5 TypeScript fixtures strict trên package đã cài; source/tarball/installed declarations byte-identical. Query validator: 88 request cases, 24 actual responses (22 success, 2 errors; 4 saved +20 edge fixtures), từ chối 123 malformed structures và 118 broken semantic joins. Validator còn tính lại angular geometry/motion, nhà chứa point, occupant sets, zodiac ruler table và complete selected relation pairs.
- Old validator regressions: temporal 121 requests/10 responses/204 malformed +119 joins; individual 38/2/34 +2; couple 55/3/66 +24; composite 105/3/84 +33, gồm wrap/antipodal. Tất cả qua trên saved samples/actual edge responses.
- 13 samples cũ từ installed 0.8 so với installed 0.9 giữ toàn bộ decoded JSON sau khi bỏ riêng engineVersion: natal, individual/custom, couple/custom, composite/custom/combined, daily/month/year forecast, annual sky và custom forecast. Tất cả 17 result samples được regenerate bằng installed Node `64430cbbc44e`.
- MCP: official SDK **1.29.0**, Ajv **8.20.0**, stdio local; client initialize/listTools và 16 callTool calls qua 5 tools. Có đủ 8 query actions, generic natal/couple/forecast, calendar/antipodal/raw-integer errors, routing-key/unknown-tool rejection. Protocol SDK dependencies dùng cache local; không thêm vào engine package và không tải mạng khi kiểm thử.
- Frontend host: Node worker_threads bridge có 11 checks qua, gồm 4 concurrent/queued requests, birth query, raw JSON/error envelope và đóng worker khi có request đang chờ. Đây là kiểm chứng worker/main-process adapter; chưa là test Electron renderer, React/Vue browser, Tauri UI hoặc WASM.

Machine-readable integration summary: `artifacts/integration-test-results.json`. Reproduction commands và host configuration ở [integrations.md](integrations.md), [MCP README](../examples/mcp/README.md), [frontend-host README](../examples/frontend/README.md).

| Query sample | Bytes, JSON 2-space |
| --- | ---: |
| [Nhà H2/H8](../examples/query-houses-result.json) | 45.883 |
| [Sun/MC/H7](../examples/query-points-result.json) | 25.981 |
| [Góc natal 37.5°](../examples/query-aspects-result.json) | 2.656 |
| [Hai tọa độ, góc 37.5°](../examples/query-between-result.json) | 1.347 |

```bash
node scripts/check-query-schema.cjs examples/query-houses-result.json examples/query-points-result.json examples/query-aspects-result.json examples/query-between-result.json
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs query-types.ts forecast-types.ts composite-types.ts couple-types.ts report-types.ts
# Đặt ASTRO_ENGINE_MODULE tới installed package trước khi chạy integration smoke:
node examples/mcp/smoke.cjs
node examples/frontend/smoke.cjs
```

Các phần 0.8/0.7/0.6/0.5 dưới đây là lịch sử đối chứng. Fingerprints/counts của chúng không đại diện artifact hiện tại; samples có thể đã regenerate bằng version mới.

## Calendar và forecast: đối chứng 0.8.0-alpha.1

Fresh artifacts 0.8.0-alpha.1 đã cài vào consumer `2887e68648fd`: 277/277 parity cases qua cả năm ngôn ngữ, 87 native tests qua, Python 9 unittest, .NET build 0 warning/0 error. Bộ đó có 72 temporal cases, 3 forecast examples và repeat/concurrent calls đã qua.

- Event tests kiểm tra ingress, stations, primary Moon phases, planetary aspects, global eclipses và exact transit vào natal. Có repeated retrograde hits, hai hits thật trong cùng khoảng 6 giờ, không tạo tangency giả; half-open boundaries, UTC offset, ngày leap và leap-second output cũng được kiểm tra.
- Regression stations 2026 giữ đủ 18 mốc: Mercury 6, Venus 2, Jupiter/Saturn/Uranus/Neptune/Pluto mỗi body 2, Sun/Moon/Mars 0. Station của Pluto ngày 6/5 được xác nhận bằng original velocity bracket. Swiss C được pin optimization level 3 cho cả Cargo debug/release; annual sky (1.400 events) và annual slow-body forecast (143) trả toàn bộ decoded JSON giống nhau giữa hai profiles, không làm tròn output để che khác biệt.
- Forecast integration kiểm tra một natal/context dùng chung, 10 views/30 sections, 260 snapshot relations mặc định, applying theo speed transit so với natal cố định, house/ruler/contact joins, custom sections độc lập, snapshot-only, exact contacts và validation trước provider. Natal trong forecast giống operation natal riêng.
- Independent references: đủ 50 primary Moon phases 2026 từ [USNO fixture](../neutral-engine/crates/astro-core/tests/fixtures/usno-moon-phases-2026.json), sai lệch tối đa 42,460 giây so với mốc được làm tròn đến phút; 4 dates/types/eclipse maxima từ [NASA fixture](../neutral-engine/crates/astro-core/tests/fixtures/nasa-eclipses-2026.json), sai lệch tối đa 17,589 giây so với bảng UT phút. Có kiểm tra mốc equinox trong lịch NASA. Đây là kiểm chứng tại các epochs này, không là cam kết độ chính xác tuyệt đối cho mọi năm. Nguồn: [USNO](https://aa.usno.navy.mil/data/MoonPhases), [NASA 2026](https://eclipse.gsfc.nasa.gov/OH/OH2026.html).
- Node/Python/.NET mỗi binding chạy thêm 20 forecast repeat/concurrent calls, trộn daily/monthly, custom profiles, eclipse/natal transits và disabled events/aspects. Các bộ 1.000 geometry và 1.000 natal calls mỗi binding được giữ.
- TypeScript strict kiểm tra cả 4 fixtures forecast/composite/couple/report trên package đã cài; declarations source/tarball/installed giống từng byte. Temporal validator chạy 121 request cases, 10 actual responses (9 success, 1 offset-bound error); từ chối 204 malformed structures và 119 broken semantic joins. Bốn edge variants gồm eclipse-only, quiet month, custom angle-only leap day và modern Whole Sign với offset âm.
- Old schema regressions: individual 38 requests/2 responses/34 malformed + 2 broken joins; couple 55/3/66 + 24; composite 105/3/84 + 33, gồm wrap/antipodal invariants. Regression cả 8 samples natal/individual/couple/composite so installed 0.7 giữ toàn bộ decoded JSON sau khi bỏ riêng engineVersion.
- Installed Node stress test: năm 2026, offset +420, đủ 10 bodies × 26 natal targets, extended aspects → 12.138 events, không truncated, 10 views/30 sections và 260 snapshot pairs. Gồm 9.069 natal transits, 2.788 planetary aspects, 209 ingresses, 18 stations, 50 phases và 4 eclipses. Raw compact JSON là 136.975.118 bytes; chọn filters phù hợp để giảm dữ liệu. Kết quả nhỏ lưu tại `artifacts/full-year-forecast-test.json`; một run không là latency/memory benchmark.

Samples dưới đây được regenerate bằng installed Node `2887e68648fd`, định dạng JSON 2-space:

| Sample | Events | Bytes |
| --- | ---: | ---: |
| [Ngày cá nhân](../examples/forecast-day-result.json) | 12 | 593.658 |
| [Tháng cá nhân](../examples/forecast-month-result.json) | 420 | 8.461.505 |
| [Năm cá nhân, Jupiter–Pluto](../examples/forecast-year-result.json) | 143 | 3.468.894 |
| [Năm thiên văn, Sun–Pluto](../examples/events-year-result.json) | 1.400 | 2.191.507 |
| [Forecast custom](../examples/custom-forecast-result.json) | 62 | 787.571 |

```bash
node scripts/check-forecast-schema.cjs examples/forecast-day-result.json examples/forecast-month-result.json examples/forecast-year-result.json examples/events-year-result.json examples/custom-forecast-result.json
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs forecast-types.ts composite-types.ts couple-types.ts report-types.ts
```

Các phần 0.7/0.6/0.5 bên dưới là lịch sử đối chứng; fingerprint và counts của chúng không đại diện artifact hiện tại. Sample files có thể đã được regenerate bằng version mới, xem bảng/current checks ở trên.

## Composite: đối chứng 0.7.0-alpha.1

Fresh artifacts tgz/wheel/nupkg được cài offline vào consumer `ceaab51fec70`. Node tests qua; Python có 8 unittest qua; .NET consumer build 0 warning/0 error và các tests qua. 205 conformance fixtures trả toàn bộ decoded JSON giống nhau qua Node/Python/.NET/Rust/C; 64 fixtures mới bao gồm composite riêng và opt-in trong couple. Tổng 67 core/FFI tests, rustfmt và Clippy all-targets `-D warnings` qua.

- Năm unit tests composite kiểm tra midpoint wrap/swap, đối đỉnh và policy rõ ràng, cusps không hợp lệ/Whole Sign tường minh, trục đối diện, facts C tính lại với motion không áp dụng. Chín integration tests độc lập kiểm tra coverage 10 profiles/30 sections, 26 points/325 relations/66 house-ruler pairs; source natal tương đương operation riêng; provenance correspondence; self/swap charts; combined/standalone parity; no partial success; section joins và validation trước provider.
- Conformance kiểm tra đủ 10 IDs riêng, defaults/major/extended/custom/disabled aspects, traditional/modern, mixed house systems, Southern Hemisphere, custom-only/mixed/angle-only, raw decimal/exponent/tiny-fraction normalization, malformed options/calendar/profiles và child inheritance trong `couple`. Fixture thực [composite-invalid-houses-request.json](../tests/conformance/composite-invalid-houses-request.json) trả `CALCULATION_FAILED` cho midpoint cusps; chỉ thành công khi caller chọn Whole Sign rõ ràng.
- Node/Python/.NET mỗi binding chạy thêm 20 repeat/concurrent mixed composite calls, có standalone và combined cùng custom/disabled-aspects. Sáu dedicated examples (composite và couple-composite qua ba bindings) cho cùng payload với Rust. Các kiểm tra 1.000 geometry và 1.000 natal calls mỗi binding tiếp tục qua.
- TypeScript `strict --noEmit`: cả `composite-types.ts`, `couple-types.ts`, `report-types.ts` qua trên package đã cài. Declarations source, tarball và installed package giống từng byte (27.825 bytes).
- AJV2020 strict và semantic composite validator chạy trên 7 actual envelopes từ installed Node (6 success, 1 invalid-midpoint-houses error): 105 request cases qua; 163 malformed structure mutations và 65 broken-reference mutations bị từ chối. Bao gồm modern/Whole Sign, angle-only/empty aspects, explicit lower-longitude policy, combined và wrap/antipodal invariants.
- Các result files đã lưu cũng qua validators: hai mẫu cá nhân (38 request cases, 34 malformed/2 broken joins); ba mẫu couple gồm combined (55 request cases, 66 malformed/24 joins); ba mẫu composite (105 request cases, 84 malformed/33 joins). Counts này thuộc các tập fixtures riêng, không cộng với bộ mở rộng phía trên.
- Regression so installed artifact `0.6.0-alpha.1` với `0.7.0-alpha.1`: năm payload `natal`, `natalDomains`, custom cá nhân, `couple`, custom couple giống toàn bộ decoded JSON sau khi bỏ riêng `engineVersion`. Không bật composite thì pair shape vẫn không có `chartCount`/C.
- Bốn astronomical references natal vẫn qua; sai lệch longitude độc lập lớn nhất `0.00012131835345030595°`. C là phép dựng hình học từ hai natal được kiểm chứng, không là một ephemeris tại birth epoch mới.

Samples đã regenerate bằng installed Node `0.7.0-alpha.1`: [composite-result.json](../examples/composite-result.json) 5.455.082 bytes (10 lĩnh vực/30 sections); [custom-composite-result.json](../examples/custom-composite-result.json) 730.411 bytes; [couple-composite-result.json](../examples/couple-composite-result.json) 9.473.891 bytes (6 synastry + 10 composite domains). Các samples natal/cá nhân/couple cũ cũng đã regenerate từ artifact này. Đây là kích thước fixtures, chưa là latency benchmark hay giới hạn mọi output; chọn các domain cần dùng để giảm dữ liệu báo cáo lặp lại.

```bash
node scripts/check-composite-schema.cjs examples/composite-result.json examples/custom-composite-result.json examples/couple-composite-result.json
node scripts/check-couple-schema.cjs examples/couple-result.json examples/custom-couple-result.json examples/couple-composite-result.json
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs composite-types.ts couple-types.ts report-types.ts
```

Các bằng chứng 0.5/0.6 bên dưới là lịch sử đối chứng cho những modules được giữ lại. Version/fingerprint của các lần đó không đại diện cho artifact hiện tại.

## Payload theo khía cạnh: đối chứng phase 0.6

`natalDomains` có 5 unit tests mới: coverage 26 unique points/325 unique pairs; endpoint/index/reference resolution; giữ aspect với body ngoài topic; traditional/modern rulership; Whole Sign MC độc lập H10; preset major/extended/custom/disabled và validation settings. Có thêm 6 tests advanced facts (dignity/solar boundaries/distributions/chains/reception/pattern topology) và 3 tests report (individual scope/support/evidence/reference closure, disabled aspects và từ chối người thứ hai). Phase 0.6 đã có 6 tests catalog/custom validation và 7 tests tích hợp profile/sections. Bộ cá nhân có 43 core tests và 1 FFI; phase 0.6 thêm 9 couple integration tests, tổng core/FFI là 53 tests.

Package consumers thêm 20 domain repeated calls Node, 20 concurrent calls Python và 20 concurrent calls .NET, trộn profile/hệ nhà/rulership. Cả ba dedicated examples (`domains.cjs`, `domains.py`, `.NET --domains`) trả cùng decoded JSON. Bộ conformance có 141 cases qua cả năm ngôn ngữ, bao gồm domain defaults, từng lĩnh vực, modern/Whole Sign, preset, custom rules, empty aspects và invalid settings. Core fmt/Clippy `-D warnings` qua; bốn astronomical references của natal vẫn qua.

Payload mẫu được tạo từ package Node đã cài: `examples/natal-domains-result.json`. Sample `0.6.0-alpha.1` có 219 matched contacts trong context; đây là tổng body/angle/cusp contacts theo extended rules, không phải số lượng chỉ body-to-body và không là chỉ số đánh giá. Domains có thể chia sẻ một contact, nên không cộng số contacts của các domains để suy ra tổng toàn chart.

## Báo cáo cá nhân nâng cao: đối chứng phase 0.6

Fresh artifacts `0.6.0-alpha.1` được cài vào consumer fingerprint `77a463182ed0` trên macOS ARM64. 141 parity cases đối chiếu toàn bộ payload, gồm các report/facts mới, Southern Hemisphere, lower-year boundary và ba trường hợp từ chối người thứ hai. 53 core/FFI tests qua; fmt/Clippy `-D warnings` qua. Package TypeScript declarations được đối chiếu với source đã pack.

- TypeScript `strict --noEmit`: `examples/node/report-types.ts` chạy trên package đã cài, kiểm tra overload/metadata cá nhân, `Result` compatibility và evidence discriminated unions; các kỳ vọng lỗi type được kiểm tra bằng `@ts-expect-error`.
- Kiểm tra AJV2020 ở phase cá nhân `0.5.0-alpha.1`: response schema hợp lệ; 38 request cases và sáu payload default/all-ten, custom-only, mixed, modern/Whole Sign, empty aspects/custom và angle-only hợp lệ; 98 mutation sai cấu trúc và 5 mutation tham chiếu sai bị từ chối; error envelope hợp lệ. Validator chỉ là dev tool, không phải dependency runtime của package.
- Sample đầy đủ được tạo từ Node package đã cài: `examples/natal-domains-result.json`, khoảng 4,89 MB (10 lĩnh vực/30 sections). Trong sample mặc định, các report chọn đủ 12 nhà nhưng điều này không phải invariant: empty/custom rules có thể cho ít nhà hỗ trợ hơn. Counts report gồm contacts hỗ trợ; không thay primary domain counts hoặc raw-pair coverage.

Chạy schema với Ajv đã có trong môi trường build; có thể chỉ định absolute module path bằng `AJV_2020_MODULE`:

```bash
node scripts/check-domain-schema.cjs examples/natal-domains-result.json examples/custom-profiles-result.json
# Trong project consumer đã cài package, dùng TypeScript compiler đã có:
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs report-types.ts
```

## Catalog, custom profiles và sections: đối chứng phase 0.6

- Default có 10 builtin profiles và 30 sections; catalog và normalized definition có trong response. Năm profile cũ giữ primary selection; report được mở rộng theo sections độc lập.
- Custom-only, mixed, angle-only, section ngoài selectors cha, disabled matching, modern/Whole Sign, ID/version/selector/null/duplicate/limit errors đều được đối chiếu qua năm bindings. Số nhà `[1.0, 9e0]` được chuẩn hóa; `1.0000000000000001` bị từ chối khi truyền raw JSON.
- Fixture 1980 có mutual reception và T-square thực trong output để các join tests không chỉ kiểm tra mảng rỗng. Section house/aspect/body/chain/reception/pattern references đều phải resolve trong parent report hoặc global context; `primaryContact` vẫn theo parent profile.
- Mỗi Node/Python/.NET consumer thêm 20 repeated/concurrent calls cho custom-only profile. Hai bộ dedicated examples (`domains` và `custom profiles`) đều trả cùng decoded JSON từ package đã cài.
- Sample custom riêng: `examples/custom-profiles-request.json` và `examples/custom-profiles-result.json`, khoảng 0,54 MB. Phase này chưa benchmark latency; payload size chỉ là kích thước sample, không phải giới hạn mọi response. Chọn `domains`/`customProfiles` đúng nhu cầu để giữ phạm vi báo cáo.

## Lá số cặp đôi: đối chứng phase 0.6

Fresh artifacts `0.6.0-alpha.1` được cài vào consumer `77a463182ed0`. Tổng 53 core/FFI tests và 141/141 parity cases qua Node/Python/.NET/Rust/C; trong đó có 9 couple integration tests và 57 couple conformance fixtures. .NET consumer build có 0 warning/0 error, Python có 7 unittest qua. Rust fmt và Clippy `-D warnings` qua.

- Kiểm tra hai natal/full contexts độc lập, mixed Placidus/Whole Sign, traditional/modern rulership và provider metadata; 52 qualified points, 676 cross pairs, 10 overlays mỗi chiều, 144 house-ruler pairs.
- Kiểm tra cross aspect/overlay/ruler direction, namespace/reference closure và cùng local body ID vẫn là hai người; cross `applying` luôn null. Aspects local trong natal vẫn theo tốc độ người đó.
- Kiểm tra 6 builtin profiles/18 sections, custom-only/mixed/angle-only, selectors section ngoài parent, đúng advanced refs về natal từng người; disabled matching giữ pairs/overlays.
- Negative cases gồm birth/calendar/location/house system sai theo từng người, options đặt sai cấp, null/unknown/duplicate/limit/reserved IDs, aspect preset/rules exclusion, empty selection và raw-number normalization.
- Node/Python/.NET consumer mỗi binding chạy thêm 20 repeat/concurrent couple calls, trộn domains/custom profiles/hệ nhà/rulership/custom rules. Các tests 1.000 geometry/natal calls trước đó vẫn qua.
- Ba dedicated examples (`couple.cjs`, `couple.py`, `.NET --couple`) có toàn bộ decoded JSON giống nhau từ package đã cài.
- TypeScript `couple-types.ts` và `report-types.ts` đều qua `strict --noEmit` trên package đã cài; tarball declarations giữ đúng source đã pack.
- AJV2020 strict chạy offline: 55 request cases, 7 actual responses (6 success + 1 error) qua; 130 malformed mutations và 46 broken-join mutations bị từ chối. Variants gồm full-six, custom-only, zero aspects, modern/Whole Sign + mixed options, angle-only và identical births. Fixture 1980 có mutual reception và T-square thực cho các local advanced joins.
- Regression package: output `natal`, `natalDomains` và custom profiles của artifact `0.5.0-alpha.1` so với `0.6.0-alpha.1` giống toàn bộ decoded payload sau khi bỏ riêng `engineVersion`. Không đổi output cá nhân để thêm cặp đôi.
- Bốn astronomical references của natal vẫn qua; lớn nhất `0.00012131835345030595°` theo kiểm chứng provider đã công bố. Couple dùng lại hai natal này; không có một provider/composite chart mới.

Samples được tạo bằng Node package đã cài: [couple-result.json](../examples/couple-result.json) 4.412.058 bytes (khoảng 4,41 MB, 6 lĩnh vực/18 sections); [custom-couple-result.json](../examples/custom-couple-result.json) 1.418.992 bytes (khoảng 1,42 MB, một profile `coCreation`/2 sections). Các samples natal/cá nhân cũng được regenerate từ artifact mới: basic natal 10.962 bytes, all-ten domains 4.892.131 bytes, custom cá nhân 537.146 bytes. Đây là kích thước samples, chưa là benchmark latency hoặc giới hạn mọi response.

```bash
node scripts/check-couple-schema.cjs examples/couple-result.json examples/custom-couple-result.json
node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs couple-types.ts report-types.ts
```

Validator đăng ký individual/pair schemas bằng URNs từ local files; `AJV_2020_MODULE` cho phép chọn Ajv trong môi trường build. Schema/semantic validator không là runtime dependency. Kết quả parity và counters nằm ở `artifacts/test-results.json`; native artifact checksums nằm ở manifest packages. Chỉ macOS ARM64 được xác minh trực tiếp trong lần chạy này.

## Regression integer normalization

Các trường UTC year/month/day/hour/minute và geometry harmonic nhận JSON number nguyên cả ở dạng decimal/exponent. Parser giữ literal qua serde_json raw_value để kiểm tra chính xác trước khi đổi sang integer; không làm tròn `2000.0000000000000001`. Tests bổ sung: canonical output, fractional/string/bool/null/overflow rejection, exponent notation và raw JSON tiny fractions qua năm bindings. Regression này được giữ trong bộ conformance hiện có.

## Đối chiếu natal và concurrency

- Node: thêm 1.000 natal repeated calls.
- Python và .NET: mỗi binding thêm 1.000 natal concurrent calls, trộn Placidus/Whole Sign và địa điểm Bắc/Nam bán cầu.
- Các parity cases natal cơ bản gồm: Placidus, Whole Sign, tọa độ 0, Nam bán cầu, polar failure/success, invalid leap day, longitude, unknown timezone và coverage endpoints.
- Bốn natal references tại các epochs 1950, 2000, 2020, gồm high-latitude Whole Sign. PySwissEph 2.10.3.2/Moshier đối chiếu longitude/latitude, speed, distance, 12 cusps, ASC/MC và TT/UT1.
- Skyfield 1.49 + JPL DE421 là mô hình độc lập, so longitude/latitude apparent geocentric ecliptic-of-date tại cùng TT epoch. Không dùng Skyfield để xác nhận house system hoặc UTC/Delta T model.
- Tolerance PySwiss: angle `0.00001°`, speed `0.00001°/day`, distance `0.000001 AU`, JD `0.0000001 day`. Tolerance JPL longitude/latitude `0.01°`. Sai lệch longitude lớn nhất quan sát: `0.00012131835345°` (khoảng 0,44 arcsecond).

Fixtures đã lưu trong `tests/conformance/natal-references.json`; scripts test không regenerate chúng. `scripts/generate-natal-references.py` là công cụ developer dùng riêng, cần PySwiss/Skyfield và BSP test trong artifacts. Consumer/test parity đọc fixtures JSON nên không cần cài hai thư viện tham chiếu hoặc tải JPL. Checksum BSP và version references nằm trong fixture.

## Giới hạn bằng chứng

Repeated calls và concurrency không thay thế memory leak sanitizer/fuzz tests. Parity cùng core xác minh binding/serialization; references thiên văn được kiểm tra riêng như dưới đây. Chưa kiểm tra rộng toàn bộ lịch sử hoặc toàn khoảng năm 1800–2399.

Candidate tại commit `2e1f53a0f786f3d0ad6309b8c659d6911717acda` từng qua [run 37093556052](https://github.com/7mlabs/sdk-astro/actions/runs/37093556052) với 107 Rust tests. Bản npm public dùng commit mới `a17850e01507485312e5cb584ed1eb82a786fcbc` và [run 37096125672](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672), thêm regression về trạng thái provider giữa các lần gọi và 108 Rust tests, cùng bốn final npm consumer jobs Node 18/24. Bản sửa station giữ nguyên tolerance và kiểm tra giới hạn thời điểm biểu diễn bằng f64. Parity dùng chung core xác minh bindings/serialization; không thay thế toàn bộ kiểm chứng thiên văn độc lập.

Windows Node linking chưa được triển khai. Final npm matrix xác minh Node 18 và 24 trên macOS ARM64/Linux x64; Node 20/22 chưa có final release matrix riêng. Python 3.10–3.12, .NET TFM khác, Linux distro/architecture khác và macOS x64 chưa được xác minh. Linux addon yêu cầu glibc 2.38+; Ubuntu CI không thay thế kiểm tra manylinux wheel. Không quảng bá các target chưa kiểm chứng là supported registry release. Các kết quả macOS trong các mục lịch sử phía trên thuộc phiên bản/đợt chạy được mô tả ở từng mục.

Legacy `LoveCompatScoringTests.GoldenScores_AreStable` đã lỗi từ source gốc. Neutral engine không chứa compatibility scoring và không phụ thuộc test đó. Không sửa kỳ vọng legacy để làm đẹp kết quả test mới.

## Chạy lại

```bash
cargo test --manifest-path neutral-engine/Cargo.toml --locked
cargo fmt --manifest-path neutral-engine/Cargo.toml --all --check
cargo clippy --manifest-path neutral-engine/Cargo.toml --all-targets --locked -- -D warnings
artifacts/python-build/bin/python scripts/build-packages.py
artifacts/python-build/bin/python scripts/test-packages.py
```

Các lần phát hành tiếp theo giữ fresh registry install verification và mở rộng OS/runtime matrix theo binary thực tế. Memory sanitizers, fuzz, data coverage rộng hơn và browser/native parity cần bổ sung khi triển khai phần tương ứng; không coi registry smoke hoặc parity cùng core là bằng chứng những kiểm tra đó đã chạy.
