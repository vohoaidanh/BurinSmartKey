//! Module kiểm tra tính hợp lệ của âm tiết Tiếng Việt (Vietnamese Phonology & Syllable Trie)

use std::collections::HashSet;
use std::sync::LazyLock;

/// Danh sách ~10,030 âm tiết tiếng Việt hợp lệ được nhúng trực tiếp
static VIETNAMESE_SYLLABLES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let content = include_str!("../../vietnamese_syllables_clean.txt");
    content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect()
});

/// Kiểm tra một từ có phải là một âm tiết tiếng Việt hợp lệ hay không
pub fn is_valid_vietnamese_syllable(word: &str) -> bool {
    let lower = word.to_lowercase();
    VIETNAMESE_SYLLABLES.contains(lower.as_str())
}

/// Kiểm tra từ sau khi biến đổi hoặc từ thô có chứa các mẫu đặc trưng của tiếng Anh
pub fn is_impossible_vietnamese_word(transformed: &str, raw_keys: &str) -> bool {
    let t_lower = transformed.to_lowercase();
    let r_lower = raw_keys.to_lowercase();

    // 1. Sau khi biến đổi telex mà vẫn còn nguyên 'w', 'j', 'f', 'z' thì không phải tiếng Việt
    // Lưu ý: Trong tiếng Việt, 'w' biến thành 'ư'/'ă', 'j' thành dấu nặng, 'f' thành dấu huyền.
    // Nếu vẫn còn sót lại ký tự này (ví dụ 'passwork' -> 'paưỏk' còn 'k' sau 'ỏ' hoặc còn 'w')
    if t_lower.contains('w') || t_lower.contains('j') || t_lower.contains('f') || t_lower.contains('z') {
        return true;
    }

    // 2. Chứa các cụm phụ âm kép kiểu tiếng Anh: "rk", "rt", "st", "sp", "sk", "sh", "ct", "ft", "ld", "lk", "sw"
    const ENGLISH_CLUSTERS: &[&str] = &[
        "rk", "rt", "st", "sp", "sk", "sh", "ct", "ft", "ld", "lk", "sw", "tw",
        "ssw", "ss", "ll", "bb", "ff", "gg", "mm", "pp", "rr", "tt"
    ];

    for &cluster in ENGLISH_CLUSTERS {
        if r_lower.contains(cluster) {
            return true;
        }
    }

    // 3. Các kết hợp nguyên âm dị dạng mà Telex tạo ra khi gõ nhầm tiếng Anh:
    // "paưỏk", "ửo", "ưo", "okẻ", "ươk"
    if t_lower.contains("ưo") || t_lower.contains("ửo") || t_lower.contains("aư") || t_lower.contains("okẻ") || t_lower.contains("ỏk") {
        return true;
    }

    false
}
