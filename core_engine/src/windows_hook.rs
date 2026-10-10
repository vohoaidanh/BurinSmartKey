//! Module bàn phím hệ thống Windows tích hợp:
//! 1. Hỗ trợ đầy đủ cả 2 kiểu gõ phổ biến nhất: TELEX và VNI
//!    - Phím tắt chuyển đổi nhanh: Ctrl + Shift hoặc F9 (hoặc chọn trong menu khởi động)
//! 2. Tự động phục hồi nguyên bản phím gõ (Full Backtracking Restoration):
//!    Khi kết thúc từ (Space, Enter...): Nếu từ hiển thị không có nghĩa tiếng Việt -> Phục hồi tiếng Anh.
//! 3. BỎ QUA HOÀN TOÀN TỔ HỢP PHÍM (Ctrl, Alt, Win)
//! 4. XÓA BỘ ĐỆM KHI ĐẦU DÒNG / ĐẦU VĂN BẢN / DI CHUYỂN CON TRỎ
//! 5. KHÔI PHỤC NGỮ CẢNH ĐA TỪ (Multi-Word Context Restoration):
//!    Xóa lùi qua dấu cách về từ trước đó, gõ tiếp ký tự dấu Telex/VNI thì vẫn ăn vào từ trước!

use std::sync::Mutex;
use vnkey_engine::{Engine, InputMethod, Options};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetAsyncKeyState, SendInput, ToUnicode, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
    VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT,
    VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB,
    VK_UP, VK_LWIN, VK_RWIN, VK_F9,
    VK_LCONTROL, VK_RCONTROL, VK_LSHIFT, VK_RSHIFT,
};
use crate::phonology::is_valid_vietnamese_syllable;
use crate::dictionary::is_english_word;

const VNKEY_INJECTED_TAG: usize = 0x564E4B59; // "VNKY"

struct SafeHHOOK(HHOOK);
unsafe impl Send for SafeHHOOK {}
unsafe impl Sync for SafeHHOOK {}

/// Thông tin một từ trong lịch sử
#[derive(Clone, Debug)]
struct WordEntry {
    raw_typed: String,
    committed_text: String,
}

/// Bộ theo dõi chuỗi ký tự đang gõ và lịch sử từ
struct WordTracker {
    current_raw: String,
    current_committed: String,
    history_words: Vec<WordEntry>,
    after_space: bool,
}

static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);
static CURRENT_METHOD: Mutex<InputMethod> = Mutex::new(InputMethod::Telex);
static TRACKER: Mutex<WordTracker> = Mutex::new(WordTracker {
    current_raw: String::new(),
    current_committed: String::new(),
    history_words: Vec::new(),
    after_space: false,
});

static KEYBOARD_HOOK: Mutex<Option<SafeHHOOK>> = Mutex::new(None);
static MOUSE_HOOK: Mutex<Option<SafeHHOOK>> = Mutex::new(None);
static IS_INJECTING: Mutex<bool> = Mutex::new(false);
static CTRL_SHIFT_PENDING: Mutex<bool> = Mutex::new(false);

struct SafeHWND(HWND);
unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

static VIET_MODE: Mutex<bool> = Mutex::new(true);
static UI_HWND: Mutex<Option<SafeHWND>> = Mutex::new(None);

/// Đăng ký HWND của cửa sổ chính để gửi thông điệp khi phím tắt thay đổi
pub fn set_ui_hwnd(hwnd: HWND) {
    if let Ok(mut lock) = UI_HWND.lock() {
        *lock = Some(SafeHWND(hwnd));
    }
}

