//! Module Giao diện người dùng hiện đại chuẩn Mockup (Burin Smart Key UI & System Tray)
//! - Header Gradient màu cam hồng với Icon Heo Tròn nổi bật
//! - Toggle Pill chọn kiểu gõ TELEX / VNI
//! - Nút chuyển trạng thái Tiếng Việt (V) / Tiếng Anh (E) màu đỏ cam với icon heo
//! - Khung phím tắt bo tròn viền xám mềm mại, các phím bấm dạng bàn phím nổi bật (Ctrl, Shift, F9)
//! - Nút "Thu nhỏ xuống khay hệ thống" màu hồng nhạt viền tinh tế
//! - Nút Close 'X' ở góc trên bên phải
//! - Không hiện cửa sổ terminal đen (#![windows_subsystem = "windows"])

use vnkey_engine::InputMethod;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, Ellipse, FillRect, GetDC,
    GetStockObject, InvalidateRect, ReleaseDC, RoundRect, SelectObject, SetBkMode,
    SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DT_CENTER,
    DT_LEFT, DT_SINGLELINE, DT_VCENTER, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HGDIOBJ,
    HPEN, PS_SOLID, WHITE_BRUSH,
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
    SetForegroundWindow, ShowWindow, TrackPopupMenu,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, IDC_HAND,
    MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SW_HIDE, SW_RESTORE, SW_SHOW, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WM_CLOSE, WM_COMMAND,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_NCHITTEST, WM_PAINT, WM_RBUTTONUP, WM_SETCURSOR,
    WM_USER, WNDCLASSEXW, WS_POPUP, WS_VISIBLE,
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

// Kích thước cửa sổ (mở rộng để chứa font chữ to rõ nét, thoáng đãng)
const WIN_WIDTH: i32 = 500;
const WIN_HEIGHT: i32 = 580;

// Các tọa độ vùng tương tác (Hit Test)
const RECT_CLOSE: RECT = RECT { left: 456, top: 12, right: 486, bottom: 42 };
const RECT_TAB_TELEX: RECT = RECT { left: 32, top: 144, right: 142, bottom: 184 };
const RECT_TAB_VNI: RECT = RECT { left: 142, top: 144, right: 252, bottom: 184 };
const RECT_BTN_TOGGLE_VE: RECT = RECT { left: 298, top: 142, right: 468, bottom: 186 };
const RECT_CHK_STARTUP: RECT = RECT { left: 32, top: 440, right: 350, bottom: 472 };
const RECT_BTN_MINIMIZE: RECT = RECT { left: 32, top: 486, right: 468, bottom: 538 };

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

