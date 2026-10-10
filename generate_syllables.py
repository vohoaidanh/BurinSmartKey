# Script sinh danh sách toàn bộ âm tiết tiếng Việt hợp lệ chuẩn ngữ âm học
# Dựa trên cấu trúc chuẩn: Phụ âm đầu + Âm đệm/chính + Phụ âm cuối + Thanh điệu

INITIALS = ["", "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", 
            "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x"]

# Các vần chuẩn tiếng Việt
RHYMES = [
    # Đơn & nguyên âm đôi
    "a", "á", "à", "ả", "ã", "ạ",
    "ă", "ắ", "ằ", "ẳ", "ẵ", "ặ",
    "â", "ấ", "ầ", "ẩ", "ẫ", "ậ",
    "e", "é", "è", "ẻ", "ẽ", "ẹ",
    "ê", "ế", "ề", "ể", "ễ", "ệ",
    "i", "í", "ì", "ỉ", "ĩ", "ị",
    "o", "ó", "ò", "ỏ", "õ", "ọ",
    "ô", "ố", "ồ", "ổ", "ỗ", "ộ",
    "ơ", "ớ", "ờ", "ở", "ỡ", "ợ",
    "u", "ú", "ù", "ủ", "ũ", "ụ",
    "ư", "ứ", "ừ", "ử", "ữ", "ự",
    "y", "ý", "ỳ", "ỷ", "ỹ", "ỵ",
    # Vần ghép đôi/ba
    "ai", "ái", "ài", "ải", "ãi", "ại",
    "ao", "áo", "ào", "ảo", "ão", "ạo",
    "au", "áu", "àu", "ảu", "ãu", "ạu",
    "ay", "áy", "ày", "ảy", "ãy", "ạy",
    "em", "ém", "èm", "ẻm", "ẽm", "ẹm",
    "en", "én", "èn", "ẻn", "ẽn", "ẹn",
    "eng", "éng", "èng", "ẻng", "ẽng", "ẹng",
    "eo", "éo", "èo", "ẻo", "ẽo", "ẹo",
    "ep", "ép", "ẹp",
    "et", "ét", "ẹt",
    "in", "ín", "ìn", "ỉn", "ĩn", "ịn",
    "im", "ím", "ìm", "ỉm", "ĩm", "ịm",
    "ip", "íp", "ịp",
    "it", "ít", "ịt",
    "om", "óm", "òm", "ỏm", "õm", "ọm",
    "on", "ón", "òn", "ỏn", "õn", "ọn",
    "ong", "óng", "òng", "ỏng", "õng", "ọng",
    "op", "óp", "ọp",
    "ot", "ót", "ọt",
    "ung", "úng", "ùng", "ủng", "ũng", "ụng",
    "un", "ún", "ùn", "ủn", "ũn", "ụn",
    "um", "úm", "ùm", "ủm", "ũm", "ụm",
    "up", "úp", "ụp",
    "ut", "út", "ụt",
    "an", "án", "àn", "ản", "ãn", "ạn",
    "am", "ám", "àm", "ảm", "ãn", "ạm",
    "ang", "áng", "àng", "ảng", "ãng", "ạng",
    "anh", "ánh", "ành", "ảnh", "ãnh", "ạnh",
    "ap", "áp", "ạp",
    "at", "át", "ạt",
    "ach", "ách", "ạch",
    "ay", "áy", "ày", "ảy", "ãy", "ạy",
    "ơi", "ới", "ời", "ởi", "ỡi", "ợi",
    "ơi", "ơm", "ớm", "ờm", "ởm", "ỡm", "ợm",
    "ơn", "ớn", "ờn", "ởn", "ỡn", "ợn",
    "ơng", "ớng", "ờng", "ởng", "ỡng", "ợng",
    "ơp", "ớp", "ợp",
    "ơt", "ớt", "ợt",
    "ui", "úi", "ùi", "ủi", "ũi", "ụi",
    "uôi", "uối", "uồi", "uổi", "uỗi", "uội",
    "uông", "uống", "uồng", "uổng", "uỗng", "uộng",
    "uôn", "uốn", "uồn", "uển", "uỗn", "uộn",
    "ươi", "ưới", "ười", "ưởi", "ưỡi", "ượi",
    "ươn", "ướn", "ườn", "ưởn", "ưỡn", "ượn",
    "ương", "ướng", "ường", "ưởng", "ưỡng", "ượng",
    "ươm", "ướm", "ườm", "ưởm", "ưỡm", "ượm",
    "ươp", "ướp", "ượp",
    "ươt", "ướt", "ượt",
    "iên", "iến", "iền", "iển", "iễn", "iện",
    "iêm", "iếm", "iềm", "iểm", "iễm", "iệm",
    "iêng", "iếng", "iềng", "iểng", "iễng", "iệng",
    "iêp", "iếp", "iệp",
    "iêt", "iết", "iệt",
    "oa", "oá", "oà", "oả", "oã", "oạ",
    "oe", "oé", "oè", "oẻ", "oẽ", "oẹ",
    "oan", "oán", "oàn", "oản", "oãn", "oạn",
    "oang", "oáng", "oàng", "oảng", "oãng", "oạng",
    "oanh", "oánh", "oành", "oảnh", "oãnh", "oạnh",
    "oat", "oát", "oạt",
    "oac", "oác", "oạc",
    "oach", "oách", "oạch",
    "oay", "oáy", "oày", "oảy", "oãy", "oạy",
    "uê", "uế", "uề", "uể", "uễ", "uệ",
    "uy", "uý", "uỳ", "uỷ", "uỹ", "uỵ",
    "uyên", "uyến", "uyền", "uyển", "uyễn", "uyện",
    "uyêt", "uyết", "uyệt",
    "uynh", "uýnh", "uỳnh", "uỷnh", "uỹnh", "uỵnh",
    "uych", "uých", "uỵch"
]

valid_syllables = set()
for init in INITIALS:
    for r in RHYMES:
        # Quy tắc chính tả: k, gh, ngh chỉ đi với i, e, ê
        if init in ["k", "gh", "ngh"]:
            first_v = r[0]
            if first_v not in "ieêiéièỉiĩiịeéèẻẽẹêếềểễệ":
                continue
        # c, g, ng không đi với i, e, ê
        if init in ["c", "g", "ng"]:
            first_v = r[0]
            if first_v in "ieêiéièỉiĩiịeéèẻẽẹêếềểễệ":
                continue
        # qu đi với các vần bắt đầu bằng a, e, i, y, ê, o, u
        if init == "qu" and r.startswith(("u", "ư", "o", "ô", "ơ")):
            continue

        syllable = init + r
        valid_syllables.add(syllable)

print(f"Tổng số âm tiết hợp lệ sinh ra: {len(valid_syllables)}")
with open(r"D:\Project\smart-vietnamese-ime\vietnamese_syllables_clean.txt", "w", encoding="utf-8") as f:
    for s in sorted(valid_syllables):
        f.write(s + "\n")
