# Phát hành package Node.js

Package [@7mlabs/astrology `0.10.0`](https://www.npmjs.com/package/@7mlabs/astrology) đã phát hành public trên npm ngày 2026-10-03, kênh ổn định `latest`. Source trong `bindings/node/package.json` vẫn đặt `private: true`; chỉ artifact đã qua kiểm tra release được chuyển sang metadata có thể publish. Python/.NET có version `0.10.0` nhưng chưa phát hành registry. UI renderer giữ version riêng `0.9.0-alpha.1` và chưa phát hành registry.

## Cài đặt và cập nhật

Cài bản ổn định:

```sh
npm install @7mlabs/astrology
```

Cập nhật tường minh sang kênh `latest`:

```sh
npm install @7mlabs/astrology@latest
```

Để tái tạo đúng một phiên bản, chỉ định version và commit lockfile của ứng dụng:

```sh
npm install --save-exact @7mlabs/astrology@0.10.0
```

Package chứa sẵn native addon; người dùng không cần Rust, compiler, server hay bước tải binary sau cài đặt. `npm ci` dùng lockfile để cài lại đúng dependency của dự án. Kênh `alpha` tiếp tục trỏ tới các bản thử nghiệm riêng; chọn `latest` không mở rộng phạm vi platform của binary.

## Nền tảng của bản npm

Một tarball npm chứa cả hai binary, với cùng version, wrapper và TypeScript declarations:

| Target | Yêu cầu binary | Môi trường đã kiểm tra |
| --- | --- | --- |
| `darwin-arm64` | macOS, Apple Silicon; Mach-O deployment target 11.0 | GitHub Actions `macos-15`, ARM64 |
| `linux-x64` | Linux x86-64, glibc **2.38 trở lên**, có `libgcc_s.so.1` | GitHub Actions `ubuntu-24.04`, x64 |

macOS 11.0 là deployment target đọc từ binary, chưa phải bằng chứng đã chạy test trên mọi bản macOS từ 11 đến 14. Binary macOS chỉ liên kết thêm `/usr/lib/libSystem.B.dylib`. Binary Linux yêu cầu `libm.so.6`, `libgcc_s.so.1`, `libc.so.6` và `ld-linux-x86-64.so.2`; symbol `__isoc23_strtol@GLIBC_2.38` đặt giới hạn glibc hiện tại. Không suy ra hỗ trợ một distro chỉ từ tên target `linux-x64`.

Ứng dụng cũng phải đáp ứng yêu cầu hệ điều hành của Node.js đang dùng. Chẳng hạn, [Node.js 24 yêu cầu macOS 13.5 trở lên](https://github.com/nodejs/node/blob/v24.x/BUILDING.md#platform-list), dù addon có deployment target 11.0. [Node.js 22 hỗ trợ macOS từ 11.0](https://github.com/nodejs/node/blob/v22.x/BUILDING.md#platform-list). Yêu cầu thực tế là mức cao hơn giữa Node.js và addon.

Windows, macOS Intel, Linux ARM64 và Alpine/musl chưa có binary trong release này. Loader chọn chính xác thư mục `native/<platform>-<arch>/astro.node`; danh sách `os` và `cpu` trong npm metadata không biểu diễn được tất cả cặp OS/architecture hợp lệ. Thiếu binary hoặc libc không phù hợp sẽ làm bước import thất bại, không chuyển sang tính qua server.

Để mở rộng Linux sang glibc cũ hơn, cần build trên một baseline libc cũ phù hợp và chạy lại conformance trên baseline đó. Việc đổi tên artifact hoặc giảm con số trong docs không làm binary tương thích hơn.

## Node.js

Metadata đặt minimum Node.js 18. Native adapter sử dụng Node-API v1 và không phụ thuộc V8 API; tarball ổn định `0.10.0` đã qua kiểm tra thực tế trên Node 18/24 ở cả hai target; bằng chứng chi tiết được ghi bên dưới. [Node-API duy trì ABI giữa các phiên bản Node.js](https://nodejs.org/api/n-api.html).

Khuyến nghị ứng dụng mới dùng Node.js 22 hoặc 24 LTS. Node.js 18 và 20 đã EOL tại thời điểm tháng 10/2026; kiểm tra tương thích với chúng không kéo dài hỗ trợ bảo mật của Node.js. [Lịch phát hành Node.js](https://nodejs.org/en/about/previous-releases), [các phiên bản EOL](https://nodejs.org/en/about/eol).

Release gate chạy consumer tests trên minimum Node.js 18 và Node.js 24 ở mỗi target, tổng cộng bốn tổ hợp. Kiểm tra Node.js 20 và 22 bổ sung có thể chạy khi runtime sẵn có. Chỉ build native addon một lần mỗi target rồi cài lại cùng tarball trên các runtime; không cần một binary riêng cho từng Node major. Node.js 24 được dùng cho job build và publish; phiên bản npm dùng publish cần đáp ứng yêu cầu hiện hành của registry.

## Chuẩn bị artifact

1. Chọn một commit đã qua toàn bộ Rust, fresh-install parity, compression và TypeScript checks trên cả hai target.
2. Tải `candidate-darwin-arm64` và `candidate-linux-x64` từ cùng workflow run. So sánh SHA256 ZIP với artifact metadata của GitHub trước khi giải nén an toàn.
3. Giữ manifest, ba package archives, ba test reports và `candidate-provenance.json` của từng target. Provenance build ghi repository, commit, workflow, run và target; manifest chứa archive checksums. Khi tải artifact thủ công, ghi thêm artifact ID và SHA256 ZIP đã xác minh; không lưu token hoặc URL tải có thời hạn.
4. Dùng `scripts/prepare-npm-release.py` để ghép binary vào một npm tarball. Assembler phải kiểm tra checksum, evidence fingerprint, cùng version/commit, wrapper/types và đầy đủ LICENSE/NOTICE/third-party notices.
5. Fresh-install tarball cuối bằng npm offline với install scripts bị tắt. Chạy conformance, helper, compression roundtrip và TypeScript checks của release gate. Consumer evidence phải tham chiếu SHA256 của chính tarball sẽ publish; báo cáo bổ sung về repeated calls hoặc worker concurrency cần ghi đúng artifact và môi trường đã chạy.
6. Publish cùng một tarball đúng một lần cho mỗi version. Không publish độc lập hai candidate tarballs cùng package name/version.

Artifact đã build cũng có thể cài local để kiểm tra đúng tarball:

```sh
npm install --offline --ignore-scripts /absolute/path/7mlabs-astrology-0.10.0.tgz
```

## Publish và phát hành tiếp theo

Workflow candidate tạo và kiểm tra artifacts. Workflow publish tách riêng, giới hạn quyền, kiểm tra commit/tag, checksum và release evidence trước khi gọi npm. Version stable như `0.10.0` dùng `latest`; prerelease như `0.10.1-alpha.1` dùng `alpha`. Release tooling phải xác thực SemVer và dist-tag cùng khớp version ở mọi bước assemble, gate, publish và registry verification.

Bước kiểm tra registry sau publish chờ metadata lan truyền trong tối đa 300 giây, với giới hạn số lần thử và timeout theo thời gian còn lại. Nếu bước này thất bại, kiểm tra version/integrity thực tế trước khi chạy lại publish; version npm đã public không được ghi đè.

Scope npm `7mlabs` đã được xác minh; bản alpha đầu phát hành bằng tài khoản npm có 2FA. Trusted publisher đã cấu hình đúng repository `7mlabs/sdk-astro`, workflow `npm-release.yml` và environment `npm-release`. Workflow publish có quyền OIDC riêng và dùng npm đáp ứng [yêu cầu trusted publishing của npm](https://docs.npmjs.com/trusted-publishers/). Bản stable `0.10.0` đã publish thành công bằng OIDC trong [run 37098509933](https://github.com/7mlabs/sdk-astro/actions/runs/37098509933); registry verification và fresh install được xác nhận sau khi metadata lan truyền. Run gốc vẫn báo lỗi do bước registry verification ban đầu hết thời gian chờ sau publish, như phần bằng chứng bên dưới.

Checklist cho maintainer khi tăng version:

1. Đồng bộ version trong `neutral-engine/Cargo.toml`, `bindings/node/package.json`, `bindings/python/pyproject.toml`, Python `__version__` ở `bindings/python/sevenmlabs_astrology/__init__.py`, `bindings/dotnet/SevenMLabs.Astrology.csproj` và package reference của `examples/dotnet/Example.csproj`; cập nhật `neutral-engine/Cargo.lock` và `examples/rust/Cargo.lock`. Đồng bộ version/`peerDependencies` của `examples/mcp/package.json` và `examples/frontend/package.json`, cùng `engineVersion` trong ba bản `rust-manifest.json` ở `bindings/node/third-party/`, `bindings/python/sevenmlabs_astrology/third-party/` và `bindings/dotnet/third-party/`. Stable dùng `0.10.0`; prerelease Python dùng PEP 440 tương ứng, ví dụ `0.10.1a1` cho `0.10.1-alpha.1`. UI renderer có vòng version riêng.
2. Chạy repository checks để xác nhận metadata đồng bộ và source npm vẫn có `private: true`. Builder/test runner đọc version từ metadata đã xác thực để tạo filename tgz/wheel/nupkg và evidence; không duy trì version/filename riêng bằng cách sửa tay trong scripts.
3. Commit/push source mới và đợi toàn bộ candidate CI qua. Có thể chạy workflow `npm release` trên nhánh với input `publish: false` để kiểm tra pipeline trước khi phát hành.
4. Tạo/push tag chính xác `v<version>`, ví dụ `v0.10.1` cho stable tiếp theo hoặc `v0.10.1-alpha.1` cho alpha. Workflow kiểm tra tag, provenance, checksum và bốn consumer reports trước publish. Mỗi version public là bất biến; không publish lại bytes khác với cùng version.
5. Sau publish, kiểm tra registry version, integrity và dist-tag khớp tarball đã qua gate; cài mới trong project độc lập rồi chạy native checksum/natal/query/compression smoke. Cập nhật docs và website bằng artifact/version đã xác minh.

## Bằng chứng bản ổn định 0.10.0

[Run 37098509933](https://github.com/7mlabs/sdk-astro/actions/runs/37098509933) build từ commit `4322f811a7a0956645e6936d63465ef438379899`. Cả bảy jobs candidate/assembly/final consumer qua: hai candidate targets, một npm assembler và bốn final consumer jobs macOS ARM64/Linux x64 × Node 18/24. Mỗi final consumer kiểm tra cùng tarball với 339 conformance cases, đủ 8 query helpers, schema/semantic checks, 27 compression contract cases/13 fixture roundtrips và TypeScript strict. Full release checker cũng xác nhận source, tag, run và đủ bốn consumer reports khớp tarball.

Publish step bằng npm OIDC thành công. Run gốc báo lỗi ở bước registry verification sau publish: registry còn trả 404 hoặc metadata `latest` cũ trong cửa sổ chờ ban đầu. Sau khi metadata lan truyền, kiểm tra thủ công đã xác nhận identity/license, `latest = 0.10.0`, registry SHA512 integrity và checksum khớp chính tarball đã publish. Không coi toàn bộ run gốc là green; các build/test jobs và publish step đã thành công, còn registry được xác minh sau đó.

Tarball `7mlabs-astrology-0.10.0.tgz`: **2.212.282 bytes**, SHA256 `23e71995eb71119acbce39219edcb252d158aac63a3ddcb9b1210a5fe6d8755a`. Fresh consumer cài bằng `npm install @7mlabs/astrology` trên macOS ARM64, Node `24.19.0`, nhận đúng `0.10.0`; package/native checksums, natal, geometry query, individual domains và compact exact roundtrip đều qua. Linux dùng cùng tarball và đã qua final consumer CI; chưa chạy registry smoke riêng trên Linux.

Bằng chứng local của lần xác minh sau publish: `artifacts/npm-stable-release/manifest.json`, `artifacts/npm-stable-release/reports/`, `artifacts/npm-stable-release/registry-verification.json` và `artifacts/npm-stable-registry-consumer/test-results.json`. Artifact/version public giữ bất biến.

## Lịch sử: bằng chứng npm alpha 0.10.0-alpha.1

[CI run 37096125672](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672) xác minh commit `a17850e01507485312e5cb584ed1eb82a786fcbc`: mỗi candidate target qua 108 tests Rust, 339 parity cases giữa năm ngôn ngữ, compression và license gates. Cùng tarball npm cuối được fresh-install offline và chạy 339 cases, query schema/semantic checks, 13 compression roundtrips và TypeScript trên bốn tổ hợp macOS ARM64/Linux x64 × Node 18/24.

Tarball `7mlabs-astrology-0.10.0-alpha.1.tgz` có SHA256 `d8dcfdbaea4e66070a75f9e5ef91d0e6302b33995b364dce40b6a1e28eba9748`. Sau publish, registry SHA512 integrity, legacy checksum và dist-tag `alpha` đã được đối chiếu với tarball này. Cài mới từ npm trên macOS ARM64 đã qua native checksum, natal, geometry và compression roundtrip. Kiểm tra registry thực hiện trên macOS; bằng chứng Linux là consumer tests của cùng tarball trong CI.

Mỗi lần cập nhật tạo version mới, build lại tất cả binary từ cùng commit, chạy lại gate và publish artifact bất biến. GitHub repository chứa source và quy trình; npm là nơi người dùng cài và cập nhật package. Cài SDK và chạy tính toán không cần duy trì server.

Package và engine dùng `AGPL-3.0-only`, bao gồm đầy đủ license text và thông báo Swiss Ephemeris cùng các dependency Rust. [Thông tin license của dự án](../LICENSE) đi kèm mọi release.