/// Thông báo cửa sổ UI cập nhật icon tray và redraw
fn notify_ui_state_changed() {
    if let Ok(lock) = UI_HWND.lock() {
        if let Some(ref safe_hwnd) = *lock {
            // Gửi message thông báo UI cập nhật (WM_USER + 102)
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                    Some(safe_hwnd.0),
                    windows::Win32::UI::WindowsAndMessaging::WM_USER + 102,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }
}

/// Kiểm tra đang ở chế độ Tiếng Việt (V) hay Tiếng Anh (E)
pub fn is_viet_mode() -> bool {
    if let Ok(vm) = VIET_MODE.lock() {
        *vm
    } else {
        true
    }
}

/// Bật hoặc tắt chế độ gõ tiếng Việt
pub fn set_viet_mode(enabled: bool) {
    if let Ok(mut vm) = VIET_MODE.lock() {
        *vm = enabled;
    }
    if let Ok(mut lock) = ENGINE.lock() {
        if let Some(engine) = lock.as_mut() {
            engine.set_viet_mode(enabled);
            engine.reset();
        }
    }
    reset_all();
    notify_ui_state_changed();
}

/// Đổi qua lại giữa chế độ V và E
pub fn toggle_viet_mode() -> bool {
    let new_mode = {
        let mut vm = VIET_MODE.lock().unwrap();
        *vm = !*vm;
        *vm
    };
    if let Ok(mut lock) = ENGINE.lock() {
        if let Some(engine) = lock.as_mut() {
            engine.set_viet_mode(new_mode);
            engine.reset();
        }
    }
    reset_all();
    notify_ui_state_changed();
    new_mode
}

/// Khởi tạo Engine với kiểu gõ chỉ định (Telex hoặc VNI)
pub fn init_engine_with_method(im: InputMethod) {
    let mut engine = Engine::new();
    engine.set_input_method(im);
    engine.set_viet_mode(is_viet_mode());

    let mut opts = Options::default();
    opts.free_marking = true;
    opts.modern_style = true;
    opts.spell_check_enabled = true;
    opts.auto_non_vn_restore = true;
    engine.options = opts;

    if let Ok(mut lock) = ENGINE.lock() {
        *lock = Some(engine);
    }
    if let Ok(mut m) = CURRENT_METHOD.lock() {
        *m = im;
    }
}

/// Lấy kiểu gõ hiện tại
pub fn get_current_method() -> InputMethod {
    if let Ok(m) = CURRENT_METHOD.lock() {
        *m
    } else {
        InputMethod::Telex
    }
}

/// Thiết lập kiểu gõ trực tiếp
pub fn set_input_method(im: InputMethod) {
    if let Ok(mut cur) = CURRENT_METHOD.lock() {
        *cur = im;
    }
    if let Ok(mut lock) = ENGINE.lock() {
        if let Some(engine) = lock.as_mut() {
            engine.set_input_method(im);
            engine.reset();
        }
    }
    reset_all();
    notify_ui_state_changed();
    println!("\n>>> [THIẾT LẬP KIỂU GÕ] Hiện tại: {:?} <<<", im);
}

/// Chuyển đổi qua lại giữa TELEX và VNI
pub fn toggle_input_method() -> InputMethod {
    let new_im = {
        let mut cur = CURRENT_METHOD.lock().unwrap();
        let next = match *cur {
            InputMethod::Telex => InputMethod::Vni,
            InputMethod::Vni => InputMethod::Telex,
            _ => InputMethod::Telex,
        };
        *cur = next;
        next
    };

    if let Ok(mut lock) = ENGINE.lock() {
        if let Some(engine) = lock.as_mut() {
            engine.set_input_method(new_im);
            engine.reset();
        }
    }
    reset_all();
    notify_ui_state_changed();

    println!("\n>>> [CHUYỂN KIỂU GÕ] Hiện tại: {:?} <<<", new_im);
    new_im
}

/// Chuyển phím ảo sang ASCII theo đúng trạng thái bàn phím
unsafe fn vk_to_ascii(vk: u32, scan_code: u32) -> u8 {
    let shift = unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 };
    let caps = unsafe { (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0 };
    let upper = shift ^ caps;

    if (0x41..=0x5A).contains(&vk) {
        return if upper { vk as u8 } else { vk as u8 + 32 };
    }

    let mut key_state = [0u8; 256];
    if shift {
        key_state[VK_SHIFT.0 as usize] = 0x80;
    }
    if caps {
        key_state[VK_CAPITAL.0 as usize] = 0x01;
    }
    let mut buf = [0u16; 4];
    let ret = unsafe { ToUnicode(vk, scan_code, Some(&key_state), &mut buf, 0) };
    if ret == 1 && buf[0] > 0 && buf[0] <= 127 {
        return buf[0] as u8;
    }

    0
}

/// Gửi backspaces và text unicode bằng SendInput
fn send_output(backspaces: usize, text: &str) {
    let mut inputs: Vec<INPUT> = Vec::new();

    // Gửi backspace
    for _ in 0..backspaces {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: VNKEY_INJECTED_TAG,
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: VNKEY_INJECTED_TAG,
                },
            },
        });
    }

    // Gửi text thay thế
    for ch in text.encode_utf16() {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: VNKEY_INJECTED_TAG,
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: VNKEY_INJECTED_TAG,
                },
            },
        });
    }

    unsafe {
        if let Ok(mut injecting) = IS_INJECTING.lock() {
            *injecting = true;
        }
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if let Ok(mut injecting) = IS_INJECTING.lock() {
            *injecting = false;
        }
    }
}

