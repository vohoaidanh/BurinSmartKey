#![windows_subsystem = "windows"]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, GetStockObject,
    SelectObject, SetBkMode, SetTextColor,
    DT_LEFT, DT_SINGLELINE, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, FW_NORMAL, HBRUSH, HGDIOBJ,
    WHITE_BRUSH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_WRITE, REG_DWORD, REG_SZ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW,
    ShowWindow, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, CS_HREDRAW, CS_VREDRAW,
    HMENU, IDC_ARROW, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WINDOW_EX_STYLE,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_PAINT,
    WNDCLASSEXW, WS_CAPTION, WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};

// Nhúng trực tiếp file BurinSmartKey.exe vào file cài đặt!
const PAYLOAD_BYTES: &[u8] = include_bytes!("../../installer/BurinSmartKey.exe");

const ID_BTN_INSTALL: usize = 101;
const ID_BTN_CANCEL: usize = 102;
const ID_CHK_DESKTOP: usize = 103;
const ID_CHK_STARTUP: usize = 104;

const BST_CHECKED_VAL: isize = 1;

static mut HWND_CHK_DESKTOP: Option<HWND> = None;
static mut HWND_CHK_STARTUP: Option<HWND> = None;

fn get_install_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\".to_string());
    Path::new(&local_app_data).join("BurinSmartKey")
}

