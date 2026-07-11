from pathlib import Path
import re

p = Path(r"c:\temp\GrokBuild\BM9000\src\main.ts")
t = p.read_text(encoding="utf-8")

old_import = """const msg = await api.importLegacyJson(text);
 toast(msg);
 await refreshAll();
 state.showCategoryNotice = true;
 render();"""
new_import = """const msg = await api.importLegacyJson(text);
 await refreshAll();
 state.showCategoryNotice = true;
 state.view = "plan";
 toast(msg);"""
if old_import in t:
    t = t.replace(old_import, new_import)
    print("fixed import")
else:
    print("import pattern missing")

old_demo = """await api.loadDemoData();
 toast("Demo budget loaded");
 await refreshAll();
 render();"""
new_demo = """await api.loadDemoData();
 await refreshAll();
 state.view = "plan";
 toast("Demo budget loaded");"""
if old_demo in t:
    t = t.replace(old_demo, new_demo)
    print("fixed demo")
else:
    print("demo pattern missing")

old_css = """.help-body { color: var(--text-muted); line-height: 1.55; font-size: 0.95rem; }
.help-body strong { color: var(--text); }
.help-body code { font-family: var(--mono); font-size: 0.85em; }"""
new_css = """.help-body { color: var(--text-muted); line-height: 1.55; font-size: 0.95rem; }
.help-body p { margin: 0 0 0.85rem; }
.help-body p:last-child { margin-bottom: 0; }
.help-body ul { margin: 0 0 0.85rem; padding-left: 1.25rem; }
.help-body li { margin-bottom: 0.5rem; }
.help-body li:last-child { margin-bottom: 0; }
.help-body strong { color: var(--text); }
.help-body code { font-family: var(--mono); font-size: 0.85em; background: var(--bg); padding: 0.1em 0.35em; border-radius: 4px; }
.help-body em { font-style: italic; }"""
if old_css in t:
    t = t.replace(old_css, new_css)
    print("fixed help css")
else:
    print("css pattern missing")

t = t.replace("Private  Offline  Local", "Private / Offline / Local")
t = t.replace("Private - Offline - Local", "Private / Offline / Local")

# Wins checkmark
t = t.replace('class="win">* ${esc(w)}', 'class="win">OK ${esc(w)}')
t = t.replace('class="attn">! ${esc(w)}', 'class="attn">! ${esc(w)}')
t = t.replace('class="trend">-> ${esc(t)}', 'class="trend">* ${esc(t)}')

# Continue arrows already -> 

# Empty amount dash
t = t.replace(' : ":"', ' : "-"')
# that might be wrong - check for money display empty
t = re.sub(
    r'l\.actualAmount != null \? money\(l\.actualAmount\) : "[^"]*"',
    'l.actualAmount != null ? money(l.actualAmount) : "-"',
    t,
)

p.write_text(t, encoding="utf-8")
print("nonascii", sum(1 for c in t if ord(c) > 127))
print("I'll present", "I'll do this later" in t)
print("hasData guard", "if (!state.status.hasData)" in t)
print("clear -> onboard", "renderOnboard();\n      return;" in t or "renderOnboard();" in t)