/// Vẽ khuôn mặt chú heo Burin siêu dễ thương tại tọa độ bất kỳ trên HDC
unsafe fn draw_piggy_face(hdc: windows::Win32::Graphics::Gdi::HDC, cx: i32, cy: i32, r: i32, is_v: bool) {
    // Màu sắc tươi sáng, ngọt ngào (Kawaii Pink)
    let face_color   = if is_v { COLORREF(0x00C3B2FF) } else { COLORREF(0x00D0D0D0) }; // Hồng phấn tươi mịn #FFB2C3
    let ear_color    = if is_v { COLORREF(0x00B4A0FF) } else { COLORREF(0x00B8B8B8) }; // Tai ngoài #FFA0B4
    let inner_ear    = if is_v { COLORREF(0x00DAAEFF) } else { COLORREF(0x00E0E0E0) }; // Tai trong hồng sáng #FFAEDA
    let cheek_color  = if is_v { COLORREF(0x009BA2FF) } else { COLORREF(0x00BEBEBE) }; // Má hồng đào ngọt #FFA29B
    let snout_color  = if is_v { COLORREF(0x00A58EFF) } else { COLORREF(0x00A8A8A8) }; // Mũi heo xinh xắn #FF8EA5
    let snout_light  = if is_v { COLORREF(0x00C0B0FF) } else { COLORREF(0x00C8C8C8) }; // Highlight sống mũi
    let eye_color    = COLORREF(0x00261E2D); // Nâu socola đen ấm #2D1E26
    let nostril_color= if is_v { COLORREF(0x005F46A0) } else { COLORREF(0x00606060) }; // Lỗ mũi đỏ hồng rượu nhẹ
    let white_color  = COLORREF(0x00FFFFFF);

    let brush_face = CreateSolidBrush(face_color);
    let brush_ear = CreateSolidBrush(ear_color);
    let brush_inner_ear = CreateSolidBrush(inner_ear);
    let brush_snout = CreateSolidBrush(snout_color);
    let brush_snout_light = CreateSolidBrush(snout_light);
    let brush_cheek = CreateSolidBrush(cheek_color);
    let brush_eye = CreateSolidBrush(eye_color);
    let brush_nostril = CreateSolidBrush(nostril_color);
    let brush_white = CreateSolidBrush(white_color);

    let pen_null = CreatePen(PS_SOLID, 0, face_color);
    let old_pen = SelectObject(hdc, HGDIOBJ(pen_null.0));

    // 1. Tai ngoài vểnh tự nhiên
    let _ = SelectObject(hdc, HGDIOBJ(brush_ear.0));
    let _ = Ellipse(hdc, cx - r * 85 / 100, cy - r * 95 / 100, cx - r * 20 / 100, cy - r * 15 / 100);
    let _ = Ellipse(hdc, cx + r * 20 / 100, cy - r * 95 / 100, cx + r * 85 / 100, cy - r * 15 / 100);

    // Tai trong hồng phấn
    let _ = SelectObject(hdc, HGDIOBJ(brush_inner_ear.0));
    let _ = Ellipse(hdc, cx - r * 75 / 100, cy - r * 85 / 100, cx - r * 30 / 100, cy - r * 25 / 100);
    let _ = Ellipse(hdc, cx + r * 30 / 100, cy - r * 85 / 100, cx + r * 75 / 100, cy - r * 25 / 100);

    // 2. Khuôn mặt tròn đầy phúng phính
    let _ = SelectObject(hdc, HGDIOBJ(brush_face.0));
    let _ = Ellipse(hdc, cx - r, cy - r * 70 / 100, cx + r, cy + r * 95 / 100);

    // 3. Đôi má hồng đào to tròn đáng yêu
    let _ = SelectObject(hdc, HGDIOBJ(brush_cheek.0));
    let _ = Ellipse(hdc, cx - r * 85 / 100, cy + r * 15 / 100, cx - r * 45 / 100, cy + r * 55 / 100);
    let _ = Ellipse(hdc, cx + r * 45 / 100, cy + r * 15 / 100, cx + r * 85 / 100, cy + r * 55 / 100);

    // 4. Mắt to tròn long lanh kiểu anime
    let _ = SelectObject(hdc, HGDIOBJ(brush_eye.0));
    let _ = Ellipse(hdc, cx - r * 55 / 100, cy - r * 15 / 100, cx - r * 25 / 100, cy + r * 25 / 100);
    let _ = Ellipse(hdc, cx + r * 25 / 100, cy - r * 15 / 100, cx + r * 55 / 100, cy + r * 25 / 100);

    // Đốm sáng to long lanh góc trên
    let _ = SelectObject(hdc, HGDIOBJ(brush_white.0));
    let _ = Ellipse(hdc, cx - r * 50 / 100, cy - r * 10 / 100, cx - r * 35 / 100, cy + r * 5 / 100);
    let _ = Ellipse(hdc, cx + r * 30 / 100, cy - r * 10 / 100, cx + r * 45 / 100, cy + r * 5 / 100);

    // Đốm sáng nhỏ góc dưới
    if r >= 14 {
        let _ = Ellipse(hdc, cx - r * 33 / 100, cy + r * 8 / 100, cx - r * 27 / 100, cy + r * 16 / 100);
        let _ = Ellipse(hdc, cx + r * 47 / 100, cy + r * 8 / 100, cx + r * 53 / 100, cy + r * 16 / 100);
    }

    // 5. Chiếc mũi heo bầu bĩnh xinh xắn
    let _ = SelectObject(hdc, HGDIOBJ(brush_snout.0));
    let _ = Ellipse(hdc, cx - r * 40 / 100, cy + r * 10 / 100, cx + r * 40 / 100, cy + r * 65 / 100);

    // Vệt highlight nhẹ trên mũi
    let _ = SelectObject(hdc, HGDIOBJ(brush_snout_light.0));
    let _ = Ellipse(hdc, cx - r * 25 / 100, cy + r * 14 / 100, cx + r * 25 / 100, cy + r * 32 / 100);

    // Lỗ mũi nhỏ xinh
    let _ = SelectObject(hdc, HGDIOBJ(brush_nostril.0));
    let _ = Ellipse(hdc, cx - r * 22 / 100, cy + r * 32 / 100, cx - r * 8 / 100, cy + r * 52 / 100);
    let _ = Ellipse(hdc, cx + r * 8 / 100, cy + r * 32 / 100, cx + r * 22 / 100, cy + r * 52 / 100);

    // Dọn dẹp
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(HGDIOBJ(pen_null.0));
    let _ = DeleteObject(HGDIOBJ(brush_face.0));
    let _ = DeleteObject(HGDIOBJ(brush_ear.0));
    let _ = DeleteObject(HGDIOBJ(brush_inner_ear.0));
    let _ = DeleteObject(HGDIOBJ(brush_snout.0));
    let _ = DeleteObject(HGDIOBJ(brush_snout_light.0));
    let _ = DeleteObject(HGDIOBJ(brush_cheek.0));
    let _ = DeleteObject(HGDIOBJ(brush_eye.0));
    let _ = DeleteObject(HGDIOBJ(brush_nostril.0));
    let _ = DeleteObject(HGDIOBJ(brush_white.0));
}

