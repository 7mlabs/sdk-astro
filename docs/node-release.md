# Phát hành package Node.js

Package `@7mlabs/astrology` đang được chuẩn bị cho lần phát hành npm đầu tiên. Source trong `bindings/node/package.json` vẫn đặt `private: true`; chỉ artifact đã qua kiểm tra release được chuyển sang metadata có thể publish. Các lệnh cài từ registry bên dưới áp dụng sau khi phiên bản đã được phát hành.

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

Metadata đặt minimum Node.js 18. Native adapter sử dụng Node-API v1 và không phụ thuộc V8 API; cần kiểm tra runtime thực tế của package đã đóng gói để xác nhận tương thích. [Node-API duy trì ABI giữa các phiên bản Node.js](https://nodejs.org/api/n-api.html).

Khuyến nghị ứng dụng mới dùng Node.js 22 hoặc 24 LTS. Node.js 18 và 20 đã EOL tại thời điểm tháng 10/2026; kiểm tra tương thích với chúng không kéo dài hỗ trợ bảo mật của Node.js. [Lịch phát hành Node.js](https://nodejs.org/en/about/previous-releases), [các phiên bản EOL](https://nodejs.org/en/about/eol).

Release gate chạy consumer tests trên minimum Node.js 18 và Node.js 24 ở mỗi target, tổng cộng bốn tổ hợp. Kiểm tra Node.js 20 và 22 bổ sung có thể chạy khi runtime sẵn có. Chỉ build native addon một lần mỗi target rồi cài lại cùng tarball trên các runtime; không cần một binary riêng cho từng Node major. Node.js 24 được dùng cho job build và publish; phiên bản npm dùng publish cần đáp ứng yêu cầu hiện hành của registry.

## Chuẩn bị artifact

1. Chọn một commit đã qua toàn bộ Rust, fresh-install parity, compression và TypeScript checks trên cả hai target.
2. Tải `candidate-darwin-arm64` và `candidate-linux-x64` từ cùng workflow run. So sánh SHA256 ZIP với artifact metadata của GitHub trước khi giải nén an toàn.
3. Giữ manifest, ba package archives, ba test reports và `candidate-provenance.json` của từng target. Provenance ghi repository, commit, workflow, run, artifact ID, target và archive SHA256; không chứa token hoặc URL tải có thời hạn.
4. Dùng `scripts/prepare-npm-release.py` để ghép binary vào một npm tarball. Assembler phải kiểm tra checksum, evidence fingerprint, cùng version/commit, wrapper/types và đầy đủ LICENSE/NOTICE/third-party notices.
5. Fresh-install tarball cuối bằng npm offline với install scripts bị tắt. Chạy conformance, helper, compression roundtrip và TypeScript checks của release gate. Consumer evidence phải tham chiếu SHA256 của chính tarball sẽ publish; báo cáo bổ sung về repeated calls hoặc worker concurrency cần ghi đúng artifact và môi trường đã chạy.
6. Publish cùng một tarball đúng một lần cho mỗi version. Không publish độc lập hai candidate tarballs cùng package name/version.

Artifact đã build có thể cài local trước khi registry sẵn sàng:

```sh
npm install --offline --ignore-scripts /absolute/path/7mlabs-astrology-0.10.0-alpha.1.tgz
```

## Publish và phát hành tiếp theo

Workflow candidate hiện chỉ tạo và kiểm tra artifacts. Workflow publish tách riêng, giới hạn quyền, kiểm tra commit/tag, checksum và release evidence trước khi gọi npm. Lần phát hành scoped package dùng public access và dist-tag `alpha`; chỉ chuyển sang `latest` khi chủ động phát hành bản ổn định.

Tài khoản hoặc organization npm phải có quyền sở hữu scope `7mlabs` và cơ chế xác thực phù hợp. Nếu dùng trusted publishing, cấu hình đúng repository, workflow filename và environment trên npm; job publish dùng OIDC và runtime/npm đáp ứng [yêu cầu trusted publishing của npm](https://docs.npmjs.com/trusted-publishers/). Khi package đầu tiên chưa tồn tại, cần hoàn tất quy trình khởi tạo mà registry hỗ trợ trước khi dựa vào cấu hình trusted publisher cho các lần sau.

Mỗi lần cập nhật tạo version mới, build lại tất cả binary từ cùng commit, chạy lại gate và publish artifact bất biến. GitHub repository chứa source và quy trình; npm là nơi người dùng cài và cập nhật package. Cài SDK và chạy tính toán không cần duy trì server.

Package và engine dùng `AGPL-3.0-only`, bao gồm đầy đủ license text và thông báo Swiss Ephemeris cùng các dependency Rust. [Thông tin license của dự án](../LICENSE) đi kèm mọi release.