/// Kiểm tra và khôi phục khi kết thúc từ (Space / Enter / Punctuation):
/// 1. Nếu từ hiển thị là âm tiết tiếng Việt hợp lệ -> Giữ nguyên tiếng Việt.
/// 2. Nếu từ hiển thị KHÔNG phải tiếng Việt hợp lệ:
///    - Nếu từ phím thô (raw) có nghĩa tiếng Anh (hoặc là từ tiếng Anh chuẩn) -> Trả về từ phím thô (ví dụ: passwork, worker, server...).
///    - Nếu từ phím thô KHÔNG có nghĩa tiếng Anh (ví dụ 'thiss'), nhưng từ đang hiển thị lại có nghĩa tiếng Anh (ví dụ 'this') -> Giữ nguyên từ hiển thị trước đó!
///    - Nếu cả hai đều không phải, mặc định trả về từ phím thô.
fn check_and_restore_if_invalid_vietnamese() -> bool {
    let mut should_restore = false;
    let mut backs = 0;
    let mut text_to_restore = String::new();

    if let Ok(tracker) = TRACKER.lock() {
        if !tracker.current_raw.is_empty() && tracker.current_raw != tracker.current_committed {
            let raw_is_en = is_english_word(&tracker.current_raw);
            let committed_is_vn = is_valid_vietnamese_syllable(&tracker.current_committed);
            let committed_is_en = is_english_word(&tracker.current_committed);

            if !committed_is_vn {
                // Chỉ xử lý khi từ hiển thị KHÔNG PHẢI TIẾNG VIỆT HỢP LỆ:
                if committed_is_en {
                    // Từ hiển thị (ví dụ: 'if' khi gõ 'iff', 'his' khi gõ 'hiss', 'this' khi gõ 'thiss')
                    // là từ tiếng Anh có nghĩa -> GIỮ NGUYÊN từ hiển thị ('if', 'his', 'this')!
                    should_restore = false;
                } else if raw_is_en {
                    // Từ phím thô là từ tiếng Anh có nghĩa (ví dụ 'passwork', 'worker', 'password') -> Hoàn tác về phím thô
                    should_restore = true;
                    backs = tracker.current_committed.chars().count();
                    text_to_restore = tracker.current_raw.clone();
                } else {
                    // Cả hai đều không phải -> Hoàn tác về phím thô
                    should_restore = true;
                    backs = tracker.current_committed.chars().count();
                    text_to_restore = tracker.current_raw.clone();
                }
            } else {
                // Từ hiển thị LÀ TIẾNG VIỆT CÓ NGHĨA -> GIỮ NGUYÊN 100% TIẾNG VIỆT!
                should_restore = false;
            }
        }
    }

    if should_restore && backs > 0 {
        send_output(backs, &text_to_restore);

        if let Ok(mut tracker) = TRACKER.lock() {
            tracker.current_committed = text_to_restore;
        }
        return true;
    }

    false
}

/// Reset trạng thái sạch sẽ hoàn toàn
fn reset_all() {
    if let Ok(mut guard) = ENGINE.lock() {
        if let Some(engine) = guard.as_mut() {
            engine.reset();
        }
    }
    if let Ok(mut tracker) = TRACKER.lock() {
        tracker.current_raw.clear();
        tracker.current_committed.clear();
        tracker.history_words.clear();
        tracker.after_space = false;
    }
}

