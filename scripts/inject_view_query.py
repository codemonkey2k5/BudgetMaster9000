from pathlib import Path

p = Path(r"c:\temp\GrokBuild\BM9000\src\main.ts")
t = p.read_text(encoding="utf-8")
needle = """if (new URLSearchParams(location.search).has(\"selftest\")) {
 await runUiSelfTest();
 return;
 }
 const [y, m] = await api.currentMonth();"""
insert = """if (new URLSearchParams(location.search).has(\"selftest\")) {
 await runUiSelfTest();
 return;
 }
 const startView = new URLSearchParams(location.search).get(\"view\");
 if (startView === \"dashboard\" || startView === \"checkin\" || startView === \"plan\" || startView === \"history\" || startView === \"settings\" || startView === \"help\") {
 state.view = startView as View;
 }
 const [y, m] = await api.currentMonth();"""
if needle in t:
    t = t.replace(needle, insert)
    print("injected")
else:
    print("NOT FOUND")
    i = t.find("selftest")
    print(repr(t[i : i + 250]))
p.write_text(t, encoding="utf-8")
