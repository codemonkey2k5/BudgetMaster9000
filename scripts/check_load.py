"""Verify the portable exe serves the UI and backend self-test passes."""
from __future__ import annotations

import shutil
import subprocess
import time
from pathlib import Path
from urllib.request import urlopen

ROOT = Path(r"c:\temp\GrokBuild\BM9000")
PORTABLE = ROOT / "release" / "portable"
EXE = PORTABLE / "BudgetMaster9000.exe"
SRC = ROOT / "src-tauri" / "target" / "release" / "budget-master-9000.exe"


def kill() -> None:
    subprocess.run(["taskkill", "/F", "/IM", "BudgetMaster9000.exe"], capture_output=True)
    subprocess.run(["taskkill", "/F", "/IM", "budget-master-9000.exe"], capture_output=True)
    time.sleep(0.5)


def main() -> None:
    kill()
    shutil.copy2(SRC, EXE)
    # Backend self-test
    r = subprocess.run([str(EXE), "--self-test"], cwd=str(PORTABLE), capture_output=True)
    result = (PORTABLE / "cli-result.txt").read_text(encoding="utf-8", errors="replace")
    print(result.strip())
    if "SELFTEST PASS" not in result:
        raise SystemExit("self-test failed")

    # UI load
    proc = subprocess.Popen([str(EXE)], cwd=str(PORTABLE))
    try:
        port = None
        for _ in range(40):
            time.sleep(0.25)
            # find listen port via netstat
            out = subprocess.check_output(
                f'netstat -ano | findstr {proc.pid}',
                shell=True,
                text=True,
                errors="replace",
            )
            for line in out.splitlines():
                if "LISTENING" in line and ("127.0.0.1" in line or "[::1]" in line or "0.0.0.0" in line):
                    # TCP    [::1]:12345
                    parts = line.split()
                    for p in parts:
                        if ":" in p:
                            try:
                                port = int(p.rsplit(":", 1)[-1])
                            except ValueError:
                                pass
            if port:
                break
        if not port:
            raise SystemExit(f"no listen port for pid {proc.pid}")
        print("port", port)
        with urlopen(f"http://localhost:{port}/", timeout=5) as resp:
            body = resp.read().decode("utf-8", errors="replace")
            code = resp.status
        print("HTTP", code, "len", len(body))
        if code != 200:
            raise SystemExit(f"HTTP {code}")
        if "Budget Master" not in body and "app" not in body:
            raise SystemExit("unexpected HTML")
        # JS asset
        import re

        m = re.search(r'src="([^"]+\.js)"', body)
        if m:
            path = m.group(1)
            with urlopen(f"http://localhost:{port}{path}", timeout=5) as resp:
                print("JS", resp.status, "len", len(resp.read()))
                if resp.status != 200:
                    raise SystemExit("JS not 200")
        print("LOAD OK")
    finally:
        proc.terminate()
        kill()


if __name__ == "__main__":
    main()