/// Callback Keyboard Hook cấp thấp
unsafe extern "system" fn ll_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let msg = wparam.0 as u32;

        let kb = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

        // Bỏ qua phím do chính bộ gõ inject
        if kb.dwExtraInfo == VNKEY_INJECTED_TAG {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }
        if let Ok(injecting) = IS_INJECTING.lock() {
            if *injecting {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }
        }

        let vk = kb.vkCode;
        let is_ctrl_key = vk == VK_CONTROL.0 as u32
            || vk == VK_LCONTROL.0 as u32
            || vk == VK_RCONTROL.0 as u32;
        let is_shift_key = vk == VK_SHIFT.0 as u32
            || vk == VK_LSHIFT.0 as u32
            || vk == VK_RSHIFT.0 as u32;

        // XỬ LÝ PHÍM TẮT CTRL + SHIFT BẬT / TẮT TIẾNG VIỆT
        // Kiểm tra trạng thái phím vật lý hiện tại
        let ctrl_down = unsafe {
            ((GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0)
                || ((GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0)
                || ((GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0)
        };
        let shift_down = unsafe {
            ((GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0)
                || ((GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16 & 0x8000) != 0)
                || ((GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000) != 0)
        };
        let alt_down = unsafe { (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 };
        let win_down = unsafe {
            ((GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0)
                || ((GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0)
        };

        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            // Khi nhấn phím khác (không phải Ctrl hay Shift), hủy trạng thái chờ
            if !is_ctrl_key && !is_shift_key {
                *CTRL_SHIFT_PENDING.lock().unwrap() = false;
            } else if !alt_down && !win_down {
                // Đang nhấn phím Ctrl hoặc Shift
                // Nếu cả hai cùng đang được đè xuống -> đánh dấu kích hoạt
                let both_down = (is_ctrl_key && shift_down) || (is_shift_key && ctrl_down) || (ctrl_down && shift_down);
                if both_down {
                    *CTRL_SHIFT_PENDING.lock().unwrap() = true;
                }
            }
        }

        if msg == WM_KEYUP || msg == WM_SYSKEYUP {
            if is_ctrl_key || is_shift_key {
                let mut pending = CTRL_SHIFT_PENDING.lock().unwrap();
                if *pending {
                    *pending = false;
                    toggle_viet_mode();
                    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
                }
            }
        }

        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            // Phím F9: Phím tắt chuyển đổi nhanh giữa TELEX và VNI
            if VIRTUAL_KEY(vk as u16) == VK_F9 {
                toggle_input_method();
                return LRESULT(1);
            }

            // Bỏ qua nếu là phím modifier đơn thuần
            if is_ctrl_key || is_shift_key || vk == VK_MENU.0 as u32 || vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 1. NẾU ĐANG TẮT TIẾNG VIỆT (Chế độ E) -> BỎ QUA HOÀN TOÀN
            if !is_viet_mode() {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 2. BỎ QUA TỔ HỢP PHÍM (Ctrl+A, Ctrl+C, Ctrl+V, Alt+Tab...)
            if ctrl_down || alt_down || win_down {
                reset_all();
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 2. DI CHUYỂN CON TRỎ (Home, End, Mũi tên, Escape) -> Reset bộ đệm
            match VIRTUAL_KEY(vk as u16) {
                VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_ESCAPE => {
                    reset_all();
                    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
                }
                _ => {}
            }

            // 3. XỬ LÝ PHÍM SPACE
            if VIRTUAL_KEY(vk as u16) == VK_SPACE {
                check_and_restore_if_invalid_vietnamese();

                if let Ok(mut tracker) = TRACKER.lock() {
                    if !tracker.current_raw.is_empty() {
                        let entry = WordEntry {
                            raw_typed: tracker.current_raw.clone(),
                            committed_text: tracker.current_committed.clone(),
                        };
                        tracker.history_words.push(entry);
                        tracker.current_raw.clear();
                        tracker.current_committed.clear();
                    }
                    tracker.after_space = true;
                }

                if let Ok(mut guard) = ENGINE.lock() {
                    if let Some(engine) = guard.as_mut() {
                        engine.soft_reset();
                    }
                }
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 4. KẾT THÚC DÒNG (ENTER / TAB)
            if VIRTUAL_KEY(vk as u16) == VK_RETURN || VIRTUAL_KEY(vk as u16) == VK_TAB {
                check_and_restore_if_invalid_vietnamese();
                reset_all();
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 5. XỬ LÝ PHÍM BACKSPACE
            if VIRTUAL_KEY(vk as u16) == VK_BACK {
                let mut needs_restore_context = false;
                let mut context_to_feed = String::new();

                if let Ok(mut tracker) = TRACKER.lock() {
                    if tracker.after_space {
                        tracker.after_space = false;
                        if let Some(prev_word) = tracker.history_words.pop() {
                            tracker.current_raw = prev_word.raw_typed.clone();
                            tracker.current_committed = prev_word.committed_text.clone();
                            context_to_feed = prev_word.raw_typed;
                            needs_restore_context = true;
                        }
                    } else if !tracker.current_raw.is_empty() {
                        tracker.current_raw.pop();
                        tracker.current_committed.pop();
                    } else {
                        if let Some(prev_word) = tracker.history_words.pop() {
                            tracker.current_raw = prev_word.raw_typed.clone();
                            tracker.current_committed = prev_word.committed_text.clone();
                            context_to_feed = prev_word.raw_typed;
                            needs_restore_context = true;
                        }
                    }
                }

                if needs_restore_context {
                    if let Ok(mut guard) = ENGINE.lock() {
                        if let Some(engine) = guard.as_mut() {
                            engine.feed_context(&context_to_feed);
                        }
                    }
                    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
                }

                let is_completely_empty = if let Ok(tracker) = TRACKER.lock() {
                    tracker.current_raw.is_empty() && tracker.history_words.is_empty()
                } else {
                    true
                };

                if is_completely_empty {
                    reset_all();
                    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
                }

                if let Ok(mut guard) = ENGINE.lock() {
                    if let Some(engine) = guard.as_mut() {
                        let res = engine.process_backspace();
                        if res.processed && res.backspaces > 1 {
                            let utf8_text = String::from_utf8_lossy(&res.output).to_string();
                            send_output(res.backspaces, &utf8_text);
                            if let Ok(mut tracker) = TRACKER.lock() {
                                for _ in 0..res.backspaces {
                                    tracker.current_committed.pop();
                                }
                                tracker.current_committed.push_str(&utf8_text);
                            }
                            return LRESULT(1);
                        }
                    }
                }
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // 6. XỬ LÝ PHÍM KÝ TỰ (Chữ A-Z và Số 0-9 cho VNI)
            let ascii = unsafe { vk_to_ascii(vk, kb.scanCode) };
            if ascii > 0 {
                let typed_char = ascii as char;

                // Nếu là ký tự dấu câu phân cách từ (ngoại trừ số 0-9 khi dùng VNI)
                if !typed_char.is_alphanumeric() {
                    check_and_restore_if_invalid_vietnamese();
                    reset_all();
                    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
                }

                if let Ok(mut tracker) = TRACKER.lock() {
                    if tracker.after_space {
                        tracker.after_space = false;
                    }
                    tracker.current_raw.push(typed_char);
                }

                if let Ok(mut guard) = ENGINE.lock() {
                    if let Some(engine) = guard.as_mut() {
                        let res = engine.process(ascii as u32);
                        if res.processed && (res.backspaces > 0 || !res.output.is_empty()) {
                            let utf8_text = String::from_utf8_lossy(&res.output).to_string();
                            send_output(res.backspaces, &utf8_text);

                            if let Ok(mut tracker) = TRACKER.lock() {
                                for _ in 0..res.backspaces {
                                    tracker.current_committed.pop();
                                }
                                tracker.current_committed.push_str(&utf8_text);
                            }

                            return LRESULT(1);
                        } else {
                            if let Ok(mut tracker) = TRACKER.lock() {
                                tracker.current_committed.push(typed_char);
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Callback Mouse Hook: Reset buffer khi click chuột chuyển trường nhập (giống Unikey/OpenKey)
unsafe extern "system" fn ll_mouse_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let msg = wparam.0 as u32;
        if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN {
            reset_all();
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Cài đặt các hooks cấp thấp
pub fn install_hooks() -> bool {
    unsafe {
        let khook = match SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(ll_keyboard_proc),
            None,
            0,
        ) {
            Ok(h) => h,
            Err(_) => return false,
        };

        if let Ok(mut lock) = KEYBOARD_HOOK.lock() {
            *lock = Some(SafeHHOOK(khook));
        }

        if let Ok(mhook) = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(ll_mouse_proc),
            None,
            0,
        ) {
            if let Ok(mut lock) = MOUSE_HOOK.lock() {
                *lock = Some(SafeHHOOK(mhook));
            }
        }
    }
    true
}

/// Gỡ bỏ các hooks
pub fn uninstall_hooks() {
    unsafe {
        if let Ok(mut lock) = KEYBOARD_HOOK.lock() {
            if let Some(h) = lock.take() {
                let _ = UnhookWindowsHookEx(h.0);
            }
        }
        if let Ok(mut lock) = MOUSE_HOOK.lock() {
            if let Some(h) = lock.take() {
                let _ = UnhookWindowsHookEx(h.0);
            }
        }
    }
}

/// Khởi chạy toàn bộ dịch vụ bàn phím hệ thống dạng console
pub fn start_keyboard_service() {
    init_engine_with_method(InputMethod::Telex);

    println!("===============================================================");
    println!("  SMART VIETNAMESE IME - HỖ TRỢ ĐA KIỂU GÕ (TELEX & VNI)");
    println!("  - Kiểu gõ mặc định: TELEX");
    println!("  - Phím tắt chuyển đổi TELEX <-> VNI: bấm F9 hoặc Ctrl + Shift");
    println!("  - Quy tắc VNI chuẩn: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng,");
    println!("                       6=mũ (^), 7=móc (ơ,ư), 8=trăng (ă), 9=đ, 0=xoá dấu");
    println!("  - Tự động hoàn tác về tiếng Anh nếu từ không có nghĩa!");
    println!("===============================================================");

    if install_hooks() {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                DispatchMessageW(&msg);
            }
            uninstall_hooks();
        }
    }
}
