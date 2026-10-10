# Đọc từ điển tiếng Anh dwyl/english-words và lọc ra danh sách từ phổ biến (từ 2 đến 15 ký tự)
with open(r"D:\Project\smart-vietnamese-ime\english_words.txt", "r", encoding="utf-8") as f:
    words = [line.strip().lower() for line in f if line.strip()]

# Lọc các từ hợp lệ gồm chữ cái a-z
clean_words = set()
for w in words:
    if 2 <= len(w) <= 20 and w.isalpha():
        clean_words.add(w)

print(f"Tổng số từ tiếng Anh sạch: {len(clean_words)}")

# Ghi ra file english_words_clean.txt
with open(r"D:\Project\smart-vietnamese-ime\english_words_clean.txt", "w", encoding="utf-8") as f:
    for w in sorted(clean_words):
        f.write(w + "\n")
