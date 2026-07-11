"""Rewrite main.ts as clean ASCII-only source (fix PowerShell mojibake)."""
from __future__ import annotations

import re
from pathlib import Path

SRC = Path(r"c:\temp\GrokBuild\BM9000\src\main.ts")


def to_ascii(text: str) -> str:
    # Common mojibake (UTF-8 misread as Latin-1 / Windows-1252)
    pairs = [
        ("I\u00e2\u20ac\u2122ll", "I'll"),
        ("you\u00e2\u20ac\u2122ll", "you'll"),
        ("you\u00e2\u20ac\u2122re", "you're"),
        ("month\u00e2\u20ac\u2122s", "month's"),
        ("\u00e2\u20ac\u2122", "'"),
        ("\u00e2\u20ac\u0153", '"'),
        ("\u00e2\u20ac\u009d", '"'),
        ("\u00e2\u20ac\u009c", '"'),
        ("\u00c2\u00b7", " - "),
        ("\u00e2\u2020\u2019", "->"),
        ("\u00e2\u2020\u0090", "<-"),
        ("\u00e2\u0153\u201c", "*"),
        ("\u00e2\u0153\u2022", "X"),
        ("\u00e2\u20ac\u00a6", "..."),
        ("\u00e2\u20ac\u201c", "-"),
        ("\u00e2\u20ac\u201d", "-"),
    ]
    for a, b in pairs:
        text = text.replace(a, b)

    # Proper Unicode (if still present)
    uni = {
        "\u2019": "'",
        "\u2018": "'",
        "\u201c": '"',
        "\u201d": '"',
        "\u2014": "-",
        "\u2013": "-",
        "\u2026": "...",
        "\u00b7": " - ",
        "\u2022": "*",
        "\u2192": "->",
        "\u2190": "<-",
        "\u2713": "*",
        "\u2715": "X",
        "\u2717": "X",
        "\u00d7": "X",
        "\u25c9": "o",
        "\u25f7": "H",
        "\u2630": "=",
        "\u2699": "S",
        "\u00a0": " ",
        "\ufffd": "",
    }
    for a, b in uni.items():
        text = text.replace(a, b)

    # Section comments
    text = re.sub(r"/\*[^\n]*DASHBOARD[^\n]*\*/", "/* DASHBOARD */", text)
    text = re.sub(r"/\*[^\n]*CHECK-IN[^\n]*\*/", "/* CHECK-IN */", text)
    text = re.sub(r"/\*[^\n]*PLAN[^\n]*\*/", "/* PLAN */", text)
    text = re.sub(r"/\*[^\n]*HISTORY[^\n]*\*/", "/* HISTORY */", text)
    text = re.sub(r"/\*[^\n]*HELP[^\n]*\*/", "/* HELP */", text)
    text = re.sub(r"/\*[^\n]*SETTINGS[^\n]*\*/", "/* SETTINGS */", text)
    text = re.sub(r"/\*[^\n]*self-test[^\n]*\*/", "/* UI self-test */", text)
    text = re.sub(r"/\*[^\n]*UI self-test[^\n]*\*/", "/* UI self-test */", text)

    # Drop any remaining non-ASCII
    text = "".join(ch if ord(ch) < 128 else "" for ch in text)
    return text


