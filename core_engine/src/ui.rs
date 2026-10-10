//! Module Giao diện người dùng hiện đại (Modern Windows UI & System Tray)
//! - Icon Chú heo hồng (Pink Piggy Icon):
//!   + Khi bật tiếng Việt: Mặt chú heo màu hồng chữ "V"
//!   + Khi tắt tiếng Việt: Mặt chú heo màu xám chữ "E"
//! - Click chuột trái vào Icon khay hệ thống -> Đổi nhanh giữa V và E
//! - Click chuột phải vào Icon khay hệ thống -> Menu ngữ cảnh đầy đủ
//! - Tùy chọn khởi động cùng Windows (Auto Start on Windows Boot)
//! - Không hiện cửa sổ terminal đen (#![windows_subsystem = "windows"])

use vnkey_engine::InputMethod;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, Ellipse, FillRect, GetDC, GetStockObject,
    InvalidateRect, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
    DT_CENTER, DT_LEFT, DT_SINGLELINE, DT_VCENTER, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, FW_NORMAL, HBRUSH, HGDIOBJ,
    WHITE_BRUSH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW,
    GetSystemMetrics, ICONINFO, LoadCursorW, PostQuitMessage, RegisterClassExW,
    SendMessageW, SetForegroundWindow, ShowWindow, TrackPopupMenu,
    BM_GETCHECK, BM_SETCHECK, CS_HREDRAW, CS_VREDRAW, HMENU, IDC_ARROW,
    MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SW_HIDE, SW_RESTORE, SW_SHOW, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP,
    WM_USER, WNDCLASSEXW, WS_CAPTION, WS_CHILD, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use crate::windows_hook::{get_current_method, is_viet_mode, set_input_method, set_ui_hwnd, toggle_viet_mode};

const WM_TRAYICON: u32 = WM_USER + 101;
const WM_STATE_CHANGED: u32 = WM_USER + 102;
const ID_TRAY_OPEN: usize = 1001;
const ID_TRAY_TOGGLE_VE: usize = 1002;
const ID_TRAY_TELEX: usize = 1003;
const ID_TRAY_VNI: usize = 1004;
const ID_TRAY_AUTORUN: usize = 1005;
const ID_TRAY_EXIT: usize = 1006;

const ID_BTN_TELEX: usize = 2001;
const ID_BTN_VNI: usize = 2002;
const ID_BTN_TOGGLE_VE: usize = 2003;
const ID_CHK_AUTORUN: usize = 2004;
const ID_BTN_HIDE: usize = 2005;

// Hằng số checkbox Win32
const BST_CHECKED_VAL: isize = 1;
const BST_UNCHECKED_VAL: isize = 0;

/// Kiểm tra xem ứng dụng đã được đặt tự khởi động cùng Windows chưa
pub fn is_autostart_enabled() -> bool {
    unsafe {
        let subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, Some(0), KEY_READ, &mut hkey).is_ok() {
            let value_name = w!("BurinSmartKey");
            let mut buf_len = 0u32;
            let res = RegQueryValueExW(hkey, value_name, None, None, None, Some(&mut buf_len));
            let _ = RegCloseKey(hkey);
            return res.is_ok();
        }
        false
    }
}

/// Đặt hoặc hủy tự khởi động cùng Windows
pub fn set_autostart(enable: bool) -> bool {
    unsafe {
        let subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, Some(0), KEY_WRITE, &mut hkey).is_ok() {
            let value_name = w!("BurinSmartKey");
            if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = format!("\"{}\"", exe_path.to_string_lossy());
                    let wide_path = path_str.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
                    let bytes = std::slice::from_raw_parts(wide_path.as_ptr() as *const u8, wide_path.len() * 2);
                    let _ = RegSetValueExW(hkey, value_name, Some(0), REG_SZ, Some(bytes));
                }
            } else {
                let _ = RegDeleteValueW(hkey, value_name);
            }
            let _ = RegCloseKey(hkey);
            return true;
        }
        false
    }
}

