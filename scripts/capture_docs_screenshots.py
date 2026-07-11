"""Capture polished screenshots into docs/screenshots for README / GitHub."""
from __future__ import annotations

import os
import shutil
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(r"c:\temp\GrokBuild\BM9000")
PORTABLE = ROOT / "release" / "portable"
EXE = PORTABLE / "BudgetMaster9000.exe"
SRC = ROOT / "src-tauri" / "target" / "release" / "budget-master-9000.exe"
SHOTS = ROOT / "docs" / "screenshots"
sys.path.insert(0, str(ROOT / "scripts"))
from screenshot_window import capture, find_window  # noqa: E402


def kill() -> None:
    subprocess.run(["taskkill", "/F", "/IM", "BudgetMaster9000.exe"], capture_output=True)
    time.sleep(0.5)


def wait_bm(timeout=25.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        for hwnd, name in find_window("Budget Master 9000"):
            if "grok" in name.lower() or "Single-Executable" in name:
                continue
            if "Budget Master 9000" in name and len(name) < 48:
                return hwnd, name
        time.sleep(0.25)
    raise RuntimeError("Budget Master 9000 window not found")


def prepare_demo() -> None:
    SHOTS.mkdir(parents=True, exist_ok=True)
    if SRC.exists():
        shutil.copy2(SRC, EXE)
    for p in PORTABLE.glob("bm9000.db*"):
        p.unlink(missing_ok=True)
    (PORTABLE / "bm9000.portable").write_text("portable\n", encoding="utf-8")
    subprocess.run(
        [str(EXE), "--seed-demo", str(PORTABLE / "bm9000.db")],
        cwd=str(PORTABLE),
        check=False,
    )
    db = PORTABLE / "bm9000.db"
    conn = sqlite3.connect(db)
    conn.execute(
        "INSERT INTO settings(key,value) VALUES('needs_category_review','0') "
        "ON CONFLICT(key) DO UPDATE SET value=excluded.value"
    )
    # Give demo some actuals / fixed fill by opening dashboard once via SQL path:
    # fixed actuals apply at runtime; leave as-is
    conn.commit()
    conn.close()


def run_view(view: str, filename: str, settle: float = 2.0) -> None:
    kill()
    env = os.environ.copy()
    env["BM9000_START_VIEW"] = view
    env.pop("BM9000_UI_SELFTEST", None)
    proc = subprocess.Popen([str(EXE)], cwd=str(PORTABLE), env=env)
    try:
        time.sleep(3.5)
        hwnd, name = wait_bm()
        print("captured", view, "as", name)
        # Maximize + extra settle so WebView finishes layout at full size
        time.sleep(settle)
        out = SHOTS / filename
        capture(hwnd, out, maximize_window=True)
        # Verify we did not save a mostly-empty crop
        from PIL import Image

        im = Image.open(out).convert("RGB")
        w, h = im.size
        if w < 1000 or h < 700:
            print(f"WARNING: small capture {w}x{h} for {filename}")
        # sample bottom-right: should not be pure black on a full UI shot
        br = im.getpixel((w - 40, h - 40))
        print(f"  size={w}x{h} bottom-right pixel={br}")
        mirror = ROOT / "docs" / "assets" / "screenshots"
        mirror.mkdir(parents=True, exist_ok=True)
        shutil.copy2(out, mirror / filename)
        print("saved", out)
    finally:
        proc.terminate()
        kill()


def main() -> None:
    prepare_demo()
    shots = [
        ("dashboard", "01-dashboard.png"),
        ("plan", "02-plan.png"),
        ("checkin", "03-checkin.png"),
        ("history", "04-history.png"),
        ("help", "05-help.png"),
        ("settings", "06-settings.png"),
    ]
    for view, name in shots:
        run_view(view, name)
    # Icon for README branding
    icon_src = ROOT / "icon.png"
    if icon_src.exists():
        shutil.copy2(icon_src, SHOTS / "icon.png")
        shutil.copy2(icon_src, ROOT / "docs" / "assets" / "icon.png")
    print("DONE", SHOTS)


if __name__ == "__main__":
    main()
