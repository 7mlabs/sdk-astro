# License miễn phí: AGPL-3.0

Owner chọn phương án miễn phí ngày 2026-10-03. Engine, provider integration và SDK mới dùng `AGPL-3.0-only`; không dùng Swiss Ephemeris Professional có phí. Toàn văn giấy phép trong [LICENSE](../LICENSE), thông tin thành phần trong [NOTICE](../NOTICE).

## Phạm vi

| Thành phần | License |
|---|---|
| Rust core, C ABI, Node native addon, bindings Node/Python/.NET, schema và docs mới | AGPL-3.0-only |
| Swiss Ephemeris vendored | Nhánh AGPL của upstream; giữ copyright và license notices |
| Renderer độc lập `examples/frontend/ui` | MIT, có [LICENSE riêng](../examples/frontend/ui/LICENSE) |
| Dependencies bên thứ ba | License gốc được giữ trong notices/inventory |

Source/history của ứng dụng legacy không được import vào repo SDK này. `docs/legacy-source.md` là tài liệu lịch sử.

## Dùng trong dự án

AGPL cho phép dùng, sửa đổi và phân phối theo điều kiện của giấy phép, gồm cả sử dụng thương mại. Khi phân phối covered work, phải giữ notices và cung cấp corresponding source theo các mục 4–6. Khi sửa đổi chương trình và cho người dùng tương tác qua mạng, mục 13 yêu cầu cung cấp cách nhận corresponding source cho những người dùng đó. Các nghĩa vụ cho phần mềm kết hợp với engine phải được xét theo AGPL; UI MIT không thay đổi license của phần tính toán.

Repo công khai cung cấp source engine cùng scripts build và schemas. Người phân phối bản sửa đổi cần cung cấp corresponding source của bản mình phân phối; chỉ dẫn về repo upstream không tự đáp ứng nghĩa vụ cho các thay đổi của họ.

Nguồn chính thức: [GNU AGPL v3](https://www.gnu.org/licenses/agpl-3.0.html), [Swiss Ephemeris programming manual, trang đầu](https://www.astro.com/swisseph-download/doc/swephprg.pdf).

## Đóng gói

Mỗi SDK giữ toàn văn AGPL và NOTICE, vendor Swiss notices, cùng dependency licenses. Package metadata ghi `AGPL-3.0-only`. Python dùng PEP 639 với setuptools 77+; NuGet dùng SPDX license expression. Crates kế thừa license từ workspace. Candidate CI kiểm tra nội dung license thực trong tgz/wheel/nupkg; quyền registry và multi-platform assembly được xử lý ở phase release riêng.