/// Vẽ icon mặt chú heo Burin:
/// - Khi bật tiếng Việt: Mặt chú heo màu hồng phấn tươi tắn (#FFA8B6, tai #FF7B90, má hồng #FF8FA3)
/// - Khi tắt tiếng Việt: Mặt chú heo màu xám (#B0B0B0, tai #8E8E8E, má #A0A0A0)
/// - Icon to rõ nét, tràn khung 32x32, không có chữ V/E
unsafe fn create_piggy_icon(is_v: bool) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    let size = 32;
    let hdc_screen = unsafe { GetDC(None) };
    let hdc = unsafe { CreateCompatibleDC(Some(hdc_screen)) };
    let hbm_color = unsafe { CreateCompatibleBitmap(hdc_screen, size, size) };
    let hbm_mask = unsafe { CreateCompatibleBitmap(hdc_screen, size, size) };

    // 1. Tạo mask (trong suốt nền ngoài hình elip chú heo)
    let old_bmp = unsafe { SelectObject(hdc, HGDIOBJ(hbm_mask.0)) };
    let black_brush = unsafe { CreateSolidBrush(COLORREF(0x00000000)) };
    let white_brush = unsafe { CreateSolidBrush(COLORREF(0x00FFFFFF)) };
    let rect_all = RECT { left: 0, top: 0, right: size, bottom: size };
    unsafe { FillRect(hdc, &rect_all, white_brush) };

    // Vùng opaque trên mask (màu đen = hiển thị màu color bitmap)
    unsafe { SelectObject(hdc, HGDIOBJ(black_brush.0)) };
    // Tai trái và phải trên mask
    unsafe { Ellipse(hdc, 2, 1, 13, 13) };
    unsafe { Ellipse(hdc, 19, 1, 30, 13) };
    // Đầu heo trên mask (chiếm trọn 1 đến 31)
    unsafe { Ellipse(hdc, 1, 5, 31, 31) };

    // 2. Vẽ màu chú heo Burin
    unsafe { SelectObject(hdc, HGDIOBJ(hbm_color.0)) };
    unsafe { FillRect(hdc, &rect_all, black_brush) };

    // Bảng màu: Bật = Hồng phấn tươi tắn, Tắt = Xám
    let face_color = if is_v { COLORREF(0x00B5A6FF) } else { COLORREF(0x00B8B8B8) }; // #FFA6B5 (BGR)
    let ear_color  = if is_v { COLORREF(0x008E70FF) } else { COLORREF(0x008A8A8A) }; // #FF708E (BGR)
    let inner_ear  = if is_v { COLORREF(0x00D0C0FF) } else { COLORREF(0x00CECECE) };
    let cheek_color= if is_v { COLORREF(0x009080FF) } else { COLORREF(0x00A0A0A0) };
    let snout_color= if is_v { COLORREF(0x007860FF) } else { COLORREF(0x00757575) }; // #FF6078 (BGR)

    let brush_face = unsafe { CreateSolidBrush(face_color) };
    let brush_ear = unsafe { CreateSolidBrush(ear_color) };
    let brush_inner_ear = unsafe { CreateSolidBrush(inner_ear) };
    let brush_snout = unsafe { CreateSolidBrush(snout_color) };
    let brush_cheek = unsafe { CreateSolidBrush(cheek_color) };
    let brush_eye = unsafe { CreateSolidBrush(COLORREF(0x001A1A1A)) };
    let brush_white = unsafe { CreateSolidBrush(COLORREF(0x00FFFFFF)) };

    // Hai tai ngoài
    unsafe { SelectObject(hdc, HGDIOBJ(brush_ear.0)) };
    unsafe { Ellipse(hdc, 2, 1, 13, 13) };
    unsafe { Ellipse(hdc, 19, 1, 30, 13) };

    // Lòng tai trong
    unsafe { SelectObject(hdc, HGDIOBJ(brush_inner_ear.0)) };
    unsafe { Ellipse(hdc, 4, 3, 11, 11) };
    unsafe { Ellipse(hdc, 21, 3, 28, 11) };

    // Khuôn mặt tròn lớn, rõ nét
    unsafe { SelectObject(hdc, HGDIOBJ(brush_face.0)) };
    unsafe { Ellipse(hdc, 1, 5, 31, 31) };

    // Hai má hồng xinh xắn
    unsafe { SelectObject(hdc, HGDIOBJ(brush_cheek.0)) };
    unsafe { Ellipse(hdc, 3, 17, 8, 22) };
    unsafe { Ellipse(hdc, 24, 17, 29, 22) };

    // Hai mắt đen to tròn
    unsafe { SelectObject(hdc, HGDIOBJ(brush_eye.0)) };
    unsafe { Ellipse(hdc, 7, 12, 11, 17) };
    unsafe { Ellipse(hdc, 21, 12, 25, 17) };

    // Đốm sáng trong mắt
    unsafe { SelectObject(hdc, HGDIOBJ(brush_white.0)) };
    unsafe { Ellipse(hdc, 8, 13, 10, 15) };
    unsafe { Ellipse(hdc, 22, 13, 24, 15) };

    // Mũi heo to tròn ở trung tâm
    unsafe { SelectObject(hdc, HGDIOBJ(brush_snout.0)) };
    unsafe { Ellipse(hdc, 10, 16, 22, 25) };

    // Hai lỗ mũi heo
    unsafe { SelectObject(hdc, HGDIOBJ(brush_eye.0)) };
    unsafe { Ellipse(hdc, 13, 19, 15, 22) };
    unsafe { Ellipse(hdc, 17, 19, 19, 22) };

    // Dọn dẹp GDI objects
    unsafe { SelectObject(hdc, old_bmp) };
    unsafe {
        let _ = DeleteObject(HGDIOBJ(brush_face.0));
        let _ = DeleteObject(HGDIOBJ(brush_ear.0));
        let _ = DeleteObject(HGDIOBJ(brush_inner_ear.0));
        let _ = DeleteObject(HGDIOBJ(brush_snout.0));
        let _ = DeleteObject(HGDIOBJ(brush_cheek.0));
        let _ = DeleteObject(HGDIOBJ(brush_eye.0));
        let _ = DeleteObject(HGDIOBJ(brush_white.0));
        let _ = DeleteObject(HGDIOBJ(black_brush.0));
        let _ = DeleteObject(HGDIOBJ(white_brush.0));
        let _ = DeleteDC(hdc);
        ReleaseDC(None, hdc_screen);
    }

    let mut icon_info = ICONINFO {
        fIcon: windows::core::BOOL(1),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = unsafe { CreateIconIndirect(&mut icon_info).unwrap_or_default() };
    unsafe {
        let _ = DeleteObject(HGDIOBJ(hbm_color.0));
        let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
    }

    hicon
}

