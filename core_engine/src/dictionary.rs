//! Module từ điển tiếng Anh: Kiểm tra từ vựng tiếng Anh
use std::collections::HashSet;
use std::sync::LazyLock;

/// Top 20,000 từ tiếng Anh phổ biến nhất được nạp sẵn
static COMMON_ENGLISH_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    let content = include_str!("../../english_words_clean.txt");
    for line in content.lines() {
        let w = line.trim();
        if !w.is_empty() {
            set.insert(w);
        }
    }
    // Bổ sung các từ kỹ thuật/viết tắt/thuật ngữ phổ biến
    let extra = [
        "passwork", "password", "passwords", "worker", "workers", "working",
        "server", "servers", "service", "services", "system", "systems",
        "browser", "browsers", "developer", "development", "frontend", "backend",
        "javascript", "typescript", "python", "golang", "rust", "react", "angular",
        "login", "logout", "signin", "signup", "register", "admin", "dashboard",
        "database", "network", "client", "connection", "request", "response"
    ];
    for &w in &extra {
        set.insert(w);
    }
    set
});

/// Kiểm tra một từ thô có phải là từ tiếng Anh
pub fn is_english_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    COMMON_ENGLISH_WORDS.contains(lower.as_str())
}

/// Kiểm tra xem một tiền tố có khả năng là từ tiếng Anh không
pub fn is_english_prefix(prefix: &str) -> bool {
    let lower = prefix.to_lowercase();
    if lower.len() < 3 {
        return false;
    }
    COMMON_ENGLISH_WORDS.iter().any(|w| w.starts_with(&lower))
}