/// Tạo Icon khay hệ thống (Tray Icon) 32x32 sắc nét với kênh Alpha 32-bit hoàn hảo
unsafe fn create_piggy_icon(is_v: bool) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use crate::tray_icons::{TRAY_V_32_DATA, TRAY_E_32_DATA, TRAY_V_32_W, TRAY_V_32_H};

    let w = TRAY_V_32_W;
    let h = TRAY_V_32_H;
    let raw_rgba = if is_v { TRAY_V_32_DATA } else { TRAY_E_32_DATA };

    let hdc_screen = GetDC(None);

    // Chuẩn bị BITMAPINFO cho 32-bit ARGB (Top-down DIB: height âm)
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: (w * h * 4) as u32,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default(); 1],
    };

    let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbm_color = CreateDIBSection(
        Some(hdc_screen),
        &bmi,
        DIB_RGB_COLORS,
        &mut bits_ptr,
        None,
        0,
    ).unwrap();

    // Điền dữ liệu điểm ảnh ARGB premultiplied
    if !bits_ptr.is_null() {
        let dest = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, (w * h * 4) as usize);
        for i in 0..(w * h) as usize {
            let r = raw_rgba[i * 4] as u32;
            let g = raw_rgba[i * 4 + 1] as u32;
            let b = raw_rgba[i * 4 + 2] as u32;
            let a = raw_rgba[i * 4 + 3] as u32;

            // Premultiply alpha cho Windows DIB
            let pr = ((r * a + 127) / 255) as u8;
            let pg = ((g * a + 127) / 255) as u8;
            let pb = ((b * a + 127) / 255) as u8;

            dest[i * 4 + 0] = pb; // Blue
            dest[i * 4 + 1] = pg; // Green
            dest[i * 4 + 2] = pr; // Red
            dest[i * 4 + 3] = a as u8; // Alpha
        }
    }

    // Tạo monochrome mask (1-bit mask: 0 cho vùng có vẽ, 1 cho vùng trong suốt)
    let hbm_mask = CreateCompatibleBitmap(hdc_screen, w, h);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
    let old_bmp = SelectObject(hdc_mem, HGDIOBJ(hbm_mask.0));

    let black_brush = CreateSolidBrush(COLORREF(0x00000000));
    let white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let rect_all = RECT { left: 0, top: 0, right: w, bottom: h };
    FillRect(hdc_mem, &rect_all, white_brush);

    // Với các pixel có alpha > 0, tô màu đen trên mask để Windows hiển thị màu ARGB
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4 + 3) as usize;
            if raw_rgba[idx] > 30 {
                let pixel_rect = RECT { left: x, top: y, right: x + 1, bottom: y + 1 };
                FillRect(hdc_mem, &pixel_rect, black_brush);
            }
        }
    }

    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(HGDIOBJ(black_brush.0));
    let _ = DeleteObject(HGDIOBJ(white_brush.0));
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    let mut icon_info = ICONINFO {
        fIcon: windows::core::BOOL(1),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = CreateIconIndirect(&mut icon_info).unwrap_or_default();

    let _ = DeleteObject(HGDIOBJ(hbm_color.0));
    let _ = DeleteObject(HGDIOBJ(hbm_mask.0));

    hicon
}

/// Khởi chạy ứng dụng với giao diện đồ họa hiện đại chuẩn Mockup
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

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - WIN_WIDTH) / 2;
        let y = (screen_h - WIN_HEIGHT) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Burin Smart Key"),
            WS_POPUP | WS_VISIBLE, // Cửa sổ phẳng hiện đại không viền thô
            x,
            y,
            WIN_WIDTH,
            WIN_HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        ).expect("Không thể tạo cửa sổ chính");

        // Tạo vùng bo góc tròn mượt mà cho toàn bộ cửa sổ (Round Region)
        let rgn = windows::Win32::Graphics::Gdi::CreateRoundRectRgn(0, 0, WIN_WIDTH + 1, WIN_HEIGHT + 1, 24, 24);
        let _ = windows::Win32::Graphics::Gdi::SetWindowRgn(hwnd, Some(rgn), true);

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

        let mut pt = POINT::default();
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

/// Helper kiểm tra tọa độ điểm nằm trong RECT
fn point_in_rect(pt: POINT, r: &RECT) -> bool {
    pt.x >= r.left && pt.x <= r.right && pt.y >= r.top && pt.y <= r.bottom
}

