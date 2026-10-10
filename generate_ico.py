import math
from PIL import Image, ImageDraw, ImageFilter

def create_super_cute_piggy(size=512):
    """
    Tạo hình minh họa chú heo Burin siêu dễ thương chuẩn vector, 
    chất lượng 4K supersampled, phong cách 3D/flat mềm mại hiện đại:
    - Bo tròn mềm mại, má phúng phính đáng yêu
    - Tai vểnh tự nhiên có độ cong, tai trong hồng phấn
    - Đôi mắt to tròn long lanh anime (con ngươi to, 2 đốm sáng lấp lánh)
    - Mũi hình oval mềm có độ chuyển màu và 2 lỗ mũi bầu bĩnh
    - Hai má ửng hồng gradient e thẹn siêu cưng
    - Viền mỏng mềm sắc nét
    """
    scale = 4
    W = size * scale
    H = size * scale

    # Tạo canvas RGBA trong suốt
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx = W / 2.0
    cy = H / 2.0

    # Bảng màu chuẩn Kawaii Aesthetic
    PIG_BODY = (255, 178, 195, 255)       # Hồng đào tươi tắn mịn màng
    PIG_BODY_SHADOW = (248, 155, 175, 255)# Đổ bóng thân nhẹ
    PIG_EAR_OUTER = (255, 160, 180, 255)  # Tai ngoài
    PIG_EAR_INNER = (255, 205, 218, 255)  # Tai trong hồng phấn
    PIG_CHEEK = (255, 130, 155, 160)      # Má hồng đào tán mịn
    PIG_SNOUT = (255, 142, 165, 255)      # Mũi heo xinh
    PIG_SNOUT_LIGHT = (255, 175, 192, 255)# Highlight mũi
    PIG_EYE_DARK = (45, 30, 38, 255)      # Mắt đen nâu socola ấm (không thô như đen tuyền)
    NOSTRIL_DARK = (160, 70, 95, 255)     # Lỗ mũi màu đỏ hồng rượu nhẹ nhàng
    WHITE = (255, 255, 255, 255)

    # 1. TAI CHÚ HEO (Vểnh nghiêng dễ thương)
    # Tai trái
    ear_w = W * 0.22
    ear_h = H * 0.26
    
    # Vẽ tai trái ngoài
    draw.pieslice([cx - W * 0.42, cy - H * 0.44, cx - W * 0.08, cy - H * 0.02], 130, 310, fill=PIG_EAR_OUTER)
    # Tai phải ngoài
    draw.pieslice([cx + W * 0.08, cy - H * 0.44, cx + W * 0.42, cy - H * 0.02], 230, 50, fill=PIG_EAR_OUTER)

    # Tai trong trái
    draw.pieslice([cx - W * 0.38, cy - H * 0.39, cx - W * 0.12, cy - H * 0.05], 140, 300, fill=PIG_EAR_INNER)
    # Tai trong phải
    draw.pieslice([cx + W * 0.12, cy - H * 0.39, cx + W * 0.38, cy - H * 0.05], 240, 40, fill=PIG_EAR_INNER)

    # 2. KHUÔN MẶT BẦU BĨNH (Mặt tròn hơi nở nhẹ ở phần má)
    face_r_x = W * 0.43
    face_r_y = H * 0.39
    face_cy = cy + H * 0.04
    # Bóng đổ nhẹ bên dưới mặt
    draw.ellipse([cx - face_r_x, face_cy - face_r_y + H * 0.02, cx + face_r_x, face_cy + face_r_y + H * 0.02], fill=PIG_BODY_SHADOW)
    # Mặt chính
    draw.ellipse([cx - face_r_x, face_cy - face_r_y, cx + face_r_x, face_cy + face_r_y], fill=PIG_BODY)

    # 3. MÁ HỒNG TỎA (Blushing cheeks)
    cheek_rx = W * 0.11
    cheek_ry = H * 0.075
    cheek_y = face_cy + H * 0.09
    draw.ellipse([cx - W * 0.32 - cheek_rx, cheek_y - cheek_ry, cx - W * 0.32 + cheek_rx, cheek_y + cheek_ry], fill=PIG_CHEEK)
    draw.ellipse([cx + W * 0.32 - cheek_rx, cheek_y - cheek_ry, cx + W * 0.32 + cheek_rx, cheek_y + cheek_ry], fill=PIG_CHEEK)

    # Vạch má e thẹn (blush lines)
    blush_pen = (255, 105, 135, 180)
    line_w = int(W * 0.01)
    for offset in [-cheek_rx * 0.4, 0, cheek_rx * 0.4]:
        draw.line([cx - W * 0.32 + offset - line_w, cheek_y - cheek_ry * 0.4, cx - W * 0.32 + offset + line_w, cheek_y + cheek_ry * 0.4], fill=blush_pen, width=line_w)
        draw.line([cx + W * 0.32 + offset - line_w, cheek_y - cheek_ry * 0.4, cx + W * 0.32 + offset + line_w, cheek_y + cheek_ry * 0.4], fill=blush_pen, width=line_w)

    # 4. ĐÔI MẮT LONG LANH ANIME (Cực kỳ dễ thương)
    eye_rx = W * 0.065
    eye_ry = H * 0.082
    eye_y = face_cy - H * 0.04
    eye_lx = cx - W * 0.20
    eye_rx_pos = cx + W * 0.20

    # Vẽ tròng mắt to tròn đen nâu ấm
    draw.ellipse([eye_lx - eye_rx, eye_y - eye_ry, eye_lx + eye_rx, eye_y + eye_ry], fill=PIG_EYE_DARK)
    draw.ellipse([eye_rx_pos - eye_rx, eye_y - eye_ry, eye_rx_pos + eye_rx, eye_y + eye_ry], fill=PIG_EYE_DARK)

    # Đốm sáng lớn long lanh ở góc trên bên trái
    s1_r = eye_rx * 0.52
    draw.ellipse([eye_lx - eye_rx * 0.45 - s1_r, eye_y - eye_ry * 0.5 - s1_r, eye_lx - eye_rx * 0.45 + s1_r, eye_y - eye_ry * 0.5 + s1_r], fill=WHITE)
    draw.ellipse([eye_rx_pos - eye_rx * 0.45 - s1_r, eye_y - eye_ry * 0.5 - s1_r, eye_rx_pos - eye_rx * 0.45 + s1_r, eye_y - eye_ry * 0.5 + s1_r], fill=WHITE)

    # Đốm sáng nhỏ lấp lánh ở góc dưới bên phải
    s2_r = eye_rx * 0.26
    draw.ellipse([eye_lx + eye_rx * 0.35 - s2_r, eye_y + eye_ry * 0.35 - s2_r, eye_lx + eye_rx * 0.35 + s2_r, eye_y + eye_ry * 0.35 + s2_r], fill=WHITE)
    draw.ellipse([eye_rx_pos + eye_rx * 0.35 - s2_r, eye_y + eye_ry * 0.35 - s2_r, eye_rx_pos + eye_rx * 0.35 + s2_r, eye_y + eye_ry * 0.35 + s2_r], fill=WHITE)

    # 5. CHIẾC MŨI HEO ĐÁNG YÊU (To vừa vặn, hơi dẹt ngang)
    snout_w = W * 0.17
    snout_h = H * 0.125
    snout_y = face_cy + H * 0.06
    # Bóng mũi
    draw.ellipse([cx - snout_w, snout_y - snout_h + H * 0.015, cx + snout_w, snout_y + snout_h + H * 0.015], fill=PIG_BODY_SHADOW)
    # Mũi chính
    draw.ellipse([cx - snout_w, snout_y - snout_h, cx + snout_w, snout_y + snout_h], fill=PIG_SNOUT)
    # Vệt highlight nhẹ trên sống mũi
    draw.ellipse([cx - snout_w * 0.65, snout_y - snout_h * 0.75, cx + snout_w * 0.65, snout_y - snout_h * 0.2], fill=PIG_SNOUT_LIGHT)

    # Hai lỗ mũi hình hạt đậu tròn bầu bĩnh
    nos_rx = snout_w * 0.22
    nos_ry = snout_h * 0.36
    nos_y = snout_y + H * 0.005
    draw.ellipse([cx - snout_w * 0.42 - nos_rx, nos_y - nos_ry, cx - snout_w * 0.42 + nos_rx, nos_y + nos_ry], fill=NOSTRIL_DARK)
    draw.ellipse([cx + snout_w * 0.42 - nos_rx, nos_y - nos_ry, cx + snout_w * 0.42 + nos_rx, nos_y + nos_ry], fill=NOSTRIL_DARK)

    # 6. MIỆNG CƯỜI NHẸ XINH XẮN (ngay dưới mũi)
    mouth_y = snout_y + snout_h + H * 0.01
    draw.arc([cx - W * 0.06, mouth_y - H * 0.03, cx + W * 0.06, mouth_y + H * 0.03], 30, 150, fill=(180, 80, 105, 255), width=int(W * 0.012))

    # Downsample với bộ lọc Lanczos chất lượng cao nhất để chống răng cưa hoàn hảo (Super-Sampling Anti-Aliasing)
    res = img.resize((size, size), Image.Resampling.LANCZOS)
    return res

def generate_multi_resolution_ico():
    sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico_frames = []

    for w, h in sizes:
        img_frame = create_super_cute_piggy(size=w)
        ico_frames.append(img_frame)

    # Lưu burin.ico tại cả root và core_engine
    paths = [
        "D:/Project/smart-vietnamese-ime/burin.ico",
        "D:/Project/smart-vietnamese-ime/core_engine/burin.ico"
    ]
    for p in paths:
        ico_frames[-1].save(p, format="ICO", sizes=[(f.width, f.height) for f in ico_frames], append_images=ico_frames[:-1])
        print(f"Generated high quality cute icon: {p}")

if __name__ == "__main__":
    generate_multi_resolution_ico()
