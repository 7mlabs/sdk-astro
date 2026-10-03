# Phát hành package Node.js

Package [@7mlabs/astrology `0.10.0-alpha.1`](https://www.npmjs.com/package/@7mlabs/astrology) đã phát hành public trên npm ngày 2026-10-03 với dist-tag `alpha`. Source trong `bindings/node/package.json` vẫn đặt `private: true`; chỉ artifact đã qua kiểm tra release được chuyển sang metadata có thể publish. Python, .NET và UI renderer chưa phát hành registry.

## Cài đặt và cập nhật

Lần phát hành thử nghiệm dùng kênh `alpha`:

```sh
npm install @7mlabs/astrology@alpha
```

Để cập nhật chủ động sang alpha mới:

```sh
npm install @7mlabs/astrology@alpha
```

Để tái tạo đúng một phiên bản, chỉ định version và commit lockfile của ứng dụng:

```sh
npm install --save-exact @7mlabs/astrology@0.10.0-alpha.1
```

Package chứa sẵn native addon; người dùng không cần Rust, compiler, server hay bước tải binary sau cài đặt. `npm ci` dùng lockfile để cài lại đúng dependency của dự án. Package alpha chưa được đưa vào kênh `latest`.

## Nền tảng của lần phát hành đầu

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

Metadata đặt minimum Node.js 18. Native adapter sử dụng Node-API v1 và không phụ thuộc V8 API; bản npm cuối đã được kiểm tra thực tế trên Node 18/24 ở cả hai target. [Node-API duy trì ABI giữa các phiên bản Node.js](https://nodejs.org/api/n-api.html).

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
npm install --offline --ignore-scripts /absolute/path/7mlabs-astrology-0.10.0-alpha.1.tgz
```

## Publish và phát hành tiếp theo

Workflow candidate hiện chỉ tạo và kiểm tra artifacts. Workflow publish tách riêng, giới hạn quyền, kiểm tra commit/tag, checksum và release evidence trước khi gọi npm. Lần phát hành scoped package dùng public access và dist-tag `alpha`; chỉ chuyển sang `latest` khi chủ động phát hành bản ổn định.

Scope npm `7mlabs` đã được xác minh và lần phát hành đầu đã hoàn tất bằng tài khoản npm có 2FA. Trusted publisher đã cấu hình đúng repository `7mlabs/sdk-astro`, workflow `npm-release.yml` và environment `npm-release`. Workflow publish có quyền OIDC riêng và dùng npm đáp ứng [yêu cầu trusted publishing của npm](https://docs.npmjs.com/trusted-publishers/); chưa có lần phát hành thực tế bằng OIDC, nên không coi cấu hình này là bằng chứng publish tự động đã chạy thành công. Bản cập nhật phải tăng version trước khi push tag tương ứng, ví dụ `v0.10.0-alpha.2`; không publish lại version hiện có.

Checklist cho maintainer khi tăng version:

1. Đồng bộ version trong `neutral-engine/Cargo.toml`, `bindings/node/package.json`, `bindings/python/pyproject.toml`, `bindings/dotnet/SevenMLabs.Astrology.csproj` và package reference của `examples/dotnet/Example.csproj`; cập nhật `neutral-engine/Cargo.lock` và `examples/rust/Cargo.lock`. Python dùng dạng PEP 440 tương ứng, ví dụ `0.10.0a2` cho `0.10.0-alpha.2`.
2. Cập nhật các version/filename đang khai báo trực tiếp trong `scripts/build-packages.py` và `scripts/test-packages.py`. Hiện builder chưa tự lấy mọi version từ metadata; chỉ sửa version npm sẽ làm candidate build/test không đồng bộ.
3. Commit/push source mới và đợi toàn bộ candidate CI qua. Có thể chạy workflow `npm release` trên nhánh với input `publish: false` để kiểm tra pipeline mà chưa phát hành.
4. Tạo/push tag khớp version mới để chạy publish; kiểm tra registry và cài mới sau khi workflow hoàn tất. Cập nhật docs và website bằng artifact/version đã xác minh.

## Bằng chứng bản phát hành đầu

[CI run 37096125672](https://github.com/7mlabs/sdk-astro/actions/runs/37096125672) xác minh commit `a17850e01507485312e5cb584ed1eb82a786fcbc`: mỗi candidate target qua 108 tests Rust, 339 parity cases giữa năm ngôn ngữ, compression và license gates. Cùng tarball npm cuối được fresh-install offline và chạy 339 cases, query schema/semantic checks, 13 compression roundtrips và TypeScript trên bốn tổ hợp macOS ARM64/Linux x64 × Node 18/24.

Tarball `7mlabs-astrology-0.10.0-alpha.1.tgz` có SHA256 `d8dcfdbaea4e66070a75f9e5ef91d0e6302b33995b364dce40b6a1e28eba9748`. Sau publish, registry SHA512 integrity, legacy checksum và dist-tag `alpha` đã được đối chiếu với tarball này. Cài mới từ npm trên macOS ARM64 đã qua native checksum, natal, geometry và compression roundtrip. Kiểm tra registry thực hiện trên macOS; bằng chứng Linux là consumer tests của cùng tarball trong CI.

Mỗi lần cập nhật tạo version mới, build lại tất cả binary từ cùng commit, chạy lại gate và publish artifact bất biến. GitHub repository chứa source và quy trình; npm là nơi người dùng cài và cập nhật package. Cài SDK và chạy tính toán không cần duy trì server.

Package và engine dùng `AGPL-3.0-only`, bao gồm đầy đủ license text và thông báo Swiss Ephemeris cùng các dependency Rust. [Thông tin license của dự án](../LICENSE) đi kèm mọi release.
