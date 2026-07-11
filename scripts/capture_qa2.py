"""Capture real app windows for Plan / Help / Dashboard using BM9000_START_VIEW."""
from __future__ import annotations

import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(r"c:\temp\GrokBuild\BM9000")
PORTABLE = ROOT / "release" / "portable"
EXE = PORTABLE / "BudgetMaster9000.exe"
SHOTS = ROOT / "release" / "qa-screenshots"
sys.path.insert(0, str(ROOT / "scripts"))
from screenshot_window import capture, find_window  # noqa: E402


def kill() -> None:
    subprocess.run(["taskkill", "/F", "/IM", "BudgetMaster9000.exe"], capture_output=True)
    time.sleep(0.4)


def wait_bm(timeout=25.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        for hwnd, name in find_window("Budget Master 9000"):
            # ignore other tools that mention the title in session chrome
            if name.strip() == "Budget Master 9000" or name.startswith("Budget Master 9000"):
                if "grok" in name.lower() or "Install UIA" in name:
                    continue
                return hwnd, name
        # also exact-ish
        wins = find_window("Budget Master 9000")
        for hwnd, name in wins:
            if "Single-Executable" in name or "grok" in name.lower():
                continue
            if "Budget Master 9000" in name and len(name) < 40:
                return hwnd, name
        time.sleep(0.3)
    raise RuntimeError("BM9000 window not found")


def run_view(view: str, shot: str) -> None:
    kill()
    env = dict(**{k: v for k, v in __import__("os").environ.items()})
    env["BM9000_START_VIEW"] = view
    env.pop("BM9000_UI_SELFTEST", None)
    proc = subprocess.Popen([str(EXE)], cwd=str(PORTABLE), env=env)
    try:
        time.sleep(3.0)
        hwnd, name = wait_bm()
        print("view", view, "window", repr(name))
        time.sleep(1.0)
        capture(hwnd, SHOTS / shot)
    finally:
        proc.terminate()
        kill()


def main() -> None:
    SHOTS.mkdir(parents=True, exist_ok=True)
    src = ROOT / "src-tauri" / "target" / "release" / "budget-master-9000.exe"
    shutil.copy2(src, EXE)
    # seed
    for p in PORTABLE.glob("bm9000.db*"):
        p.unlink(missing_ok=True)
    (PORTABLE / "bm9000.portable").write_text("portable\n", encoding="utf-8")
    subprocess.run(
        [str(EXE), "--seed-demo", str(PORTABLE / "bm9000.db")],
        cwd=str(PORTABLE),
        check=False,
    )

    # verify JS bundle
    js = next((ROOT / "dist" / "assets").glob("index-*.js"))
    text = js.read_text(encoding="utf-8", errors="replace")
    issues = []
    for bad in ["Iâ", "â€™", "Â·", "âœ", "Ã—", "Ã"]:
        if bad in text:
            issues.append(f"mojibake {bad}")
    if "ll do this later" not in text:
        issues.append("missing I'll do this later")
    if ">X</button>" not in text:
        issues.append("missing X delete button")
    if "Each category is either:" not in text and "Each category is either" not in text:
        # may be in help.ts bundled
        if "categories-fixed" not in text:
            issues.append("help content missing")

    for view, shot in [
        ("dashboard", "01-dashboard.png"),
        ("plan", "02-plan.png"),
        ("help", "03-help.png"),
        ("settings", "04-settings.png"),
    ]:
        run_view(view, shot)

    # OCR-free visual: open images and check not mostly black
    from PIL import Image, ImageStat

    for shot in sorted(SHOTS.glob("*.png")):
        im = Image.open(shot).convert("L")
        stat = ImageStat.Stat(im)
        mean = stat.mean[0]
        print(shot.name, im.size, "mean", round(mean, 1))
        if mean < 5:
            issues.append(f"{shot.name} too dark")

    if issues:
        print("ISSUES", issues)
        raise SystemExit(1)
    print("QA OK")


if __name__ == "__main__":
    main()