/// Khởi chạy ứng dụng với giao diện đồ họa hiện đại và khay hệ thống
pub fn run_ui_app() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let class_name = w!("BurinSmartKey_MainUI");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: create_piggy_icon(true),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: create_piggy_icon(true),
        };

        let _ = RegisterClassExW(&wc);

        let width = 430;
        let height = 450;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Burin Smart Key"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            None,
        ).expect("Không thể tạo cửa sổ chính");

        set_ui_hwnd(hwnd);
        update_tray_icon(hwnd, is_viet_mode(), true);
        let _ = ShowWindow(hwnd, SW_SHOW);

        // Cài đặt bàn phím hook
        crate::windows_hook::install_hooks();

        // Message pump chính
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            DispatchMessageW(&msg);
        }

        crate::windows_hook::uninstall_hooks();
        remove_tray_icon(hwnd);
    }
}

/// Cập nhật icon System Tray (Thêm mới hoặc Đổi V/E)
unsafe fn update_tray_icon(hwnd: HWND, is_v: bool, is_add: bool) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = unsafe { create_piggy_icon(is_v) };

    let mode_str = if is_v { "BẬT Tiếng Việt [Mặt hồng]" } else { "TẮT Tiếng Anh [Mặt xám]" };
    let tip = format!("Burin Smart Key: {}\n• Bật/Tắt: Ctrl + Shift\n• Đổi kiểu: F9\0", mode_str).encode_utf16().collect::<Vec<u16>>();
    let len = tip.len().min(nid.szTip.len());
    nid.szTip[..len].copy_from_slice(&tip[..len]);

    let action = if is_add { NIM_ADD } else { NIM_MODIFY };
    unsafe {
        let _ = Shell_NotifyIconW(action, &nid);
    }
}

