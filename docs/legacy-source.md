# Tài liệu source legacy

Tài liệu dưới đây lưu trạng thái Python/.NET/sandbox và chiến lược SDK của lần tách trước. Kiến trúc hiện tại đã chọn engine local Rust với bindings; đọc README mới và architecture.md để theo định hướng đang triển khai. Các tính năng/API legacy chưa tự động tồn tại trong Rust alpha.

**Tài liệu lưu trữ:** các thư mục `Astro.Physics`, `Astro.Engineer`, `Astro.Core` và `astro-sandbox` không nằm trong repo `7mlabs/sdk-astro`. Lệnh khởi chạy bên dưới chỉ mô tả source cũ, không chạy trong repo SDK này. Để build engine hiện tại, dùng hướng dẫn trong [README](../README.md).

# Astrology Engine 🌌

**Astrology Engine** là hệ thống tính toán chiêm tinh dùng Skyfield với JPL DE421 và Swiss Ephemeris. Dự án đang chuyển sang kiến trúc thư viện có thể đóng gói, dùng chung cho HTTP API, SDK theo ngôn ngữ, MCP và frontend. Kết quả luận giải và điểm tương hợp được tạo bằng quy tắc; việc dùng dữ liệu JPL không bảo chứng mọi công thức hay nội dung luận giải.

---

## 🚀 Hướng dẫn Khởi chạy (Quick Start)

Ứng dụng hiện có 3 tiến trình và thư viện `Astro.Core`. Mở 3 terminal để chạy Python Physics, HTTP API .NET và sandbox. `Astro.Core` không cần chạy server riêng.

### 1. Khởi động Astro.Physics (Bộ Não Thiên Văn)
Đóng vai trò Engine xử lý dữ liệu thiên văn thô (Sử dụng Skyfield & NASA JPL Ephemeris).
*   **Yêu cầu:** `Python 3.10+`
*   **Port mặc định:** `8000`

```bash
cd Astro.Physics
python -m venv venv
source venv/bin/activate  # (Trên Windows dùng: venv\Scripts\activate)
pip install -r requirements.txt
uvicorn main:app --host 0.0.0.0 --port 8000
```
*(Lưu ý: Lần chạy đầu tiên có thể tốn thời gian để tự động tải file `de421.bsp` từ NASA).*

### 2. Khởi động Astro.Engineer (Kiến trúc sư Dữ liệu)
Đóng vai trò Backend API Gateway xử lý logic chiêm tinh (Cung Hoàng Đạo, Góc Chiếu, Nhà).
*   **Yêu cầu:** `.NET 10 SDK`
*   **Port mặc định:** `5001`

```bash
cd Astro.Engineer
dotnet run --launch-profile http
```

### 3. Khởi động Astro Sandbox (Trang Tài liệu & API Test)
Giao diện trực quan tích hợp toàn bộ tài liệu giới thiệu hệ thống và công cụ Test API trực tiếp.
*   **Yêu cầu:** phiên bản Node.js đáp ứng yêu cầu của Next.js 16 trong `astro-sandbox/package.json`.
*   **Port mặc định:** `3001`

```bash
cd astro-sandbox
npm install
npm run dev
```

---

## 📖 Tài liệu Kỹ thuật & Danh sách API (Documentation)

Sandbox chứa tài liệu API và ví dụ thử nghiệm. Định hướng package và cấu trúc tính toán được mô tả bên dưới.

Mọi thông tin chi tiết về:
- Kiến trúc 2 lớp độc lập.
- Bảo chứng độ chính xác (NASA-Grade Precision).
- Mô hình tích hợp quy trình LLM Workflow (Prompt Engineering).
- Định vị hệ thống so với các API ngoài thị trường.
- **Danh sách API đầy đủ và Giao diện Sandbox thử nghiệm.**

**👉 Vui lòng khởi chạy Astro Sandbox và truy cập vào địa chỉ:**
### [http://localhost:3001](http://localhost:3001)

Tại trang chủ của Sandbox, hãy nhấn vào Menu **"📖 Giới thiệu Dự án"** ở góc trái màn hình để đọc toàn bộ tài liệu giải pháp (Solution Architecture).

## Các nhóm tính năng cốt lõi

