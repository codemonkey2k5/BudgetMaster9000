"""Capture a full named window to PNG (DPI-aware, full client area)."""
from __future__ import annotations

import ctypes
import sys
import time
from ctypes import wintypes
from pathlib import Path

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32
try:
    shcore = ctypes.windll.shcore
except Exception:  # pragma: no cover
    shcore = None

PW_RENDERFULLCONTENT = 2
SW_MAXIMIZE = 3
SW_RESTORE = 9
MONITOR_DEFAULTTONEAREST = 2

# Per-monitor DPI aware v2
DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = ctypes.c_void_p(-4)


class RECT(ctypes.Structure):
    _fields_ = [
        ("left", ctypes.c_long),
        ("top", ctypes.c_long),
        ("right", ctypes.c_long),
        ("bottom", ctypes.c_long),
    ]


def enable_dpi_awareness() -> None:
    """Must run before measuring/capturing windows on high-DPI displays."""
    try:
        user32.SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
        return
    except Exception:
        pass
    if shcore is not None:
        try:
            # 2 = PROCESS_PER_MONITOR_DPI_AWARE
            shcore.SetProcessDpiAwareness(2)
            return
        except Exception:
            pass
    try:
        user32.SetProcessDPIAware()
    except Exception:
        pass


enable_dpi_awareness()


def find_window(title_substr: str):
    result = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
    def enum_proc(hwnd, _lparam):
        if not user32.IsWindowVisible(hwnd):
            return True
        length = user32.GetWindowTextLengthW(hwnd)
        if length == 0:
            return True
        buf = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buf, length + 1)
        if title_substr.lower() in buf.value.lower():
            result.append((hwnd, buf.value))
        return True

    user32.EnumWindows(enum_proc, 0)
    return result


def maximize(hwnd) -> None:
    user32.ShowWindow(hwnd, SW_MAXIMIZE)
    time.sleep(0.35)


def window_rect_physical(hwnd) -> tuple[int, int, int, int]:
    """Return left, top, width, height in physical pixels."""
    rect = RECT()
    # Prefer DWM extended frame bounds (true visible bounds)
    try:
        dwmapi = ctypes.windll.dwmapi
        DWMWA_EXTENDED_FRAME_BOUNDS = 9
        hr = dwmapi.DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            ctypes.byref(rect),
            ctypes.sizeof(rect),
        )
        if hr == 0:
            w = rect.right - rect.left
            h = rect.bottom - rect.top
            if w > 0 and h > 0:
                return rect.left, rect.top, w, h
    except Exception:
        pass

    user32.GetWindowRect(hwnd, ctypes.byref(rect))
    return rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top


def capture(hwnd, out: Path, *, maximize_window: bool = True) -> None:
    """Capture full window bitmap to PNG."""
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.2)
    if maximize_window:
        maximize(hwnd)

    left, top, w, h = window_rect_physical(hwnd)
    if w <= 10 or h <= 10:
        raise RuntimeError(f"invalid window size {w}x{h}")

    # Method 1: PrintWindow into a bitmap matching physical size
    hwnd_dc = user32.GetWindowDC(hwnd)
    mem_dc = gdi32.CreateCompatibleDC(hwnd_dc)
    bmp = gdi32.CreateCompatibleBitmap(hwnd_dc, w, h)
    old = gdi32.SelectObject(mem_dc, bmp)

    ok = user32.PrintWindow(hwnd, mem_dc, PW_RENDERFULLCONTENT)
    if not ok:
        ok = user32.PrintWindow(hwnd, mem_dc, 0)

    class BITMAPINFOHEADER(ctypes.Structure):
        _fields_ = [
            ("biSize", wintypes.DWORD),
            ("biWidth", ctypes.c_long),
            ("biHeight", ctypes.c_long),
            ("biPlanes", wintypes.WORD),
            ("biBitCount", wintypes.WORD),
            ("biCompression", wintypes.DWORD),
            ("biSizeImage", wintypes.DWORD),
            ("biXPelsPerMeter", ctypes.c_long),
            ("biYPelsPerMeter", ctypes.c_long),
            ("biClrUsed", wintypes.DWORD),
            ("biClrImportant", wintypes.DWORD),
        ]

    class BITMAPINFO(ctypes.Structure):
        _fields_ = [("bmiHeader", BITMAPINFOHEADER), ("bmiColors", wintypes.DWORD * 3)]

    def bits_from_dc(src_dc, width: int, height: int, bitmap):
        bmi = BITMAPINFO()
        bmi.bmiHeader.biSize = ctypes.sizeof(BITMAPINFOHEADER)
        bmi.bmiHeader.biWidth = width
        bmi.bmiHeader.biHeight = -height
        bmi.bmiHeader.biPlanes = 1
        bmi.bmiHeader.biBitCount = 32
        bmi.bmiHeader.biCompression = 0
        buf_len = width * height * 4
        buf = (ctypes.c_ubyte * buf_len)()
        gdi32.GetDIBits(src_dc, bitmap, 0, height, buf, ctypes.byref(bmi), 0)
        return bytes(buf)

    buf = bits_from_dc(mem_dc, w, h, bmp)

    # Detect "only top-left painted" failure: large black/empty region
    from PIL import Image

    img = Image.frombuffer("RGBA", (w, h), buf, "raw", "BGRA", 0, 1)

    def content_ratio(im: Image.Image) -> float:
        # Non-near-black pixel ratio in bottom-right quadrant
        w0, h0 = im.size
        crop = im.crop((w0 // 2, h0 // 2, w0 - 2, h0 - 2)).convert("RGB")
        pix = list(crop.getdata())
        if not pix:
            return 0.0
        alive = sum(1 for r, g, b in pix if r + g + b > 30)
        return alive / len(pix)

    ratio = content_ratio(img)
    if ratio < 0.15:
        # Fallback: BitBlt from the screen (works reliably for maximized windows)
        print(f"PrintWindow sparse (ratio={ratio:.2f}), falling back to screen BitBlt")
        screen_dc = user32.GetDC(0)
        mem2 = gdi32.CreateCompatibleDC(screen_dc)
        bmp2 = gdi32.CreateCompatibleBitmap(screen_dc, w, h)
        gdi32.SelectObject(mem2, bmp2)
        # SRCCOPY
        gdi32.BitBlt(mem2, 0, 0, w, h, screen_dc, left, top, 0x00CC0020)
        buf = bits_from_dc(mem2, w, h, bmp2)
        img = Image.frombuffer("RGBA", (w, h), buf, "raw", "BGRA", 0, 1)
        gdi32.DeleteObject(bmp2)
        gdi32.DeleteDC(mem2)
        user32.ReleaseDC(0, screen_dc)

    gdi32.SelectObject(mem_dc, old)
    gdi32.DeleteObject(bmp)
    gdi32.DeleteDC(mem_dc)
    user32.ReleaseDC(hwnd, hwnd_dc)

    # Convert to RGB for smaller, cleaner PNGs
    rgb = img.convert("RGB")
    out.parent.mkdir(parents=True, exist_ok=True)
    rgb.save(out, optimize=True)
    print(f"saved {out} ({rgb.size[0]}x{rgb.size[1]}) content_ratio~{content_ratio(img):.2f}")


def main() -> None:
    title = sys.argv[1] if len(sys.argv) > 1 else "Budget Master"
    out = Path(sys.argv[2] if len(sys.argv) > 2 else "screenshot.png")
    wins = find_window(title)
    if not wins:
        raise SystemExit(f"no window matching {title!r}")
    hwnd, name = wins[0]
    print("capturing", name)
    capture(hwnd, out)


if __name__ == "__main__":
    main()
