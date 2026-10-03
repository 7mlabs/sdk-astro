# Astrology Engine

Engine chiêm tinh trung lập hướng đến package chạy cục bộ cho nhiều ngôn ngữ. Dev cài thư viện vào dự án và gọi hàm; không cần duy trì server tính toán. Kiến trúc được chọn là một lõi Rust dùng chung, C ABI cho native bindings và Node-API cho Node.js.

Repo đích: [7mlabs/sdk-astro](https://github.com/7mlabs/sdk-astro). Đây là bản source riêng cho engine và SDK; ứng dụng Python/.NET/sandbox cũ và lịch sử Git cũ không được đưa vào repo này. Website giới thiệu và docs: [7mlabs](https://sevenmlabs-packages.velikho.chatgpt.site/astro/).

## Trạng thái hiện tại

Bản alpha `0.10.0-alpha.1` có hai luồng cá nhân **ngày giờ UTC + tọa độ → natal JSON**: `natal` trả lá số cơ bản và `natalDomains` bổ sung dữ liệu theo 10 lĩnh vực cá nhân: công việc, tình cảm, quan hệ, gia đình, tài chính, bản thân, học tập, sáng tạo, nội tâm và đời sống hằng ngày. Đầu ra cơ bản gồm Sun–Pluto, longitude/latitude/distance, speed/retrograde, zodiac sign, ASC/MC/DSC/IC, 12 nhà và major aspects. Hỗ trợ tropical, geocentric, Placidus hoặc Whole Sign; dùng Swiss Ephemeris native với Moshier tích hợp, chạy offline trong process của caller.

`natalDomains` trả natal của **một cá nhân**, 26 points, đủ 325 cặp góc, aspects theo preset hoặc custom rules, chủ tinh/occupants của 12 nhà và các views có selection reasons. Mỗi lĩnh vực có `report` mở rộng nhà liên quan và facts nâng cao: body conditions, distributions, dispositor chains/cycles, mutual domicile receptions, supported aspect patterns và evidence để dựng báo cáo. `love`/`relationships` không so sánh hai người. Có các mục báo cáo con và `customProfiles` để dev khai báo selectors riêng; mọi profile dùng chung công thức core. Profile và rules có version; engine không trả scoring hoặc luận giải. Bỏ `domains` mặc định tạo cả 10 lĩnh vực; chọn danh sách IDs để giảm phần report lặp lại.

`couple` nhận birth input của hai người, tính hai natal độc lập rồi dựng dữ liệu synastry: 52 points có namespace theo người, đủ 676 cross-relations, contacts, 20 house overlays và 144 quan hệ giữa chủ tinh nhà A/B. Có 6 lĩnh vực cặp đôi, 18 sections và custom profiles; xem [contract cặp đôi](docs/couple.md). Đây là payload thô phục vụ báo cáo, không có compatibility score hoặc lời luận giải.

`composite` dựng chart C bằng trung điểm của hai natal, có đủ 10 lĩnh vực/30 sections như lá số đơn. Tọa độ, nhà, góc chiếu, chủ tinh và advanced facts được tính lại cho C; không gán ngày sinh hay tốc độ hành tinh giả. Có thể gọi riêng hoặc thêm `composite: {}` vào `couple` để dùng lại hai natal đã tính; xem [contract composite](docs/composite.md).

Package local Node.js, Python, .NET và sample Rust/C cùng gọi một core. Geometry `chart`, `harmonic`, `synastry` từ positions vẫn được giữ. `events` quét ngày/tháng/năm gồm ingress, stations, Moon phases, planetary aspects và global eclipses. `forecast` gắn các mốc với natal cá nhân: snapshot, exact natal transits, house/ruler/contact evidence, 10 lĩnh vực/30 sections và messageContext thô; xem [payload ngày/tháng/năm](docs/forecast.md). `query` bổ sung 8 phép tính/truy vấn theo nhóm geometry/aspects/houses/points; xem [truy vấn theo nhu cầu](docs/query.md). Có [mẫu MCP local và frontend worker bridge](docs/integrations.md). Chưa thêm Davison, progression/return charts, timezone IANA hoặc browser WASM. Xem [kết quả kiểm thử](docs/testing.md), [API natal](docs/natal.md) và [payload theo lĩnh vực](docs/domains.md).

`compressPayload` là bước xử lý riêng sau tính toán. Chế độ `compact` nén biểu diễn và giải nén được toàn bộ giá trị JSON; `focused` chọn lĩnh vực/sections, giữ context hỗ trợ; `budgeted` kiểm tra giới hạn và báo rõ khi dữ liệu tối thiểu vượt budget. `calculateWithContext` trả cả `result` gốc và `context` đã nén. Xem [hướng dẫn bộ nén](docs/payload-compression.md); API tính toán hiện có không đổi.

## Tài liệu

| Tài liệu | Nội dung |
|---|---|
| [Kiến trúc](docs/architecture.md) | Lựa chọn lõi, binding, provider và ranh giới module |
| [Plan chuyển đổi theo phase](docs/phases.md) | Đầu ra, gate và trạng thái từng phase |
| [Plan triển khai](docs/implementation.md) | Mapping legacy, backlog, build và release |
| [Natal cơ bản](docs/natal.md) | Birth input, provider, phạm vi và ví dụ output |
| [Payload theo lĩnh vực](docs/domains.md) | 10 lĩnh vực cá nhân, nhà H, rulership và toàn bộ contacts |
| [Báo cáo natal cá nhân](docs/individual-reports.md) | Advanced facts, nhà hỗ trợ có evidence, các mục báo cáo con và giới hạn |
| [Ngày/tháng/năm và sự kiện](docs/forecast.md) | Daily messageContext, monthly overview, annual events và ảnh hưởng theo natal |
| [Lá số thứ ba](docs/composite.md) | Midpoint composite C, 10 lĩnh vực/30 sections và provenance |
| [Lá số cặp đôi](docs/couple.md) | Birth input hai người, synastry, overlays, lĩnh vực và reference joins |
| [Profiles tùy chỉnh](docs/profiles.md) | Catalog 10 profiles, custom selectors, validation và migration |
| [Truy vấn theo nhu cầu](docs/query.md) | Góc tùy ý, nhà/điểm được chọn và 4 nhóm SDK |
| [Nén payload cho LLM](docs/payload-compression.md) | Bộ nén dùng chung, chọn lĩnh vực, budget và giải nén |
| [MCP và frontend host](docs/integrations.md) | Adapter stdio local, native worker và runtime matrix |
| [API](docs/api.md) | Input/output, quy ước tính, lỗi và giới hạn |
| [Cài package](docs/packages.md) | Cài tgz/wheel/nupkg và chạy từng ngôn ngữ |
| [Kiểm thử](docs/testing.md) | Unit, fresh install, parity và giới hạn bằng chứng |
| [Frontend](docs/frontend.md) | Playground JSON/chart, WASM và chức năng offline |
| [Phân phối public](docs/distribution.md) | GitHub, registry, license và dữ liệu |
| [Thiết lập GitHub và phát hành](docs/github-setup.md) | Repo đích, CI candidate, registry và các bước còn cần owner cấu hình |
| [Tài liệu legacy](docs/legacy-source.md) | Source Python/.NET/sandbox và lần tách .NET trước |

## Build và test trực tiếp

Máy build cần Rust 1.83+, Node/npm, Python và .NET 10 SDK. Sau khi public prebuilt package, consumer chỉ cần runtime ngôn ngữ của mình trên nền tảng được hỗ trợ.

```bash
python3 -m venv artifacts/python-build
artifacts/python-build/bin/python -m pip install setuptools wheel
artifacts/python-build/bin/python scripts/build-packages.py
artifacts/python-build/bin/python scripts/test-packages.py
```

Script test cài các artifact thật vào project riêng, không dùng source imports của Node/Python/.NET. Tất cả calculation/parity chạy local. `NODE_BIN` và `NPM_CLI` cho phép chọn tool không nằm trên PATH. Dùng `--offline` khi Cargo dependencies đã có trong cache.

## Ví dụ Node.js

Cài tgz tạo trong `artifacts/packages`, rồi:

```js
const { calculate } = require('@7mlabs/astrology');
const result = calculate({
  operation: 'natal',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  houseSystem: 'placidus'
});
console.log(result.data.placements, result.data.houses);
```

Lấy dữ liệu theo lĩnh vực bằng cùng SDK:

```js
const domains = calculate({
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['career', 'love'],
  aspectPreset: 'extended',
  rulership: 'traditional'
});
console.log(domains.data.domains.career.houses);
console.log(domains.data.domains.love.aspects);
console.log(domains.data.domains.love.report.sections);
```

Lá số cặp đôi dùng cùng SDK với `operation: 'couple'`, `personA` và `personB`; mỗi người có `utc`, `location` và `houseSystem` riêng. [Request mẫu](examples/couple-request.json) và samples [Node.js](examples/node/couple.cjs), [Python](examples/python/couple.py), [.NET `--couple`](examples/dotnet/Program.cs) tạo đủ 6 lĩnh vực.

Ví dụ dùng ngày 01/01/2000 lúc 12:00 UTC và tọa độ TP.HCM; giờ địa phương phải được caller đổi sang UTC. [Source mẫu](examples/) gồm Node.js, Python, .NET, Rust và C.

## Public release

Repo đích đã được xác định là `7mlabs/sdk-astro`; source hiện được chuẩn bị ở local, chưa push. Package chưa publish trên registry; các tên package chưa được xác nhận quyền sở hữu. CI candidate build/test artifact trên macOS ARM64 và Linux x64, không phát hành package. Kết quả build ở local chỉ xác nhận nền tảng đã chạy, không thay thế kết quả GitHub Actions.

License của source mới và mô hình phân phối Swiss Ephemeris cần được owner xác định trước khi public. Giữ nguyên vendor licenses và notices; không tự relicense source clone. Xem [các bước thiết lập](docs/github-setup.md) và [phân phối](docs/distribution.md).
