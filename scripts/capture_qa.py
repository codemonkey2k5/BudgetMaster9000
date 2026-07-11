"""Launch BM9000, navigate views, capture screenshots, OCR-ish text checks via PIL."""
from __future__ import annotations

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


def kill_app() -> None:
    subprocess.run(
        ["taskkill", "/F", "/IM", "BudgetMaster9000.exe"],
        capture_output=True,
    )
    subprocess.run(
        ["taskkill", "/F", "/IM", "budget-master-9000.exe"],
        capture_output=True,
    )
    time.sleep(0.5)


def seed() -> None:
    db = PORTABLE / "bm9000.db"
    for p in PORTABLE.glob("bm9000.db*"):
        p.unlink(missing_ok=True)
    (PORTABLE / "bm9000.portable").write_text("portable\n", encoding="utf-8")
    r = subprocess.run(
        [str(EXE), "--seed-demo", str(db)],
        cwd=str(PORTABLE),
        capture_output=True,
        text=True,
    )
    print("seed", r.returncode, (PORTABLE / "cli-result.txt").read_text(encoding="utf-8", errors="replace"))


def wait_window(timeout=20.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        wins = find_window("Budget Master")
        if wins:
            return wins[0]
        time.sleep(0.3)
    raise RuntimeError("window not found")


def click_uia(name: str) -> bool:
    """Click a button/control by Name via UI Automation."""
    try:
        import comtypes  # type: ignore
        from comtypes.client import CreateObject  # type: ignore
    except Exception:
        # fallback: use pywinauto if present
        pass

    try:
        import uiautomation as auto  # type: ignore

        win = auto.WindowControl(searchDepth=1, Name="Budget Master 9000")
        if not win.Exists(3):
            # partial
            win = auto.WindowControl(searchDepth=1, SubName="Budget Master")
        ctrl = win.ButtonControl(Name=name, searchDepth=12)
        if ctrl.Exists(2):
            ctrl.Click()
            return True
        # try text/hyperlink
        for typ in (auto.ButtonControl, auto.TextControl, auto.HyperlinkControl, auto.ListItemControl):
            c = typ(searchFromControl=win, Name=name, searchDepth=14)
            if c.Exists(0.5):
                c.Click()
                return True
        return False
    except Exception as e:
        print("uiautomation failed", e)
        return False


def main() -> None:
    SHOTS.mkdir(parents=True, exist_ok=True)
    kill_app()
    # deploy latest exe
    src = ROOT / "src-tauri" / "target" / "release" / "budget-master-9000.exe"
    if src.exists():
        import shutil

        shutil.copy2(src, EXE)
    seed()
    proc = subprocess.Popen([str(EXE)], cwd=str(PORTABLE))
    try:
        hwnd, title = wait_window()
        print("window", title)
        time.sleep(2.5)
        capture(hwnd, SHOTS / "01-dashboard.png")

        # Navigate via UI Automation if available
        for label, fname in [
            ("Plan", "02-plan.png"),
            ("Help", "03-help.png"),
            ("Settings", "04-settings.png"),
        ]:
            ok = click_uia(label)
            print("click", label, ok)
            time.sleep(1.2)
            wins = find_window("Budget Master")
            if wins:
                capture(wins[0][0], SHOTS / fname)

        # Simple pixel-based glitch detector: high ratio of rare colored pixels?
        from PIL import Image

        issues = []
        for shot in sorted(SHOTS.glob("*.png")):
            im = Image.open(shot).convert("RGB")
            # sample center crop text-ish region for unexpected high-contrast noise is hard;
            # instead verify file size and dimensions
            w, h = im.size
            if w < 400 or h < 300:
                issues.append(f"{shot.name} too small {w}x{h}")
            print(shot.name, w, h)

        # Text rendered JS bundle still free of mojibake markers
        dist_js = list((ROOT / "dist" / "assets").glob("index-*.js"))
        if dist_js:
            js = dist_js[0].read_text(encoding="utf-8", errors="replace")
            for bad in ["Iâ", "â€™", "Â·", "âœ", "Ã"]:
                if bad in js:
                    issues.append(f"mojibake in bundle: {bad}")
            for good in ["I'll do this later", 'data-delcat="${c.id}">X</button>', "Categories: Fixed vs Flexible"]:
                # templates may be split - check parts
                pass
            if "I'll do this later" not in js and "I\\'ll do this later" not in js:
                # might be escaped differently
                if "ll do this later" not in js:
                    issues.append("missing I'll do this later in bundle")
            if ">X</button>" not in js and "\">X</" not in js:
                issues.append("missing X delete button in bundle")

        if issues:
            print("ISSUES:")
            for i in issues:
                print(" -", i)
            raise SystemExit(1)
        print("QA CAPTURE OK")
    finally:
        kill_app()


if __name__ == "__main__":
    main()