/// Gỡ icon khỏi System Tray
unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

/// Hiển thị Context Menu chuột phải tại Khay hệ thống
unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = unsafe { CreatePopupMenu().unwrap() };
    let cur_method = get_current_method();
    let is_v = is_viet_mode();

    unsafe {
        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, w!("Mở bảng điều khiển..."));
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

        let ve_label = if is_v {
            w!("Bật/Tắt [V/E] (Ctrl + Shift)   [Đang BẬT]")
        } else {
            w!("Bật/Tắt [V/E] (Ctrl + Shift)   [Đang TẮT]")
        };
        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_TOGGLE_VE, ve_label);
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

        let telex_flag = if cur_method == InputMethod::Telex { MF_CHECKED } else { MF_UNCHECKED };
        let vni_flag = if cur_method == InputMethod::Vni { MF_CHECKED } else { MF_UNCHECKED };

        let _ = AppendMenuW(hmenu, MF_STRING | telex_flag, ID_TRAY_TELEX, w!("Kiểu gõ TELEX   (F9)"));
        let _ = AppendMenuW(hmenu, MF_STRING | vni_flag, ID_TRAY_VNI, w!("Kiểu gõ VNI   (F9)"));
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

        let autorun_flag = if is_autostart_enabled() { MF_CHECKED } else { MF_UNCHECKED };
        let _ = AppendMenuW(hmenu, MF_STRING | autorun_flag, ID_TRAY_AUTORUN, w!("Khởi động cùng Windows"));
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, w!("Thoát chương trình"));

        let mut pt = windows::Win32::Foundation::POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            hmenu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            pt.x,
            pt.y,
            Some(0),
            hwnd,
            None,
        );
        let _ = DestroyMenu(hmenu);
    }
}

