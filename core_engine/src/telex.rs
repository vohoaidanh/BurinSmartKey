//! Module xử lý ngữ âm và đặt dấu tiếng Việt chuẩn
//! Tiếng Việt chuẩn theo Viện Ngôn ngữ học

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    None,
    Sac,   // 's'
    Huyen, // 'f'
    Hoi,   // 'r'
    Nga,   // 'x'
    Nang,  // 'j'
}

/// Bảng dấu cho từng nguyên âm: [Gốc, Sắc, Huyền, Hỏi, Ngã, Nặng]
const VOWEL_TABLE: &[[char; 6]] = &[
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
];

pub fn apply_tone(c: char, tone: Tone) -> char {
    let tone_idx = match tone {
        Tone::None => 0,
        Tone::Sac => 1,
        Tone::Huyen => 2,
        Tone::Hoi => 3,
        Tone::Nga => 4,
        Tone::Nang => 5,
    };
    for row in VOWEL_TABLE {
        if row[0] == c {
            return row[tone_idx];
        }
    }
    c
}

pub fn remove_tone(c: char) -> (char, Tone) {
    for row in VOWEL_TABLE {
        for (idx, &v) in row.iter().enumerate() {
            if v == c {
                let tone = match idx {
                    1 => Tone::Sac,
                    2 => Tone::Huyen,
                    3 => Tone::Hoi,
                    4 => Tone::Nga,
                    5 => Tone::Nang,
                    _ => Tone::None,
                };
                return (row[0], tone);
            }
        }
    }
    (c, Tone::None)
}

/// Tìm vị trí đặt dấu chuẩn ngữ pháp tiếng Việt:
/// - Nếu có 1 nguyên âm: đặt vào đó.
/// - Nếu có cụm "ươ", "ưa", "uô", "iê": đặt vào nguyên âm thứ 2 (ơ, ê, ô...).
/// - Nếu kết thúc bằng nguyên âm đôi không có phụ âm cuối: "oa", "oe", "uy" đặt vào âm thứ 2; còn lại đặt vào âm thứ 1 ("hòa" vs "hoà" - chuẩn mới đặt ở âm chính).
pub fn find_tone_position(vowels: &[(usize, char)], has_coda: bool) -> usize {
    if vowels.is_empty() {
        return 0;
    }
    if vowels.len() == 1 {
        return vowels[0].0;
    }

    let v_str: String = vowels.iter().map(|(_, c)| *c).collect();
    
    // Các nguyên âm đôi có dấu mũ/móc ưu tiên nhận dấu: ươ, uô, iê
    for (i, (_, c)) in vowels.iter().enumerate() {
        if *c == 'ơ' || *c == 'ê' || *c == 'ô' {
            return vowels[i].0;
        }
    }

    // Nếu có phụ âm cuối (coda), dấu đặt ở nguyên âm thứ 2 (ví dụ: toàn, hoàng, muốn)
    if has_coda {
        return vowels[vowels.len() - 1].0;
    }

    // Nếu không có phụ âm cuối:
    // Cặp oa, oe, uy: chuẩn thường đặt ở 'a', 'e', 'y' hoặc 'o' tùy trường phái; chuẩn tự nhiên: chào -> đặt ở 'a'
    if v_str == "ao" || v_str == "au" || v_str == "ay" || v_str == "ai" {
        return vowels[0].0; // chào, chàu, cháy, chái -> dấu ở 'a'
    }
    if v_str == "eo" || v_str == "eu" {
        return vowels[0].0; // khéo -> dấu ở 'e'
    }
    if v_str == "ia" || v_str == "ua" || v_str == "ưa" {
        return vowels[0].0; // mía, múa, mưa -> dấu ở 'i', 'u', 'ư'
    }
    if v_str == "oa" || v_str == "oe" || v_str == "uy" {
        return vowels[1].0; // hoà/hóa, hoè, tuý -> đặt ở âm thứ 2
    }

    // Mặc định: đặt ở nguyên âm cuối nếu 2 nguyên âm
    vowels[vowels.len() - 1].0
}

/// Chuyển đổi chuỗi phím thô theo quy tắc Telex chuẩn
pub fn telex_transform(raw: &str) -> String {
    let mut base_chars: Vec<char> = Vec::new();
    let mut current_tone = Tone::None;

    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let next2 = chars.get(i + 2).copied();

        // 1. Phím dấu thanh (s, f, r, x, j)
        match c.to_ascii_lowercase() {
            's' => { current_tone = Tone::Sac; i += 1; continue; }
            'f' => { current_tone = Tone::Huyen; i += 1; continue; }
            'r' => { current_tone = Tone::Hoi; i += 1; continue; }
            'x' => { current_tone = Tone::Nga; i += 1; continue; }
            'j' => { current_tone = Tone::Nang; i += 1; continue; }
            _ => {}
        }

        // 2. Phím kép Telex
        match (c.to_ascii_lowercase(), next.map(|x| x.to_ascii_lowercase()), next2.map(|x| x.to_ascii_lowercase())) {
            ('a', Some('a'), _) => { base_chars.push('â'); i += 2; }
            ('a', Some('w'), _) => { base_chars.push('ă'); i += 2; }
            ('e', Some('e'), _) => { base_chars.push('ê'); i += 2; }
            ('o', Some('o'), _) => { base_chars.push('ô'); i += 2; }
            ('o', Some('w'), _) => { base_chars.push('ơ'); i += 2; }
            ('u', Some('o'), Some('w')) => {
                base_chars.push('ư');
                base_chars.push('ơ');
                i += 3;
            }
            ('u', Some('w'), _) => { base_chars.push('ư'); i += 2; }
            ('d', Some('d'), _) => { base_chars.push('đ'); i += 2; }
            ('w', _, _) => {
                base_chars.push('ư');
                i += 1;
            }
            _ => {
                base_chars.push(c);
                i += 1;
            }
        }
    }

    if current_tone == Tone::None {
        return base_chars.into_iter().collect();
    }

    // 3. Đặt dấu thanh vào vị trí đúng
    let vowel_set = "aăâeêioôơuưy";
    let mut vowel_indices = Vec::new();
    let mut last_vowel_idx = None;

    for (idx, &ch) in base_chars.iter().enumerate() {
        if vowel_set.contains(ch) {
            vowel_indices.push((idx, ch));
            last_vowel_idx = Some(idx);
        }
    }

    if vowel_indices.is_empty() {
        return base_chars.into_iter().collect();
    }

    // Có phụ âm cuối hay không
    let has_coda = match last_vowel_idx {
        Some(idx) => idx < base_chars.len() - 1,
        None => false,
    };

    let target_idx = find_tone_position(&vowel_indices, has_coda);
    base_chars[target_idx] = apply_tone(base_chars[target_idx], current_tone);

    base_chars.into_iter().collect()
}