fn create_shortcut(target_exe: &Path, shortcut_path: &Path, description: &str, working_dir: &Path) {
    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.WorkingDirectory = '{}'; $s.Description = '{}'; $s.Save()",
        shortcut_path.to_string_lossy(),
        target_exe.to_string_lossy(),
        working_dir.to_string_lossy(),
        description
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

fn perform_installation(create_desktop: bool, enable_startup: bool) -> Result<(), String> {
    // 1. Dừng các tiến trình cũ nếu đang chạy
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "BurinSmartKey.exe", "/T"])
        .output();
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "SmartVietnameseIME.exe", "/T"])
        .output();
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "core_engine.exe", "/T"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(400));

    // 2. Tạo thư mục cài đặt
    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir).map_err(|e| format!("Không thể tạo thư mục cài đặt: {}", e))?;

    // 3. Trích xuất file BurinSmartKey.exe
    let target_exe = install_dir.join("BurinSmartKey.exe");
    fs::write(&target_exe, PAYLOAD_BYTES).map_err(|e| format!("Không thể ghi file BurinSmartKey.exe: {}", e))?;

    // 4. Tạo file Uninstall.bat
    let uninstaller_bat = install_dir.join("Uninstall.bat");
    let uninstaller_content = "@echo off\r\nchcp 65001 >nul\r\necho Đang gỡ bỏ Burin Smart Key...\r\ntaskkill /F /IM BurinSmartKey.exe /T 2>nul\r\ntimeout /t 1 /nobreak >nul\r\nreg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\" /v \"BurinSmartKey\" /f 2>nul\r\nreg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\BurinSmartKey\" /f 2>nul\r\ndel /f /q \"%USERPROFILE%\\Desktop\\Burin Smart Key.lnk\" 2>nul\r\ndel /f /q \"%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\Burin Smart Key.lnk\" 2>nul\r\necho Gỡ bỏ thành công!\r\npause\r\n";
    let _ = fs::write(&uninstaller_bat, uninstaller_content);

    // 5. Đăng ký thông tin vào Windows Control Panel / Settings (Uninstall)
    unsafe {
        let subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\BurinSmartKey");
        let mut hkey = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            subkey,
            Some(0),
            None,
            windows::Win32::System::Registry::REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        ).is_ok() {
            let set_str = |name: PCWSTR, val: &str| {
                let wide: Vec<u16> = val.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
                let _ = RegSetValueExW(hkey, name, Some(0), REG_SZ, Some(bytes));
            };

            set_str(w!("DisplayName"), "Burin Smart Key");
            set_str(w!("DisplayVersion"), "1.0.0");
            set_str(w!("Publisher"), "Burin Software");
            set_str(w!("DisplayIcon"), &format!("{},0", target_exe.to_string_lossy()));
            set_str(w!("UninstallString"), &format!("cmd.exe /c \"{}\"", uninstaller_bat.to_string_lossy()));
            set_str(w!("InstallLocation"), &install_dir.to_string_lossy());

            let dword_val = 1u32.to_ne_bytes();
            let _ = RegSetValueExW(hkey, w!("NoModify"), Some(0), REG_DWORD, Some(&dword_val));
            let _ = RegSetValueExW(hkey, w!("NoRepair"), Some(0), REG_DWORD, Some(&dword_val));

            let _ = RegCloseKey(hkey);
        }
    }

    // 6. Tạo Desktop & Start Menu Shortcut
    let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\".to_string());

    if create_desktop {
        let desktop_lnk = Path::new(&user_profile).join("Desktop").join("Burin Smart Key.lnk");
        create_shortcut(&target_exe, &desktop_lnk, "Burin Smart Key - Bộ gõ Tiếng Việt", &install_dir);
    }

    let start_menu_lnk = Path::new(&app_data)
        .join("Microsoft\\Windows\\Start Menu\\Programs")
        .join("Burin Smart Key.lnk");
    create_shortcut(&target_exe, &start_menu_lnk, "Burin Smart Key - Bộ gõ Tiếng Việt", &install_dir);

    // 7. Khởi động cùng Windows
    unsafe {
        let run_subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let mut hkey_run = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            run_subkey,
            Some(0),
            None,
            windows::Win32::System::Registry::REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey_run,
            None,
        ).is_ok() {
            let val_name = w!("BurinSmartKey");
            if enable_startup {
                let cmd_val = format!("\"{}\"", target_exe.to_string_lossy());
                let wide: Vec<u16> = cmd_val.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
                let _ = RegSetValueExW(hkey_run, val_name, Some(0), REG_SZ, Some(bytes));
            } else {
                let _ = windows::Win32::System::Registry::RegDeleteValueW(hkey_run, val_name);
            }
            let _ = RegCloseKey(hkey_run);
        }
    }

    // 8. Khởi chạy ứng dụng ngay sau khi cài
    let _ = Command::new(&target_exe).current_dir(&install_dir).spawn();

    Ok(())
}

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
                // Checkbox Tạo Desktop Shortcut
                let chk_desktop = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Tạo biểu tượng ngoài màn hình chính (Desktop Shortcut)"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(BS_AUTOCHECKBOX as _),
                    30,
                    220,
                    390,
                    24,
                    Some(hwnd),
                    Some(HMENU(ID_CHK_DESKTOP as _)),
                    Some(instance.into()),
                    None,
                ).unwrap();
                let _ = SendMessageW(chk_desktop, BM_SETCHECK, Some(WPARAM(BST_CHECKED_VAL as _)), Some(LPARAM(0)));
                HWND_CHK_DESKTOP = Some(chk_desktop);

                // Checkbox Khởi động cùng Windows
                let chk_startup = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Tự động khởi động cùng Windows khi bật máy"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(BS_AUTOCHECKBOX as _),
                    30,
                    248,
                    390,
                    24,
                    Some(hwnd),
                    Some(HMENU(ID_CHK_STARTUP as _)),
                    Some(instance.into()),
                    None,
                ).unwrap();
                let _ = SendMessageW(chk_startup, BM_SETCHECK, Some(WPARAM(BST_CHECKED_VAL as _)), Some(LPARAM(0)));
                HWND_CHK_STARTUP = Some(chk_startup);

                // Nút "Cài đặt ngay"
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Cài đặt ngay"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    200,
                    290,
                    120,
                    38,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_INSTALL as _)),
                    Some(instance.into()),
                    None,
                );

                // Nút "Hủy"
                let _ = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("Hủy"),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                    330,
                    290,
                    80,
                    38,
                    Some(hwnd),
                    Some(HMENU(ID_BTN_CANCEL as _)),
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
                // Header hồng sang trọng
                let header_rect = RECT { left: 0, top: 0, right: 450, bottom: 70 };
                let header_brush = CreateSolidBrush(COLORREF(0x006548F4));
                FillRect(hdc, &header_rect, header_brush);
                let _ = DeleteObject(HGDIOBJ(header_brush.0));

                SetBkMode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00FFFFFF));

                let font_title = CreateFontW(
                    20, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                let old_font = SelectObject(hdc, HGDIOBJ(font_title.0));
                let mut title_rect = RECT { left: 24, top: 12, right: 420, bottom: 38 };
                let mut title_w = to_wide_null("BURIN SMART KEY SETUP");
                DrawTextW(hdc, &mut title_w, &mut title_rect, DT_LEFT | DT_SINGLELINE);

                let font_sub = CreateFontW(
                    13, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                    FONT_CHARSET(1), FONT_OUTPUT_PRECISION(0),
                    FONT_CLIP_PRECISION(0), FONT_QUALITY(0), 0, w!("Segoe UI")
                );
                SelectObject(hdc, HGDIOBJ(font_sub.0));
                SetTextColor(hdc, COLORREF(0x00FEE2E2));
                let mut sub_rect = RECT { left: 24, top: 40, right: 420, bottom: 60 };
                let mut sub_w = to_wide_null("Trình cài đặt phần mềm gõ tiếng Việt thông minh");
                DrawTextW(hdc, &mut sub_w, &mut sub_rect, DT_LEFT | DT_SINGLELINE);

                // Khung giới thiệu tính năng
                SelectObject(hdc, HGDIOBJ(font_title.0));
                SetTextColor(hdc, COLORREF(0x001E293B));
                let mut lbl_rect = RECT { left: 30, top: 82, right: 420, bottom: 102 };
                let mut lbl_w = to_wide_null("TÍNH NĂNG NỔI BẬT:");
                DrawTextW(hdc, &mut lbl_w, &mut lbl_rect, DT_LEFT | DT_SINGLELINE);

                let card_rect = RECT { left: 30, top: 108, right: 410, bottom: 206 };
                let card_brush = CreateSolidBrush(COLORREF(0x00FFF5F5));
                FillRect(hdc, &card_rect, card_brush);
                let _ = DeleteObject(HGDIOBJ(card_brush.0));

                SelectObject(hdc, HGDIOBJ(font_sub.0));
                SetTextColor(hdc, COLORREF(0x000F766E));

                let mut f1_rect = RECT { left: 42, top: 114, right: 400, bottom: 134 };
                let mut f1_w = to_wide_null("✔ Phím tắt: Ctrl + Shift (Bật/Tắt) | F9 (Đổi kiểu gõ)");
                DrawTextW(hdc, &mut f1_w, &mut f1_rect, DT_LEFT | DT_SINGLELINE);

                let mut f2_rect = RECT { left: 42, top: 136, right: 400, bottom: 156 };
                let mut f2_w = to_wide_null("✔ Khôi phục từ tiếng Anh thông minh (worker, password, if...)");
                DrawTextW(hdc, &mut f2_w, &mut f2_rect, DT_LEFT | DT_SINGLELINE);

                let mut f3_rect = RECT { left: 42, top: 158, right: 400, bottom: 178 };
                let mut f3_w = to_wide_null("✔ Icon chú heo Burin đổi màu trực quan (Hồng = Bật, Xám = Tắt)");
                DrawTextW(hdc, &mut f3_w, &mut f3_rect, DT_LEFT | DT_SINGLELINE);

                let mut f4_rect = RECT { left: 42, top: 180, right: 400, bottom: 200 };
                let mut f4_w = to_wide_null("✔ Tích hợp gỡ cài đặt (Uninstall) trong Control Panel / Settings");
                DrawTextW(hdc, &mut f4_w, &mut f4_rect, DT_LEFT | DT_SINGLELINE);

                // Dọn dẹp
                SelectObject(hdc, old_font);
                let _ = DeleteObject(HGDIOBJ(font_title.0));
                let _ = DeleteObject(HGDIOBJ(font_sub.0));
                let _ = windows::Win32::Graphics::Gdi::EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = wparam.0 as usize & 0xFFFF;
            if id == ID_BTN_INSTALL {
                let chk_desktop = unsafe { HWND_CHK_DESKTOP.unwrap() };
                let chk_startup = unsafe { HWND_CHK_STARTUP.unwrap() };

                let desktop_checked = unsafe { SendMessageW(chk_desktop, BM_GETCHECK, None, None).0 == BST_CHECKED_VAL };
                let startup_checked = unsafe { SendMessageW(chk_startup, BM_GETCHECK, None, None).0 == BST_CHECKED_VAL };

                match perform_installation(desktop_checked, startup_checked) {
                    Ok(_) => {
                        unsafe {
                            let msg = w!("Cài đặt Burin Smart Key thành công!\nỨng dụng đã được khởi chạy với icon chú heo tại khay hệ thống.\n\nPhím tắt tiện lợi:\n• Ctrl + Shift : Bật / Tắt bộ gõ\n• F9 : Chuyển đổi TELEX <-> VNI");
                            let title = w!("Hoàn tất cài đặt");
                            MessageBoxW(Some(hwnd), msg, title, MB_OK | MB_ICONINFORMATION);
                            let _ = DestroyWindow(hwnd);
                        }
                    }
                    Err(e) => {
                        unsafe {
                            let wide_err: Vec<u16> = e.encode_utf16().chain(std::iter::once(0)).collect();
                            MessageBoxW(Some(hwnd), PCWSTR(wide_err.as_ptr()), w!("Lỗi cài đặt"), MB_OK | MB_ICONERROR);
                        }
                    }
                }
            } else if id == ID_BTN_CANCEL {
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            unsafe {
                let _ = DestroyWindow(hwnd);
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

fn main() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let class_name = w!("BurinSmartKey_Setup_UI");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
        };

        let _ = RegisterClassExW(&wc);

        let width = 450;
        let height = 380;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Cài đặt Burin Smart Key"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            None,
        ).expect("Không thể tạo cửa sổ cài đặt");

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            DispatchMessageW(&msg);
        }
    }
}