/// Window Procedure xử lý sự kiện giao diện
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let instance = unsafe { GetModuleHandleW(None).unwrap() };

            unsafe {
                // Nút bấm TELEX
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("TELEX"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    30,
                    88,
                    110,
                    40,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_TELEX as _)),
                    Some(instance.into()),
                    None,
                );

                // Nút bấm VNI
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("VNI"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    150,
                    88,
                    110,
                    40,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_VNI as _)),
                    Some(instance.into()),
                    None,
                );

                // Nút Chuyển đổi V/E
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Bật / Tắt [V/E]"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    270,
                    88,
                    120,
                    40,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_TOGGLE_VE as _)),
                    Some(instance.into()),
                    None,
                );

                // Checkbox Khởi động cùng Windows
                let chk_hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Khởi động cùng Windows khi bật máy"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(windows::Win32::UI::WindowsAndMessaging::BS_AUTOCHECKBOX as _),
                    30,
                    315,
                    300,
                    25,
                    Some(hwnd),
                    Some(HMENU(ID_CHK_AUTORUN as _)),
                    Some(instance.into()),
                    None,
                ).unwrap();

                let is_auto = is_autostart_enabled();
                let check_state = if is_auto { BST_CHECKED_VAL } else { BST_UNCHECKED_VAL };
                let _ = SendMessageW(chk_hwnd, BM_SETCHECK, Some(WPARAM(check_state as _)), Some(LPARAM(0)));

                // Nút Thu nhỏ xuống khay hệ thống
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Thu nhỏ xuống khay hệ thống"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    30,
                    348,
                    360,
                    38,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_HIDE as _)),
                    Some(instance.into()),
                    None,
                );
            }

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
            let hdc = unsafe { windows::Win32::Graphics::Gdi::BeginPaint(hwnd, &mut ps) };

            let to_wide_null = |s: &str| -> Vec<u16> {
                s.encode_utf16().chain(std::iter::once(0)).collect()
            };

            unsafe {
                // 1. Nền Header Hiện đại
                let header_rect = RECT { left: 0, top: 0, right: 430, bottom: 65 };
                let header_brush = CreateSolidBrush(COLORREF(0x006548F4));
                FillRect(hdc, &header_rect, header_brush);
                let _ = DeleteObject(HGDIOBJ(header_brush.0));

                // Tiêu đề App
                SetBkMode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                let font_title = CreateFontW(
                    19, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                let old_font = SelectObject(hdc, HGDIOBJ(font_title.0));
                let mut title_rect = RECT { left: 24, top: 12, right: 400, bottom: 38 };
                let mut title_w = to_wide_null("BURIN SMART KEY");
                DrawTextW(hdc, &mut title_w, &mut title_rect, DT_LEFT | DT_SINGLELINE);

                // Phụ đề
                let font_sub = CreateFontW(
                    13, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_sub.0));
                SetTextColor(hdc, COLORREF(0x00FEE2E2));
                let mut sub_rect = RECT { left: 24, top: 38, right: 400, bottom: 58 };
                let mut sub_w = to_wide_null("Bộ gõ tiếng Việt thông minh thế hệ mới");
                DrawTextW(hdc, &mut sub_w, &mut sub_rect, DT_LEFT | DT_SINGLELINE);

                // 2. Nội dung thân
                SelectObject(hdc, HGDIOBJ(font_title.0));
                SetTextColor(hdc, COLORREF(0x001E293B));
                let mut label_rect = RECT { left: 30, top: 70, right: 390, bottom: 86 };
                let mut label_w = to_wide_null("TÙY CHỈNH KIỂU GÕ & CHẾ ĐỘ:");
                DrawTextW(hdc, &mut label_w, &mut label_rect, DT_LEFT | DT_SINGLELINE);

                // Trạng thái hiện tại
                let cur_m = get_current_method();
                let is_v = is_viet_mode();
                let mode_name = if is_v { "BẬT [Tiếng Việt] (Mặt hồng)" } else { "TẮT [Tiếng Anh] (Mặt xám)" };
                let method_name = match cur_m {
                    InputMethod::Telex => "TELEX",
                    InputMethod::Vni => "VNI",
                    _ => "TELEX",
                };
                let status_text = format!("Hiện tại: {} | {}", method_name, mode_name);

                SelectObject(hdc, HGDIOBJ(font_sub.0));
                let status_color = if is_v { COLORREF(0x002563EB) } else { COLORREF(0x00707070) };
                SetTextColor(hdc, status_color);
                let mut status_rect = RECT { left: 30, top: 134, right: 390, bottom: 154 };
                let mut status_w = to_wide_null(&status_text);
                DrawTextW(hdc, &mut status_w, &mut status_rect, DT_LEFT | DT_SINGLELINE);

                // 3. Khung Hướng dẫn Phím tắt & Trạng thái Icon (Card)
                let card_rect = RECT { left: 30, top: 158, right: 390, bottom: 305 };
                let card_brush = CreateSolidBrush(COLORREF(0x00FFF5F5));
                FillRect(hdc, &card_rect, card_brush);
                let _ = DeleteObject(HGDIOBJ(card_brush.0));

                let font_bold_sub = CreateFontW(
                    13, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );

                // Tiêu đề khung phím tắt
                SelectObject(hdc, HGDIOBJ(font_bold_sub.0));
                SetTextColor(hdc, COLORREF(0x000F766E));
                let mut hk_title_rect = RECT { left: 40, top: 164, right: 380, bottom: 182 };
                let mut hk_title_w = to_wide_null("📌 BẢNG TRA CỨU PHÍM TẮT & BIỂU TƯỢNG:");
                DrawTextW(hdc, &mut hk_title_w, &mut hk_title_rect, DT_LEFT | DT_SINGLELINE);

                SelectObject(hdc, HGDIOBJ(font_sub.0));
                SetTextColor(hdc, COLORREF(0x001F2937));

                let mut row1_rect = RECT { left: 42, top: 186, right: 380, bottom: 204 };
                let mut row1_w = to_wide_null("• Ctrl + Shift : Bật / Tắt bộ gõ (Tiếng Việt <-> Tiếng Anh)");
                DrawTextW(hdc, &mut row1_w, &mut row1_rect, DT_LEFT | DT_SINGLELINE);

                let mut row2_rect = RECT { left: 42, top: 206, right: 380, bottom: 224 };
                let mut row2_w = to_wide_null("• F9           : Chuyển đổi qua lại giữa TELEX và VNI");
                DrawTextW(hdc, &mut row2_w, &mut row2_rect, DT_LEFT | DT_SINGLELINE);

                let mut row3_rect = RECT { left: 42, top: 226, right: 380, bottom: 244 };
                let mut row3_w = to_wide_null("• Mặt heo HỒNG : Đang BẬT gõ tiếng Việt");
                DrawTextW(hdc, &mut row3_w, &mut row3_rect, DT_LEFT | DT_SINGLELINE);

                let mut row4_rect = RECT { left: 42, top: 246, right: 380, bottom: 264 };
                let mut row4_w = to_wide_null("• Mặt heo XÁM  : Đang TẮT gõ tiếng Việt (chế độ tiếng Anh)");
                DrawTextW(hdc, &mut row4_w, &mut row4_rect, DT_LEFT | DT_SINGLELINE);

                SetTextColor(hdc, COLORREF(0x00059669));
                let mut row5_rect = RECT { left: 42, top: 270, right: 380, bottom: 290 };
                let mut row5_w = to_wide_null("💡 Click chuột trái vào icon khay hệ thống để bật/tắt nhanh!");
                DrawTextW(hdc, &mut row5_w, &mut row5_rect, DT_LEFT | DT_SINGLELINE);

                // Dọn dẹp
                SelectObject(hdc, old_font);
                let _ = DeleteObject(HGDIOBJ(font_title.0));
                let _ = DeleteObject(HGDIOBJ(font_sub.0));
                let _ = DeleteObject(HGDIOBJ(font_bold_sub.0));
                let _ = windows::Win32::Graphics::Gdi::EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = wparam.0 as usize & 0xFFFF;
            unsafe {
                match id {
                    ID_BTN_TELEX | ID_TRAY_TELEX => {
                        set_input_method(InputMethod::Telex);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_BTN_VNI | ID_TRAY_VNI => {
                        set_input_method(InputMethod::Vni);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_BTN_TOGGLE_VE | ID_TRAY_TOGGLE_VE => {
                        let new_v = toggle_viet_mode();
                        update_tray_icon(hwnd, new_v, false);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_CHK_AUTORUN => {
                        let cur = is_autostart_enabled();
                        set_autostart(!cur);
                    }
                    ID_TRAY_AUTORUN => {
                        let cur = is_autostart_enabled();
                        set_autostart(!cur);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_BTN_HIDE => {
                        let _ = ShowWindow(hwnd, SW_HIDE);
                    }
                    ID_TRAY_OPEN => {
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                        let _ = SetForegroundWindow(hwnd);
                    }
                    ID_TRAY_EXIT => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }

        WM_TRAYICON => {
            let event = lparam.0 as u32;
            unsafe {
                if event == WM_RBUTTONUP {
                    show_tray_menu(hwnd);
                } else if event == WM_LBUTTONUP {
                    // Click chuột trái vào chú heo -> Đổi trạng thái V/E ngay lập tức giống Unikey!
                    let new_v = toggle_viet_mode();
                    update_tray_icon(hwnd, new_v, false);
                    let _ = InvalidateRect(Some(hwnd), None, true);
                } else if event == WM_LBUTTONDBLCLK {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            LRESULT(0)
        }

        WM_STATE_CHANGED => {
            unsafe {
                update_tray_icon(hwnd, is_viet_mode(), false);
                let _ = InvalidateRect(Some(hwnd), None, true);
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