| Nhóm | Phạm vi | Ưu tiên |
|---|---|---|
| Thiên văn | Vị trí, vận tốc, nghịch hành, hệ tọa độ, nhà và Ayanamsa | Nền tảng bắt buộc |
| Bản đồ sao | Natal, Big Three, nhà, góc chiếu, chủ tinh và cân bằng nguyên tố | Bản phát hành đầu |
| Tương hợp | Synastry, house overlays, composite, tình yêu và công việc | Sau khi natal được kiểm chứng |
| Diễn biến thời gian | Daily transit, lịch tháng, Solar Return, tìm tọa độ và trạm nghịch hành | Hoàn thiện lấy mẫu và tìm nghiệm |
| Hiển thị | SVG, vòng bản đồ sao, bảng dữ liệu và báo cáo | Dùng cùng kết quả từ core |
| Mở rộng | Harmonics, progression, fixed stars, asteroids, Human Design | Module tùy chọn, công bố mức hỗ trợ |

Nội dung luận giải là một lớp riêng. Không dùng điểm tương hợp làm độ tin cậy của tính toán thiên văn. Social hiện tái sử dụng công thức work; chưa quảng bá là engine bạn bè độc lập. Prenatal epoch hiện chỉ cung cấp design date và placements, chưa phải Human Design hoàn chỉnh.

## Cấu trúc hiện tại sau khi tách thư viện

```text
Astro.Core/
  Models/            # Model dùng chung, giữ namespace cũ để giảm thay đổi API
  Calculations/      # Tính natal/transit aspects từ positions được cung cấp
  Compatibility/     # Scoring love/work/social từ SynastryChart
Astro.Physics/
  main.py            # FastAPI và tích hợp ephemeris hiện tại
  angles.py          # Hàm góc và vận tốc độc lập với web/ephemeris
  tests/             # Test toán học chạy bằng unittest
Astro.Engineer/
  Controllers/       # HTTP endpoints
  Services/          # Điều phối Physics, natal enrichment, synastry, transit, SVG
Astro.Engineer.Tests/ # Test thư viện core
astro-sandbox/       # Frontend tài liệu và thử API
```

`Astro.Core` không phụ thuộc ASP.NET, FastAPI, HTTP, database hay file ephemeris. Nó nhận positions/chart đã được tính để tạo aspects và điểm tương hợp. Package này chưa tự tính vị trí hành tinh từ ngày sinh. Natal enrichment, nhà, composite và việc gọi Physics vẫn nằm trong API; đây là giai đoạn tách đầu tiên, chưa phải toàn bộ engine đã được chuyển thành package.

API vẫn giữ các đường dẫn và namespace model/scoring cũ. Assembly chứa các type đã chuyển sang `Astro.Core`; các tích hợp dùng reflection hoặc binary cũ cần build lại. Python và .NET cần triển khai cùng nhau do đơn vị `Speed` đã đổi từ độ/giờ sang độ/ngày cho hành tinh Skyfield. Chỉ đổi số version không thể tự chuyển response cache cũ; xóa hoặc phân vùng cache khi nâng cấp.

## Quy ước tính toán và kiến trúc đích

Pipeline đích: kiểm tra input → chuẩn hóa UTC và địa điểm → ephemeris provider → positions và houses → aspects và natal features → transit/synastry → scoring → output có metadata → render hoặc luận giải.

1. Góc dùng độ; longitude nằm trong `[0, 360)`. Vận tốc dọc hoàng đạo dùng độ/ngày. Orb dùng độ, lưu giá trị thô cho tính toán và chỉ làm tròn khi xuất kết quả.
2. API thấp tầng hiện nhận UTC. API/SDK cấp cao trong tương lai nhận ISO datetime với timezone IANA, tọa độ, zodiac, Ayanamsa, house system, detail level và danh sách bodies. Phân biệt không biết giờ sinh với giờ sinh 00:00.
3. UTC là thời gian input/output; TT/UT1 do provider chuyển đổi khi thuật toán yêu cầu. Không cộng offset Modified Julian Date vào Julian Date đầy đủ.
4. Provider trả positions cùng nguồn dữ liệu, đơn vị, phiên bản và danh sách bodies thiếu. Thiếu ephemeris phải trả cảnh báo hoặc lỗi, không tạo tọa độ giả hay im lặng bỏ qua.
5. Phần hình học không gọi mạng hoặc đọc file. Chính sách orb, scoring và nội dung mô tả phải là cấu hình có version. Tách metadata tính toán khỏi nội dung luận giải.
6. Output đích gồm `schemaVersion`, `engineVersion`, `providerVersion`, `calculation`, `data`, `warnings` và `errors`. Hiện natal đã thêm metadata đơn vị/schema; envelope chung và schema liên ngôn ngữ chưa được triển khai.
7. Golden fixtures dùng chung cho mọi SDK/provider, có tolerance cho tọa độ và thời điểm. Test bao phủ ranh giới 0/360 độ, năm nhuận, múi giờ, vĩ độ cao, thiếu dữ liệu, retrograde và Sidereal. Điểm scoring có version và fixtures riêng.

