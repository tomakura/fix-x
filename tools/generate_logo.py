from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
SOURCE_PATH = ASSETS / "103d668e-2545-49b6-bdfa-2708d540447e.jpg"
PNG_PATH = ASSETS / "logo.png"
ICO_PATH = ASSETS / "logo.ico"


def main() -> None:
    if not SOURCE_PATH.exists():
        raise FileNotFoundError(f"Logo source not found: {SOURCE_PATH}")

    image = Image.open(SOURCE_PATH).convert("RGBA")
    image.save(PNG_PATH)
    image.save(
        ICO_PATH,
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )


if __name__ == "__main__":
    main()
