"""Jersey number, chest wordmark and cap letter as transparent PNGs (fictional team).
usage: python3 -I decals.py OUTDIR"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
SERIF = "/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf"
NAVY = (30, 44, 90, 255)
RED = (165, 28, 38, 255)
WHITE = (245, 245, 240, 255)


def decal(text, size, fill, outline, stroke, path, pad=0.04, condense=1.0):
    f = ImageFont.truetype(SERIF, size)
    tmp = Image.new("RGBA", (10, 10))
    d = ImageDraw.Draw(tmp)
    x0, y0, x1, y1 = d.textbbox((0, 0), text, font=f, stroke_width=stroke)
    w, h = x1 - x0, y1 - y0
    img = Image.new("RGBA", (int(w * (1 + 2 * pad)), int(h * (1 + 2 * pad))), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.text((int(w * pad) - x0, int(h * pad) - y0), text, font=f, fill=fill, stroke_width=stroke, stroke_fill=outline)
    if condense != 1.0:
        img = img.resize((int(img.width * condense), img.height), Image.LANCZOS)
    img.save(path)
    print(path, img.size, "aspect w/h", round(img.width / img.height, 3))


decal("17", 600, NAVY, RED, 14, os.path.join(OUT, "number.png"), condense=0.82)
decal("HANGANG", 300, NAVY, RED, 7, os.path.join(OUT, "wordmark.png"), condense=0.9)
decal("H", 600, WHITE, WHITE, 0, os.path.join(OUT, "caplogo.png"))