Các việc còn cần làm: tách ephemeris provider khỏi FastAPI; bổ sung DTO request và validation; chuyển houses/composite/natal enrichment sang core; thống nhất chính sách strength; hoàn thiện input không có giờ sinh; sửa harmonic output phụ thuộc tọa độ mới; kiểm chứng progression và search; bổ sung test đối chiếu thiên văn. Ngày quét transit hiện là mẫu lấy theo lịch, không phải chứng nhận thời điểm exact.

## Chiến lược package theo ngôn ngữ

Giữ một engine tham chiếu, một hợp đồng dữ liệu và một bộ fixtures. Ưu tiên TypeScript/Node.js, Python và .NET vì đây là các runtime sẵn có trong source. Chưa viết lại toàn bộ engine cho mỗi ngôn ngữ; thêm Java, PHP, Go hoặc Swift khi có nhu cầu và contract ổn định.

Phân biệt hai loại package:

- SDK: đóng gói client, type, validation, timeout, lỗi và API cấp cao; cần backend đang hoạt động. Một lệnh npm/pip cài SDK không tự cài .NET, Python server hay dữ liệu thiên văn.
- Engine: tính cục bộ từ input. Cần đóng gói runtime, ephemeris, binding và công thức. `Astro.Core` hiện chỉ là thư viện tính từ dữ liệu đã cung cấp; chưa là engine natal offline.

Các tên dưới đây là đề xuất, chưa kiểm tra quyền sở hữu registry và chưa publish:

| Runtime | Package dự kiến | Lệnh sau khi phát hành |
|---|---|---|
| Node.js/TypeScript | `@7mlabs/astrology` — SDK HTTP | `npm install @7mlabs/astrology` |
| Python | `sevenmlabs-astrology` — SDK và provider local theo extras | `pip install sevenmlabs-astrology` |
| .NET | `SevenMLabs.Astrology.Core` và `.Client` | `dotnet add package SevenMLabs.Astrology.Core` |
| CLI | `@7mlabs/astrology-cli` | `npx @7mlabs/astrology-cli natal --help` |
| MCP | `@7mlabs/astrology-mcp` | `npx @7mlabs/astrology-mcp` |
| React UI | `@7mlabs/astrology-react` | `npm install @7mlabs/astrology-react` |

Ví dụ API SDK đề xuất, chưa chạy được ở trạng thái hiện tại:

```ts
const client = new AstrologyClient({ baseUrl: "http://localhost:5001" });
const result = await client.natal({
  birth: { dateTime: "2000-01-01T12:00:00", timeZone: "Asia/Ho_Chi_Minh" },
  location: { latitude: 10.8231, longitude: 106.6297 },
  zodiac: "tropical",
  detailLevel: "standard"
});
```

Lộ trình phát hành:

1. Tách core, sửa đơn vị và kiểm chứng toán học. Có thể pack `.nupkg` để dùng nội bộ.
2. Ổn định contract và OpenAPI/JSON Schema; tạo fixtures chuẩn và error model. SDK các ngôn ngữ có cùng ý nghĩa input/output, dùng naming phù hợp từng ngôn ngữ.
3. Phát hành SDK TypeScript/Python/.NET có timeout, cancellation, khả năng cấu hình endpoint, tài liệu và test contract. SDK/browser không chứa server credential.
4. Phát hành CLI và MCP tái sử dụng SDK hoặc application service, không viết lại công thức.
5. Tách UI kit và xây web app người dùng. Nếu dùng React kit, chỉ phụ thuộc contract/rendering, không import backend runtime.
6. Thử nghiệm engine offline riêng. Node.js cần native binding, WASM hoặc runner cục bộ được đóng gói đúng nền tảng; không yêu cầu SDK browser tải Python/.NET.

Trước mỗi lần phát hành: kiểm tra build từ source sạch, đóng gói và cài trong project trống, parity fixtures, changelog và compatibility matrix. Tách version schema, version engine và version scoring; breaking changes cần migration guide. Package ổn định phải khai báo license và dependency/data provenance.

