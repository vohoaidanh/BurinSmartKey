//! Comprehensive unit tests for real-world typing sequence simulation

use core_engine::process_input_word;
use core_engine::EngineResult;

#[cfg(test)]
mod tests {
    use super::*;

    /// Giả lập việc người dùng gõ từng phím một
    fn simulate_keystrokes(keys: &str) -> Vec<(String, EngineResult)> {
        let mut buffer = String::new();
        let mut steps = Vec::new();
        for ch in keys.chars() {
            buffer.push(ch);
            let res = process_input_word(&buffer);
            steps.push((buffer.clone(), res));
        }
        steps
    }

    #[test]
    fn test_step_by_step_worker() {
        let steps = simulate_keystrokes("worker");
        println!("\n=== Trình tự gõ: 'worker' ===");
        for (buf, res) in &steps {
            println!("  Buffer: {:<10} => {:?}", buf, res);
        }

        // Khi gõ đủ 'worker', kết quả BẮT BUỘC phải là English("worker")
        let last = steps.last().unwrap();
        assert_eq!(last.1, EngineResult::English("worker".to_string()));
    }

    #[test]
    fn test_step_by_step_vietnamese_viet() {
        let steps = simulate_keystrokes("vieetj");
        println!("\n=== Trình tự gõ: 'vieetj' ===");
        for (buf, res) in &steps {
            println!("  Buffer: {:<10} => {:?}", buf, res);
        }

        let last = steps.last().unwrap();
        assert_eq!(last.1, EngineResult::Vietnamese("việt".to_string()));
    }

    #[test]
    fn test_step_by_step_vietnamese_nguoi() {
        let steps = simulate_keystrokes("nguowfi");
        println!("\n=== Trình tự gõ: 'nguowfi' ===");
        for (buf, res) in &steps {
            println!("  Buffer: {:<10} => {:?}", buf, res);
        }

        let last = steps.last().unwrap();
        assert_eq!(last.1, EngineResult::Vietnamese("người".to_string()));
    }

    #[test]
    fn test_many_vietnamese_words() {
        // Danh sách các từ tiếng Việt thông dụng
        let words = vec![
            ("chaof", "chào"),
            ("banj", "bạn"),
            ("tooi", "tôi"),
            ("yeeu", "yêu"),
            ("queeh", "quêh"),
            ("huowng", "hương"),
            ("ddaats", "đất"),
            ("nuowsc", "nước"),
        ];

        for (raw, expected) in words {
            let res = process_input_word(raw);
            assert_eq!(
                res,
                EngineResult::Vietnamese(expected.to_string()),
                "Thất bại khi gõ '{}', mong đợi '{}'",
                raw,
                expected
            );
        }
    }

    #[test]
    fn test_many_english_words() {
        // Danh sách các từ tiếng Anh không được phép bị lỗi telex
        let english = vec![
            "work", "worker", "works", "world", "word",
            "server", "service", "system",
            "password", "pass",
            "game", "code", "file", "test", "reset", "user",
            "software", "network", "browser", "warning", "error",
            "this", "if", "his", "that", "with", "have", "from", "they", "which"
        ];

        for word in english {
            let res = process_input_word(word);
            assert_eq!(
                res,
                EngineResult::English(word.to_string()),
                "Từ tiếng Anh '{}' bị biến đổi sai thành tiếng Việt!",
                word
            );
        }
    }
}