/// Window Procedure xử lý sự kiện giao diện
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            // Cho phép kéo thả di chuyển cửa sổ khi click vào phần Header (trừ nút Close)
            let pt_screen = POINT {
                x: (lparam.0 & 0xFFFF) as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
            };
            let mut pt_client = pt_screen;
            unsafe {
                windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut pt_client);
            }
            if pt_client.y >= 0 && pt_client.y <= 100 && !point_in_rect(pt_client, &RECT_CLOSE) {
                return LRESULT(2); // HTCAPTION -> Kéo cửa sổ mượt mà
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_SETCURSOR => {
            let mut pt = POINT::default();
            unsafe {
                let _ = GetCursorPos(&mut pt);
                windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut pt);
            }
            // Đổi con trỏ thành bàn tay (IDC_HAND) khi hover vào các nút bấm
            if point_in_rect(pt, &RECT_CLOSE)
                || point_in_rect(pt, &RECT_TAB_TELEX)
                || point_in_rect(pt, &RECT_TAB_VNI)
                || point_in_rect(pt, &RECT_BTN_TOGGLE_VE)
                || point_in_rect(pt, &RECT_CHK_STARTUP)
                || point_in_rect(pt, &RECT_BTN_MINIMIZE)
            {
                unsafe {
                    let hand = LoadCursorW(None, IDC_HAND).unwrap_or_default();
                    windows::Win32::UI::WindowsAndMessaging::SetCursor(Some(hand));
                }
                return LRESULT(1);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_LBUTTONDOWN => {
            let pt = POINT {
                x: (lparam.0 & 0xFFFF) as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
            };

            // 1. Nút Close 'X' ở góc trên bên phải
            if point_in_rect(pt, &RECT_CLOSE) {
                unsafe { let _ = ShowWindow(hwnd, SW_HIDE); };
                return LRESULT(0);
            }

            // 2. Tab chọn TELEX
            if point_in_rect(pt, &RECT_TAB_TELEX) {
                set_input_method(InputMethod::Telex);
                unsafe { let _ = InvalidateRect(Some(hwnd), None, true); };
                return LRESULT(0);
            }

            // 3. Tab chọn VNI
            if point_in_rect(pt, &RECT_TAB_VNI) {
                set_input_method(InputMethod::Vni);
                unsafe { let _ = InvalidateRect(Some(hwnd), None, true); };
                return LRESULT(0);
            }

            // 4. Nút đổi trạng thái Tiếng Việt / Tiếng Anh
            if point_in_rect(pt, &RECT_BTN_TOGGLE_VE) {
                let new_v = toggle_viet_mode();
                unsafe {
                    update_tray_icon(hwnd, new_v, false);
                    let _ = InvalidateRect(Some(hwnd), None, true);
                };
                return LRESULT(0);
            }

            // 5. Checkbox Khởi động cùng Windows
            if point_in_rect(pt, &RECT_CHK_STARTUP) {
                let cur = is_autostart_enabled();
                set_autostart(!cur);
                unsafe { let _ = InvalidateRect(Some(hwnd), None, true); };
                return LRESULT(0);
            }

            // 6. Nút "Thu nhỏ xuống khay hệ thống"
            if point_in_rect(pt, &RECT_BTN_MINIMIZE) {
                unsafe { let _ = ShowWindow(hwnd, SW_HIDE); };
                return LRESULT(0);
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
                // ==========================================
                // 1. NỀN CHÍNH CỬA SỔ (Off-White / Light Cream)
                // ==========================================
                let rect_all = RECT { left: 0, top: 0, right: WIN_WIDTH, bottom: WIN_HEIGHT };
                let bg_brush = CreateSolidBrush(COLORREF(0x00F5F6F8)); // #F8F6F5
                FillRect(hdc, &rect_all, bg_brush);
                let _ = DeleteObject(HGDIOBJ(bg_brush.0));

                // ==========================================
                // 2. HEADER GRADIENT CAM HỒNG (Coral / Peach Pink)
                // ==========================================
                // Vẽ gradient từ top xuống 110px
                for y in 0..110 {
                    let ratio = y as f32 / 110.0;
                    let r = (244.0 + (248.0 - 244.0) * ratio) as u32;
                    let g = (112.0 + (126.0 - 112.0) * ratio) as u32;
                    let b = (115.0 + (130.0 - 115.0) * ratio) as u32;
                    let line_brush = CreateSolidBrush(COLORREF(r | (g << 8) | (b << 16)));
                    let line_rect = RECT { left: 0, top: y, right: WIN_WIDTH, bottom: y + 1 };
                    FillRect(hdc, &line_rect, line_brush);
                    let _ = DeleteObject(HGDIOBJ(line_brush.0));
                }

                // Vệt sóng cong nhẹ bên phải header mô phỏng ảnh mẫu
                let wave_brush = CreateSolidBrush(COLORREF(0x008ED2FB)); // Coral cam sáng
                let pen_null = CreatePen(PS_SOLID, 0, COLORREF(0x008ED2FB));
                let old_pen = SelectObject(hdc, HGDIOBJ(pen_null.0));
                let _ = SelectObject(hdc, HGDIOBJ(wave_brush.0));
                let _ = Ellipse(hdc, WIN_WIDTH - 160, -40, WIN_WIDTH + 80, 150);
                SelectObject(hdc, old_pen);
                let _ = DeleteObject(HGDIOBJ(pen_null.0));
                let _ = DeleteObject(HGDIOBJ(wave_brush.0));

                // Nút Đóng '✕' ở góc phải trên
                SetBkMode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00333333));
                let font_close = CreateFontW(
                    20, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                let old_font = SelectObject(hdc, HGDIOBJ(font_close.0));
                let mut close_rect = RECT_CLOSE;
                let mut close_w = to_wide_null("✕");
                DrawTextW(hdc, &mut close_w, &mut close_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                let _ = DeleteObject(HGDIOBJ(font_close.0));

                // Icon Chú Heo Tròn trong hình tròn trắng ở Header (To hơn và rõ nét)
                let circle_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                let circle_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FEE2E2));
                let _ = SelectObject(hdc, HGDIOBJ(circle_pen.0));
                let _ = SelectObject(hdc, HGDIOBJ(circle_brush.0));
                let _ = Ellipse(hdc, 28, 18, 86, 76);
                let _ = DeleteObject(HGDIOBJ(circle_pen.0));
                let _ = DeleteObject(HGDIOBJ(circle_brush.0));

                // Vẽ chú heo siêu dễ thương ở giữa hình tròn
                draw_piggy_face(hdc, 57, 47, 22, true);

                // Tiêu đề App: BURIN SMART KEY (To, đậm, sắc nét)
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                let font_header_title = CreateFontW(
                    24, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_header_title.0));
                let mut htitle_rect = RECT { left: 98, top: 22, right: 440, bottom: 50 };
                let mut htitle_w = to_wide_null("BURIN SMART KEY");
                DrawTextW(hdc, &mut htitle_w, &mut htitle_rect, DT_LEFT | DT_SINGLELINE);

                // Phụ đề: Bộ gõ tiếng Việt thông minh thế hệ mới (Nâng font to rõ 16px)
                let font_header_sub = CreateFontW(
                    16, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_header_sub.0));
                SetTextColor(hdc, COLORREF(0x00FFF5F5));
                let mut hsub_rect = RECT { left: 98, top: 52, right: 450, bottom: 78 };
                let mut hsub_w = to_wide_null("Bộ gõ tiếng Việt thông minh thế hệ mới");
                DrawTextW(hdc, &mut hsub_w, &mut hsub_rect, DT_LEFT | DT_SINGLELINE);

                // ==========================================
                // 3. THÂN GIAO DIỆN: KIỂU GÕ & CHẾ ĐỘ
                // ==========================================
                let font_section = CreateFontW(
                    16, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_section.0));
                SetTextColor(hdc, COLORREF(0x00332822)); // Màu xám đen đậm
                let mut sec_rect = RECT { left: 32, top: 120, right: 280, bottom: 142 };
                let mut sec_w = to_wide_null("KIỂU GÕ & CHẾ ĐỘ");
                DrawTextW(hdc, &mut sec_w, &mut sec_rect, DT_LEFT | DT_SINGLELINE);

                // --- VÙNG PILL CHỌN TELEX / VNI ---
                let cur_im = get_current_method();
                let is_v = is_viet_mode();

                // Nền bo tròn ngoài của Pill (xám nhạt)
                let pill_bg_brush = CreateSolidBrush(COLORREF(0x00E2DFDE)); // #DEDFE2
                let pen_transparent = CreatePen(PS_SOLID, 1, COLORREF(0x00E2DFDE));
                let _ = SelectObject(hdc, HGDIOBJ(pen_transparent.0));
                let _ = SelectObject(hdc, HGDIOBJ(pill_bg_brush.0));
                let _ = RoundRect(hdc, 30, 142, 254, 186, 40, 40);
                let _ = DeleteObject(HGDIOBJ(pen_transparent.0));
                let _ = DeleteObject(HGDIOBJ(pill_bg_brush.0));

                // Nền trắng nổi của Tab được chọn
                let active_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                let active_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00F0F0F0));
                let _ = SelectObject(hdc, HGDIOBJ(active_pen.0));
                let _ = SelectObject(hdc, HGDIOBJ(active_brush.0));

                if cur_im == InputMethod::Telex {
                    let _ = RoundRect(hdc, 32, 144, 142, 184, 36, 36);
                } else {
                    let _ = RoundRect(hdc, 142, 144, 252, 184, 36, 36);
                }
                let _ = DeleteObject(HGDIOBJ(active_pen.0));
                let _ = DeleteObject(HGDIOBJ(active_brush.0));

                // Text TELEX & VNI (Nâng font lên 16px to đậm)
                let font_tab_bold = CreateFontW(
                    16, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                let font_tab_normal = CreateFontW(
                    16, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );

                if cur_im == InputMethod::Telex {
                    SelectObject(hdc, HGDIOBJ(font_tab_bold.0));
                    SetTextColor(hdc, COLORREF(0x00111111));
                    let mut telex_rect = RECT_TAB_TELEX;
                    let mut telex_w = to_wide_null("TELEX");
                    DrawTextW(hdc, &mut telex_w, &mut telex_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                    SelectObject(hdc, HGDIOBJ(font_tab_normal.0));
                    SetTextColor(hdc, COLORREF(0x00666666));
                    let mut vni_rect = RECT_TAB_VNI;
                    let mut vni_w = to_wide_null("VNI");
                    DrawTextW(hdc, &mut vni_w, &mut vni_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                } else {
                    SelectObject(hdc, HGDIOBJ(font_tab_normal.0));
                    SetTextColor(hdc, COLORREF(0x00666666));
                    let mut telex_rect = RECT_TAB_TELEX;
                    let mut telex_w = to_wide_null("TELEX");
                    DrawTextW(hdc, &mut telex_w, &mut telex_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                    SelectObject(hdc, HGDIOBJ(font_tab_bold.0));
                    SetTextColor(hdc, COLORREF(0x00111111));
                    let mut vni_rect = RECT_TAB_VNI;
                    let mut vni_w = to_wide_null("VNI");
                    DrawTextW(hdc, &mut vni_w, &mut vni_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                }

                // Dòng chữ: Trạng thái: ĐANG BẬT [Tiếng Việt] (Nâng font 15px)
                let font_status = CreateFontW(
                    15, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_status.0));
                SetTextColor(hdc, COLORREF(0x001F1F1F));
                let status_mode_text = if is_v { "Trạng thái: ĐANG BẬT [Tiếng Việt]" } else { "Trạng thái: ĐANG TẮT [Tiếng Anh]" };
                let mut smode_rect = RECT { left: 32, top: 194, right: 300, bottom: 216 };
                let mut smode_w = to_wide_null(status_mode_text);
                DrawTextW(hdc, &mut smode_w, &mut smode_rect, DT_LEFT | DT_SINGLELINE);

                // --- NÚT BẬT / TẮT TIẾNG VIỆT (BÊN PHẢI) ---
                if is_v {
                    // Nút ĐỎ CAM BẬT: [🐷 TIẾNG VIỆT (V)]
                    let btn_coral = CreateSolidBrush(COLORREF(0x00696EEF)); // #EF6E69 (BGR)
                    let pen_coral = CreatePen(PS_SOLID, 1, COLORREF(0x00696EEF));
                    let _ = SelectObject(hdc, HGDIOBJ(pen_coral.0));
                    let _ = SelectObject(hdc, HGDIOBJ(btn_coral.0));
                    let _ = RoundRect(hdc, RECT_BTN_TOGGLE_VE.left, RECT_BTN_TOGGLE_VE.top, RECT_BTN_TOGGLE_VE.right, RECT_BTN_TOGGLE_VE.bottom, 42, 42);
                    let _ = DeleteObject(HGDIOBJ(pen_coral.0));
                    let _ = DeleteObject(HGDIOBJ(btn_coral.0));

                    // Vẽ chú heo nhỏ bên trong nút
                    draw_piggy_face(hdc, RECT_BTN_TOGGLE_VE.left + 24, RECT_BTN_TOGGLE_VE.top + 22, 13, true);

                    // Chữ TIẾNG VIỆT (V)
                    SetTextColor(hdc, COLORREF(0x00FFFFFF));
                    SelectObject(hdc, HGDIOBJ(font_tab_bold.0));
                    let mut btext_rect = RECT { left: RECT_BTN_TOGGLE_VE.left + 38, top: RECT_BTN_TOGGLE_VE.top, right: RECT_BTN_TOGGLE_VE.right - 8, bottom: RECT_BTN_TOGGLE_VE.bottom };
                    let mut btext_w = to_wide_null("TIẾNG VIỆT (V)");
                    DrawTextW(hdc, &mut btext_w, &mut btext_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                    // Phía dưới có dòng TIẾNG ANH (E) mờ nhỏ
                    SetTextColor(hdc, COLORREF(0x00666666));
                    SelectObject(hdc, HGDIOBJ(font_status.0));
                    draw_piggy_face(hdc, RECT_BTN_TOGGLE_VE.left + 45, 202, 8, false);
                    let mut sub_e_rect = RECT { left: RECT_BTN_TOGGLE_VE.left + 58, top: 194, right: RECT_BTN_TOGGLE_VE.right, bottom: 216 };
                    let mut sub_e_w = to_wide_null("TIẾNG ANH (E)");
                    DrawTextW(hdc, &mut sub_e_w, &mut sub_e_rect, DT_LEFT | DT_SINGLELINE);
                } else {
                    // Nút XÁM: [🐷 TIẾNG ANH (E)]
                    let btn_gray = CreateSolidBrush(COLORREF(0x00A0A0A0));
                    let pen_gray = CreatePen(PS_SOLID, 1, COLORREF(0x00A0A0A0));
                    let _ = SelectObject(hdc, HGDIOBJ(pen_gray.0));
                    let _ = SelectObject(hdc, HGDIOBJ(btn_gray.0));
                    let _ = RoundRect(hdc, RECT_BTN_TOGGLE_VE.left, RECT_BTN_TOGGLE_VE.top, RECT_BTN_TOGGLE_VE.right, RECT_BTN_TOGGLE_VE.bottom, 42, 42);
                    let _ = DeleteObject(HGDIOBJ(pen_gray.0));
                    let _ = DeleteObject(HGDIOBJ(btn_gray.0));

                    draw_piggy_face(hdc, RECT_BTN_TOGGLE_VE.left + 24, RECT_BTN_TOGGLE_VE.top + 22, 13, false);

                    SetTextColor(hdc, COLORREF(0x00FFFFFF));
                    SelectObject(hdc, HGDIOBJ(font_tab_bold.0));
                    let mut btext_rect = RECT { left: RECT_BTN_TOGGLE_VE.left + 38, top: RECT_BTN_TOGGLE_VE.top, right: RECT_BTN_TOGGLE_VE.right - 8, bottom: RECT_BTN_TOGGLE_VE.bottom };
                    let mut btext_w = to_wide_null("TIẾNG ANH (E)");
                    DrawTextW(hdc, &mut btext_w, &mut btext_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                    SetTextColor(hdc, COLORREF(0x00666666));
                    SelectObject(hdc, HGDIOBJ(font_status.0));
                    draw_piggy_face(hdc, RECT_BTN_TOGGLE_VE.left + 45, 202, 8, true);
                    let mut sub_e_rect = RECT { left: RECT_BTN_TOGGLE_VE.left + 58, top: 194, right: RECT_BTN_TOGGLE_VE.right, bottom: 216 };
                    let mut sub_e_w = to_wide_null("TIẾNG VIỆT (V)");
                    DrawTextW(hdc, &mut sub_e_w, &mut sub_e_rect, DT_LEFT | DT_SINGLELINE);
                }

                // ==========================================
                // 4. KHUNG TRẮNG CARD "PHÍM TẮT NHANH" (Mở rộng thoải mái)
                // ==========================================
                let card_bg = CreateSolidBrush(COLORREF(0x00FFFFFF));
                let card_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00EFE8E5));
                let _ = SelectObject(hdc, HGDIOBJ(card_pen.0));
                let _ = SelectObject(hdc, HGDIOBJ(card_bg.0));
                let _ = RoundRect(hdc, 30, 226, 468, 418, 22, 22);
                let _ = DeleteObject(HGDIOBJ(card_pen.0));
                let _ = DeleteObject(HGDIOBJ(card_bg.0));

                // Tiêu đề card: ✦ Phím tắt nhanh (Nâng font 17px)
                let font_card_title = CreateFontW(
                    17, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_card_title.0));
                SetTextColor(hdc, COLORREF(0x00111111));
                let mut ctitle_rect = RECT { left: 48, top: 240, right: 320, bottom: 266 };
                let mut ctitle_w = to_wide_null("✦ Phím tắt nhanh");
                DrawTextW(hdc, &mut ctitle_w, &mut ctitle_rect, DT_LEFT | DT_SINGLELINE);

                // --- HÀNG 1: [ Ctrl ] + [ Shift ] : Bật / Tắt bộ gõ ---
                let draw_key_cap = |left: i32, top: i32, right: i32, bottom: i32, text: &str| {
                    let key_bg = CreateSolidBrush(COLORREF(0x00FAFAFA));
                    let key_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00CBCBCB));
                    let _ = SelectObject(hdc, HGDIOBJ(key_pen.0));
                    let _ = SelectObject(hdc, HGDIOBJ(key_bg.0));
                    let _ = RoundRect(hdc, left, top, right, bottom, 10, 10);
                    let _ = DeleteObject(HGDIOBJ(key_pen.0));
                    let _ = DeleteObject(HGDIOBJ(key_bg.0));

                    SetTextColor(hdc, COLORREF(0x00111111));
                    let mut krect = RECT { left, top, right, bottom: bottom - 1 };
                    let mut kw = to_wide_null(text);
                    DrawTextW(hdc, &mut kw, &mut krect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                };

                // Font phím bấm to 16px đậm
                let font_key = CreateFontW(
                    16, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_key.0));

                // Phím Ctrl
                draw_key_cap(48, 276, 104, 314, "Ctrl");

                // Dấu '+'
                let mut plus_rect = RECT { left: 108, top: 276, right: 126, bottom: 314 };
                let mut plus_w = to_wide_null("+");
                DrawTextW(hdc, &mut plus_w, &mut plus_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                // Phím Shift
                draw_key_cap(130, 276, 192, 314, "Shift");

                // Nhãn: : Bật / Tắt bộ gõ (Nâng font 16px to rõ ràng)
                let font_body = CreateFontW(
                    16, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_body.0));
                SetTextColor(hdc, COLORREF(0x001F1F1F));
                let mut desc1_rect = RECT { left: 204, top: 276, right: 450, bottom: 314 };
                let mut desc1_w = to_wide_null(": Bật / Tắt bộ gõ");
                DrawTextW(hdc, &mut desc1_w, &mut desc1_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

                // --- HÀNG 2: [ F9 ] : Chuyển Telex <-> VNI ---
                SelectObject(hdc, HGDIOBJ(font_key.0));
                draw_key_cap(48, 324, 104, 362, "F9");

                SelectObject(hdc, HGDIOBJ(font_body.0));
                let mut desc2_rect = RECT { left: 190, top: 324, right: 450, bottom: 362 };
                let mut desc2_w = to_wide_null(": Chuyển Telex <-> VNI");
                DrawTextW(hdc, &mut desc2_w, &mut desc2_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

                // Dòng ghi chú dưới cùng của Card (Nâng font 15px cực kỳ dễ đọc)
                let font_tip = CreateFontW(
                    15, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_tip.0));
                SetTextColor(hdc, COLORREF(0x004A4A4A));
                let mut tip_rect = RECT { left: 48, top: 376, right: 460, bottom: 404 };
                let mut tip_w = to_wide_null("Nhấp chuột trái vào icon khay hệ thống để bật/tắt nhanh");
                DrawTextW(hdc, &mut tip_w, &mut tip_rect, DT_LEFT | DT_SINGLELINE);

                // ==========================================
                // 5. CHECKBOX "KHỞI ĐỘNG CÙNG WINDOWS"
                // ==========================================
                let is_auto = is_autostart_enabled();
                // Vẽ ô vuông checkbox bo tròn lớn hơn
                let chk_box_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                let chk_box_pen = CreatePen(PS_SOLID, 1, COLORREF(0x008E8E8E));
                let _ = SelectObject(hdc, HGDIOBJ(chk_box_pen.0));
                let _ = SelectObject(hdc, HGDIOBJ(chk_box_brush.0));
                let _ = RoundRect(hdc, 32, 442, 52, 462, 8, 8);
                let _ = DeleteObject(HGDIOBJ(chk_box_pen.0));
                let _ = DeleteObject(HGDIOBJ(chk_box_brush.0));

                if is_auto {
                    // Dấu tích check màu đỏ cam
                    let font_check = CreateFontW(15, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0), FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI"));
                    SelectObject(hdc, HGDIOBJ(font_check.0));
                    SetTextColor(hdc, COLORREF(0x00696EEF));
                    let mut check_mark_rect = RECT { left: 32, top: 442, right: 52, bottom: 462 };
                    let mut check_mark_w = to_wide_null("✔");
                    DrawTextW(hdc, &mut check_mark_w, &mut check_mark_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                    let _ = DeleteObject(HGDIOBJ(font_check.0));
                }

                // Nhãn "Khởi động cùng Windows" (Nâng font 16px to rõ nét)
                SelectObject(hdc, HGDIOBJ(font_body.0));
                SetTextColor(hdc, COLORREF(0x001F1F1F));
                let mut chk_label_rect = RECT { left: 60, top: 438, right: 400, bottom: 466 };
                let mut chk_label_w = to_wide_null("Khởi động cùng Windows");
                DrawTextW(hdc, &mut chk_label_w, &mut chk_label_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

                // ==========================================
                // 6. NÚT "THU NHỎ XUỐNG KHAY HỆ THỐNG"
                // ==========================================
                let btn_min_bg = CreateSolidBrush(COLORREF(0x00DAE4FA)); // #FAE4DA (hồng kem nhạt)
                let btn_min_pen = CreatePen(PS_SOLID, 1, COLORREF(0x008E98F4)); // viền hồng san hô
                let _ = SelectObject(hdc, HGDIOBJ(btn_min_pen.0));
                let _ = SelectObject(hdc, HGDIOBJ(btn_min_bg.0));
                let _ = RoundRect(hdc, RECT_BTN_MINIMIZE.left, RECT_BTN_MINIMIZE.top, RECT_BTN_MINIMIZE.right, RECT_BTN_MINIMIZE.bottom, 20, 20);
                let _ = DeleteObject(HGDIOBJ(btn_min_pen.0));
                let _ = DeleteObject(HGDIOBJ(btn_min_bg.0));

                let font_btn_min = CreateFontW(
                    17, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_btn_min.0));
                SetTextColor(hdc, COLORREF(0x001A1A1A));
                let mut bmin_rect = RECT_BTN_MINIMIZE;
                let mut bmin_w = to_wide_null("-- Thu nhỏ xuống khay hệ thống");
                DrawTextW(hdc, &mut bmin_w, &mut bmin_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                let _ = DeleteObject(HGDIOBJ(font_btn_min.0));

                // Dọn dẹp fonts
                SelectObject(hdc, old_font);
                let _ = DeleteObject(HGDIOBJ(font_header_title.0));
                let _ = DeleteObject(HGDIOBJ(font_header_sub.0));
                let _ = DeleteObject(HGDIOBJ(font_section.0));
                let _ = DeleteObject(HGDIOBJ(font_tab_bold.0));
                let _ = DeleteObject(HGDIOBJ(font_tab_normal.0));
                let _ = DeleteObject(HGDIOBJ(font_status.0));
                let _ = DeleteObject(HGDIOBJ(font_card_title.0));
                let _ = DeleteObject(HGDIOBJ(font_key.0));
                let _ = DeleteObject(HGDIOBJ(font_body.0));
                let _ = DeleteObject(HGDIOBJ(font_tip.0));

                let _ = windows::Win32::Graphics::Gdi::EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = wparam.0 as usize & 0xFFFF;
            unsafe {
                match id {
                    ID_TRAY_TELEX => {
                        set_input_method(InputMethod::Telex);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_TRAY_VNI => {
                        set_input_method(InputMethod::Vni);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_TRAY_TOGGLE_VE => {
                        let new_v = toggle_viet_mode();
                        update_tray_icon(hwnd, new_v, false);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    ID_TRAY_AUTORUN => {
                        let cur = is_autostart_enabled();
                        set_autostart(!cur);
                        let _ = InvalidateRect(Some(hwnd), None, true);
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
