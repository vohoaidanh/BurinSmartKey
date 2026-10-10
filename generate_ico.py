from PIL import Image, ImageDraw

def create_piggy_ico(out_path):
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    images = []

    for w, h in sizes:
        img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)

        # Scale coordinates from 32x32
        def sx(x): return x * w / 32.0
        def sy(y): return y * h / 32.0

        # Colors (match the app)
        pig_pink = (255, 166, 181, 255) # #FFA6B5
        ear_pink = (255, 112, 142, 255) # #FF708E
        inner_ear = (255, 192, 208, 255)
        cheek = (255, 128, 144, 255)
        snout = (255, 96, 120, 255)
        dark = (26, 26, 26, 255)
        white = (255, 255, 255, 255)

        # Ears
        draw.ellipse([sx(2), sy(1), sx(13), sy(13)], fill=ear_pink)
        draw.ellipse([sx(19), sy(1), sx(30), sy(13)], fill=ear_pink)

        # Inner ears
        draw.ellipse([sx(4), sy(3), sx(11), sy(11)], fill=inner_ear)
        draw.ellipse([sx(21), sy(3), sx(28), sy(11)], fill=inner_ear)

        # Face
        draw.ellipse([sx(1), sy(5), sx(31), sy(31)], fill=pig_pink)

        # Cheeks
        draw.ellipse([sx(3), sy(17), sx(8), sy(22)], fill=cheek)
        draw.ellipse([sx(24), sy(17), sy(29), sy(22)], fill=cheek)

        # Eyes
        draw.ellipse([sx(7), sy(12), sx(11), sy(17)], fill=dark)
        draw.ellipse([sx(21), sy(12), sx(25), sy(17)], fill=dark)

        # Eye shines
        draw.ellipse([sx(8), sy(13), sx(10), sy(15)], fill=white)
        draw.ellipse([sx(22), sy(13), sx(24), sy(15)], fill=white)

        # Snout
        draw.ellipse([sx(10), sy(16), sx(22), sy(25)], fill=snout)

        # Nostrils
        draw.ellipse([sx(13), sy(19), sx(15), sy(22)], fill=dark)
        draw.ellipse([sx(17), sy(19), sx(19), sy(22)], fill=dark)

        images.append(img)

    images[0].save(out_path, format="ICO", sizes=sizes, append_images=images[1:])
    print(f"Created {out_path} successfully!")

if __name__ == "__main__":
    create_piggy_ico("D:/Project/smart-vietnamese-ime/burin.ico")
    create_piggy_ico("D:/Project/smart-vietnamese-ime/core_engine/burin.ico")
