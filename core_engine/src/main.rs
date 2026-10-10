#![windows_subsystem = "windows"]

use core_engine::ui::run_ui_app;
use core_engine::windows_hook::init_engine_with_method;
use vnkey_engine::InputMethod;

fn main() {
    // Khởi tạo engine ngầm định Telex
    init_engine_with_method(InputMethod::Telex);

    // Chạy ứng dụng giao diện hiện đại
    run_ui_app();
}
