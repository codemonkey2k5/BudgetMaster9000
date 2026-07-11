from PIL import Image
from pathlib import Path

src = Path(
    r"C:\Users\tony\.grok\sessions\C%3A%5CUsers%5Ctony\019f4d0c-070a-7c40-8689-10f99ce6462a\images\1.jpg"
)
root = Path(r"c:\temp\GrokBuild\BM9000")
icons_dir = root / "src-tauri" / "icons"
icons_dir.mkdir(parents=True, exist_ok=True)
(root / "public").mkdir(parents=True, exist_ok=True)

im = Image.open(src).convert("RGBA")
w, h = im.size
side = min(w, h)
left = (w - side) // 2
top = (h - side) // 2
im = im.crop((left, top, left + side, top + side))


def save_png(size: int, name: str) -> None:
    out = im.resize((size, size), Image.Resampling.LANCZOS)
    path = icons_dir / name
    out.save(path, "PNG")
    print("wrote", path, size)


sizes = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}
for name, s in sizes.items():
    save_png(s, name)

ico_sizes = [16, 24, 32, 48, 64, 128, 256]
ico_images = [im.resize((s, s), Image.Resampling.LANCZOS) for s in ico_sizes]
ico_path = icons_dir / "icon.ico"
ico_images[0].save(
    ico_path,
    format="ICO",
    sizes=[(s, s) for s in ico_sizes],
    append_images=ico_images[1:],
)
print("wrote", ico_path)

im.resize((256, 256), Image.Resampling.LANCZOS).save(root / "icon.png", "PNG")
im.resize((256, 256), Image.Resampling.LANCZOS).save(root / "public" / "icon.png", "PNG")
ico_images[0].save(
    root / "icon.ico",
    format="ICO",
    sizes=[(s, s) for s in ico_sizes],
    append_images=ico_images[1:],
)
print("root icons updated")