def fix_clear_flow(text: str) -> str:
    """After clear data: stay on onboarding; render() must honor !hasData."""
    # Fix render() to redirect to onboard when no data
    old_render = """function render() {
 if (!state.status || state.status.locked) {
 renderLock();
 return;
 }
 let content = "";"""
    new_render = """function render() {
 if (!state.status || state.status.locked) {
 renderLock();
 return;
 }
 if (!state.status.hasData) {
 renderOnboard();
 return;
 }
 let content = "";"""
    if old_render in text:
        text = text.replace(old_render, new_render)
    else:
        # try looser
        text = text.replace(
            """if (!state.status || state.status.locked) {
 renderLock();
 return;
 }
 let content = "";""",
            """if (!state.status || state.status.locked) {
 renderLock();
 return;
 }
 if (!state.status.hasData) {
 renderOnboard();
 return;
 }
 let content = "";""",
        )

    # Clear data handler: force onboard then plan after choice
    old_clear = """await api.clearAllData(phrase);
      toast("All budget data cleared.");
      state.view = "dashboard";
      state.dash = null;
      state.lines = [];
      state.categories = [];
      state.history = [];
      state.income = null;
      state.status = await api.getStatus();
      if (!state.status.hasData) {
        renderOnboard();
        return;
      }
      await refreshAll();
      render();"""
    new_clear = """await api.clearAllData(phrase);
      state.view = "plan";
      state.dash = null;
      state.lines = [];
      state.categories = [];
      state.history = [];
      state.income = null;
      state.checkinResult = null;
      state.status = await api.getStatus();
      // Stay on startup wizard until user picks import/demo/blank.
      // render() also guards on !hasData so toasts cannot dump us on Dashboard.
      renderOnboard();
      return;"""
    if old_clear in text:
        text = text.replace(old_clear, new_clear)

    # After onboard choices, go to Plan
    text = text.replace(
        """await refreshAll();
      state.showCategoryNotice = true;
      render();
    } catch (e) {
      toast(String(e), true);
    }
  });
  app.querySelector("#ob-demo")!.addEventListener("click", async () => {
    try {
      await api.loadDemoData();
      toast("Demo budget loaded");
      await refreshAll();
      render();""",
        """await refreshAll();
      state.showCategoryNotice = true;
      state.view = "plan";
      render();
    } catch (e) {
      toast(String(e), true);
    }
  });
  app.querySelector("#ob-demo")!.addEventListener("click", async () => {
    try {
      await api.loadDemoData();
      toast("Demo budget loaded");
      await refreshAll();
      state.view = "plan";
      render();""",
    )

    # blank already goes to plan - ensure
    text = text.replace(
        """await api.upsertCategory({ id: null, name: "General", isFixed: false });
      await refreshAll();
      state.view = "plan";
      render();""",
        """await api.upsertCategory({ id: null, name: "General", isFixed: false });
      await refreshAll();
      state.view = "plan";
      render();""",
    )

    # Fix dismiss button text explicitly
    text = text.replace("Ill do this later", "I'll do this later")
    text = text.replace('id="notice-dismiss">', 'id="notice-dismiss">I\'ll do this later</button>')
    # if we doubled the button content fix
    text = re.sub(
        r'id="notice-dismiss">I\'ll do this later</button>[^<]*</button>',
        'id="notice-dismiss">I\'ll do this later</button>',
        text,
    )
    text = re.sub(
        r'id="notice-dismiss">[^<]*</button>',
        'id="notice-dismiss">I\'ll do this later</button>',
        text,
    )

    # Delete buttons use X
    text = re.sub(
        r'data-delcat="\$\{c\.id\}">[^<]*</button>',
        'data-delcat="${c.id}">X</button>',
        text,
    )
    text = re.sub(
        r'data-del="\$\{l\.id\}">Delete</button>',
        'data-del="${l.id}">Delete</button>',
        text,
    )

    # Nav icons simple ASCII
    text = text.replace('${nav("dashboard", "o", "Dashboard")}', '${nav("dashboard", "D", "Dashboard")}')
    text = text.replace('${nav("checkin", "*", "Check-In")}', '${nav("checkin", "C", "Check-In")}')
    text = text.replace('${nav("plan", "=", "Plan")}', '${nav("plan", "P", "Plan")}')
    text = text.replace('${nav("history", "H", "History")}', '${nav("history", "H", "History")}')
    text = text.replace('${nav("settings", "S", "Settings")}', '${nav("settings", "S", "Settings")}')
    # After sanitization icons may be empty
    text = re.sub(
        r'\$\{nav\("dashboard", "[^"]*", "Dashboard"\)\}',
        '${nav("dashboard", "D", "Dashboard")}',
        text,
    )
    text = re.sub(
        r'\$\{nav\("checkin", "[^"]*", "Check-In"\)\}',
        '${nav("checkin", "C", "Check-In")}',
        text,
    )
    text = re.sub(
        r'\$\{nav\("plan", "[^"]*", "Plan"\)\}',
        '${nav("plan", "P", "Plan")}',
        text,
    )
    text = re.sub(
        r'\$\{nav\("history", "[^"]*", "History"\)\}',
        '${nav("history", "H", "History")}',
        text,
    )
    text = re.sub(
        r'\$\{nav\("help", "[^"]*", "Help"\)\}',
        '${nav("help", "?", "Help")}',
        text,
    )
    text = re.sub(
        r'\$\{nav\("settings", "[^"]*", "Settings"\)\}',
        '${nav("settings", "S", "Settings")}',
        text,
    )

    # Month nav arrows
    text = re.sub(
        r'id="m-prev"[^>]*>[^<]*</button>',
        'id="m-prev" title="Previous month">&lt;</button>',
        text,
    )
    text = re.sub(
        r'id="m-next"[^>]*>[^<]*</button>',
        'id="m-next" title="Next month">&gt;</button>',
        text,
    )

    # Loading ellipsis
    text = text.replace("Loading", "Loading...")
    text = text.replace("Loading......", "Loading...")
    text = text.replace("Loading.....", "Loading...")
    text = text.replace("Loading....", "Loading...")

    # Placeholder search
    text = text.replace('placeholder="Search"', 'placeholder="Search..."')
    text = text.replace('placeholder="Search..."', 'placeholder="Search..."')

    return text


def main() -> None:
    original = SRC.read_text(encoding="utf-8", errors="replace")
    cleaned = to_ascii(original)
    cleaned = fix_clear_flow(cleaned)
    SRC.write_text(cleaned, encoding="utf-8", newline="\n")
    # report
    bad = [(i + 1, line) for i, line in enumerate(cleaned.splitlines()) if any(ord(c) > 127 for c in line)]
    print("non-ascii lines:", len(bad))
    print("I'll do this later present:", "I'll do this later" in cleaned)
    print("hasData guard present:", "!state.status.hasData" in cleaned)
    print("delcat X:", any('data-delcat="${c.id}">X</button>' in line for line in cleaned.splitlines()))
    print("done")


if __name__ == "__main__":
    main()
