pub mod phonology;
pub mod dictionary;
pub mod telex;
pub mod windows_hook;
pub mod ui;
pub mod tray_icons;

use phonology::{is_impossible_vietnamese_word, is_valid_vietnamese_syllable};
use dictionary::{is_english_word, is_english_prefix};
use telex::telex_transform;

/// Kết quả xử lý của Bộ gõ thông minh
#[derive(Debug, PartialEq, Eq)]
pub enum EngineResult {
    /// Xuất ra tiếng Việt đã biến đổi
    Vietnamese(String),
    /// Giữ nguyên chuỗi tiếng Anh thô
    English(String),
}

/// Trọng tâm xử lý thông minh (Smart Bilingual Engine):
/// 1. Nếu từ thô là từ tiếng Anh hoàn chỉnh (passwork, password, worker, server, works, tests...) -> Giữ tiếng Anh
/// 2. Chuyển đổi thử theo quy tắc Telex
/// 3. Nếu từ sau biến đổi hoặc phím thô chứa các mẫu bất thường đặc trưng tiếng Anh (như 'paưỏk', 'ửokẻ', 'ssw', 'rk'...) -> Giữ tiếng Anh
/// 4. Nếu từ sau biến đổi là âm tiết TIẾNG VIỆT HỢP LỆ (trong từ điển 10,030 âm tiết) -> Xuất tiếng Việt
/// 5. Nếu từ biến đổi không có trong từ điển tiếng Việt và phím thô có thể là tiếng Anh -> Giữ tiếng Anh
pub fn process_input_word(raw_keys: &str) -> EngineResult {
    if raw_keys.is_empty() {
        return EngineResult::Vietnamese(String::new());
    }

    let lower = raw_keys.to_lowercase();

    // 1. Kiểm tra từ tiếng Anh hoàn chỉnh
    if is_english_word(&lower) {
        return EngineResult::English(raw_keys.to_string());
    }

    // 2. Chuyển đổi thử theo luật Telex
    let transformed = telex_transform(raw_keys);

    // 3. Nếu xuất hiện các hình thái dị dạng đặc trưng gõ nhầm Telex (như paưỏk, ửokẻ, ssw, rk...)
    if is_impossible_vietnamese_word(&transformed, &lower) {
        return EngineResult::English(raw_keys.to_string());
    }

    // 4. Nếu là âm tiết tiếng Việt hợp lệ chuẩn ngữ âm học
    if is_valid_vietnamese_syllable(&transformed) {
        return EngineResult::Vietnamese(transformed);
    }

    // 5. Nếu không phải tiếng Việt hợp lệ, kiểm tra xem có phải tiền tố từ tiếng Anh không
    if is_english_prefix(&lower) {
        return EngineResult::English(raw_keys.to_string());
    }

    // Mặc định xuất ra tiếng Việt
    EngineResult::Vietnamese(transformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passwork_case() {
        // Gõ "passwork": Unikey ra "Paưỏk", Bộ gõ thông minh giữ "passwork"
        assert_eq!(
            process_input_word("passwork"),
            EngineResult::English("passwork".to_string())
        );
    }

    #[test]
    fn test_password_case() {
        assert_eq!(
            process_input_word("password"),
            EngineResult::English("password".to_string())
        );
    }

    #[test]
    fn test_worker_case() {
        assert_eq!(
            process_input_word("worker"),
            EngineResult::English("worker".to_string())
        );
    }

    #[test]
    fn test_server_case() {
        assert_eq!(
            process_input_word("server"),
            EngineResult::English("server".to_string())
        );
    }

    #[test]
    fn test_valid_vietnamese() {
        assert_eq!(
            process_input_word("vieetj"),
            EngineResult::Vietnamese("việt".to_string())
        );
        assert_eq!(
            process_input_word("banj"),
            EngineResult::Vietnamese("bạn".to_string())
        );
        assert_eq!(
            process_input_word("chaof"),
            EngineResult::Vietnamese("chào".to_string())
        );
        assert_eq!(
            process_input_word("nguowfi"),
            EngineResult::Vietnamese("người".to_string())
        );
    }
}