Repo hiện chưa có LICENSE ở gốc; cần xác minh quyền phân phối source gốc trước khi chọn license mới. Swiss Ephemeris có mô hình license kép AGPL hoặc Professional theo [tài liệu của nhà cung cấp](https://www.astro.com/swisseph-download/doc/swephprg.pdf); quyết định mô hình phân phối và dữ liệu đi kèm là điều kiện của bước release. Gỡ remote Git không thay đổi bản quyền.

## MCP

Có thể phát triển MCP từ source này. MCP là adapter giúp AI gọi cùng application service; không thay thế engine, SDK hoặc frontend.

Tools đề xuất: `calculate_natal_chart`, `get_big_three`, `calculate_synastry`, `calculate_compatibility`, `calculate_transits`, `find_planetary_event`, `render_chart`. Mỗi tool có input/output schema và trả kết quả có metadata/cảnh báo. Tool tính toán được đánh dấu read-only khi không lưu dữ liệu. Không đưa chức năng Human Design đầy đủ vào tool trước khi module thực sự tồn tại.

Giai đoạn đầu, server MCP gọi HTTP API hiện tại bằng SDK. Local dùng stdio; triển khai remote dùng Streamable HTTP theo [MCP TypeScript SDK](https://ts.sdk.modelcontextprotocol.io/server). Với stdio, stdout chỉ chứa protocol; log đi stderr. Đặt giới hạn khoảng thời gian quét, timeout và số tác vụ; bổ sung auth khi remote. Resources cung cấp quy ước tính, schema và khả năng hỗ trợ; prompts chỉ hỗ trợ trình bày kết quả, không tự tính lại tọa độ.

Một lệnh `npx` chỉ đơn giản hóa việc chạy adapter. Nếu adapter dùng HTTP thì backend vẫn phải chạy hoặc được cung cấp dưới dạng dịch vụ. Cấu hình endpoint thuộc client/server, không được AI tùy ý đổi URL đích.

## Frontend tương thích

Package tính toán không tự sinh frontend. Có thể phát triển frontend tương thích bằng cách dùng cùng contract và SDK. Giữ sandbox hiện tại làm công cụ developer; tạo app người dùng và UI kit khi API ổn định.

- Web app: form ngày/giờ/nơi sinh, xác nhận timezone, hồ sơ, bản đồ sao, trang tương hợp, lịch transit và xuất báo cáo.
- UI kit: `BirthInput`, `ChartWheel`, `PlanetTable`, `AspectTable`, `CompatibilityReport`, `TransitTimeline`. Các component hiển thị cùng kết quả JSON và chuyển đổi format/language; không chứa công thức scoring khác core.
- Các app Vue/mobile dùng SDK/HTTP cùng contract; React component không dùng trực tiếp trên mọi framework.
- Browser → backend của app → SDK/application service → core/provider. Dữ liệu sinh và credential server không đặt vào bundle frontend; lưu hồ sơ là module sản phẩm riêng.
- MVP UI ưu tiên natal/Big Three và love compatibility. Hiển thị trạng thái thiếu giờ sinh hoặc thiếu ephemeris; loading, lỗi và nội dung giải thích phải dựa trên response thật.

## Kiểm tra và đóng gói nội bộ

```bash
# Tests core, không cần Python server
dotnet test Astro.Engineer.Tests/Astro.Engineer.Tests.csproj
# Test toán học Python, không tải ephemeris
(cd Astro.Physics && python3 -m unittest discover -s tests -v)
# Từ thư mục gốc repo: tạo NuGet artifact cục bộ
dotnet pack Astro.Core/Astro.Core.csproj -c Release -o artifacts/packages
```

Artifact local không phải package đã publish trên NuGet. Chưa có npm/PyPI/MCP/UI package ở thời điểm tách core này.

Kết quả kiểm tra lần tách đầu: API .NET build được; 6 test toán học Python qua; 19/20 test .NET qua, gồm 4 test aspects mới. `LoveCompatScoringTests.GoldenScores_AreStable` vẫn lỗi do kỳ vọng `balanced` trong khi công thức trả `contact_leads`; chạy lại nguyên models/scoring/tests từ HEAD gốc cũng tái hiện lỗi này. Chưa đổi công thức hoặc kỳ vọng để ép test qua. Package NuGet cục bộ được kiểm tra cài và gọi phép tính ở project độc lập.

Build còn cảnh báo nullable từ source gốc và cảnh báo dependency `Microsoft.OpenApi` 2.0.0 ([GHSA-v5pm-xwqc-g5wc](https://github.com/advisories/GHSA-v5pm-xwqc-g5wc)). Giải quyết các cảnh báo và lỗi golden test trước bản phát hành public. Chưa chạy frontend hoặc toàn bộ ephemeris pipeline, nên các sửa đổi đơn vị/thời gian cần kiểm chứng thêm bằng fixtures thiên văn.
