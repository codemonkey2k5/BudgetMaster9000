import "./styles.css";
import {
 Chart,
 DoughnutController,
 BarController,
 ArcElement,
 BarElement,
 CategoryScale,
 LinearScale,
 Tooltip,
 Legend,
} from "chart.js";
import {
 api,
 money,
 amountInput,
 monthName,
 statusLabel,
 gradeExplain,
 type AppStatus,
 type BudgetLine,
 type Category,
 type CheckInResult,
 type IncomeSettings,
 type MonthDashboard,
 type MonthInfo,
 type MonthLine,
} from "./api";
import { HELP_SECTIONS, searchHelp } from "./help";

Chart.register(
 DoughnutController,
 BarController,
 ArcElement,
 BarElement,
 CategoryScale,
 LinearScale,
 Tooltip,
 Legend
);

type View = "dashboard" | "checkin" | "plan" | "history" | "settings" | "help";

interface State {
 status: AppStatus | null;
 view: View;
 year: number;
 month: number;
 dash: MonthDashboard | null;
 lines: BudgetLine[];
 categories: Category[];
 income: IncomeSettings | null;
 history: MonthInfo[];
 filter: string;
 search: string;
 checkinStep: number;
 checkinActuals: Record<number, string>;
 checkinNotes: string;
 checkinResult: CheckInResult | null;
 editLine: BudgetLine | null;
 editCategory: Category | null;
 helpQuery: string;
 showCategoryNotice: boolean;
 toast: string | null;
 toastError: boolean;
}

const state: State = {
 status: null,
 view: "dashboard",
 year: new Date().getFullYear(),
 month: new Date().getMonth() + 1,
 dash: null,
 lines: [],
 categories: [],
 income: null,
 history: [],
 filter: "all",
 search: "",
 checkinStep: 0,
 checkinActuals: {},
 checkinNotes: "",
 checkinResult: null,
 editLine: null,
 editCategory: null,
 helpQuery: "",
 showCategoryNotice: false,
 toast: null,
 toastError: false,
};

let pieChart: Chart | null = null;
let barChart: Chart | null = null;
let toastTimer: number | null = null;
const app = document.querySelector<HTMLDivElement>("#app")!;

function variableLines(lines: MonthLine[]): MonthLine[] {
 return lines.filter((l) => !l.isFixed);
}
function fixedLines(lines: MonthLine[]): MonthLine[] {
 return lines.filter((l) => l.isFixed);
}

function seedCheckinActuals() {
 state.checkinActuals = {};
 if (!state.dash) return;
 for (const line of state.dash.lines) {
 if (line.isFixed) {
 state.checkinActuals[line.budgetLineId] = line.budgetAmount.toFixed(2);
 } else {
 const val =
 line.actualAmount != null ? line.actualAmount : line.budgetAmount;
 state.checkinActuals[line.budgetLineId] = val.toFixed(2);
 }
 }
}

function toast(msg: string, error = false) {
 state.toast = msg;
 state.toastError = error;
 render();
 if (toastTimer) window.clearTimeout(toastTimer);
 toastTimer = window.setTimeout(() => {
 state.toast = null;
 render();
 }, 3400);
}

function esc(s: string): string {
 return s
 .replace(/&/g, "&amp;")
 .replace(/</g, "&lt;")
 .replace(/>/g, "&gt;")
 .replace(/"/g, "&quot;");
}

function applyTheme(theme: string) {
 document.documentElement.setAttribute(
 "data-theme",
 theme === "light" ? "light" : "dark"
 );
}

async function refreshAll() {
 const [dash, lines, cats, income, history, status] = await Promise.all([
 api.getDashboard(state.year, state.month),
 api.listBudgetLines(),
 api.listCategories(),
 api.getIncome(),
 api.listHistory(),
 api.getStatus(),
 ]);
 state.dash = dash;
 state.lines = lines;
 state.categories = cats;
 state.income = income;
 state.history = history;
 state.status = status;
 state.showCategoryNotice = !!status.needsCategoryReview;
}

async function reloadDash() {
 try {
 state.dash = await api.getDashboard(state.year, state.month);
 render();
 } catch (e) {
 toast(String(e), true);
 }
}

/* UI self-test */
async function runUiSelfTest(): Promise<void> {
 const lines: string[] = [];
 const step = async (name: string, fn: () => Promise<unknown>) => {
 try {
 await fn();
 lines.push(`OK ${name}`);
 } catch (e) {
 lines.push(`FAIL ${name}: ${e}`);
 throw e;
 }
 };
 try {
 await step("current_month", () => api.currentMonth());
 await step("get_status", () => api.getStatus());
 await step("set_theme", () => api.setTheme("dark"));
 await step("load_demo_data", () => api.loadDemoData());
 await step("get_income", () => api.getIncome());
 await step("save_income", async () => {
 const inc = await api.getIncome();
 await api.saveIncome({ ...inc, netMonthly: inc.netMonthly || 4800 });
 });
 await step("list_categories", () => api.listCategories());
 await step("upsert_category", () =>
 api.upsertCategory({
 id: null,
 name: `SelfTestCat${Date.now()}`,
 isFixed: false,
 })
 );
 const cats = await api.listCategories();
 const catId = cats.find((c) => !c.isFixed)?.id || cats[0]?.id;
 if (!catId) throw new Error("no category");
 let lineId = 0;
 await step("upsert_budget_line", async () => {
 lineId = await api.upsertBudgetLine({
 id: null,
 name: "SelfTest Line",
 categoryId: catId,
 amount: 12.34,
 frequency: "month",
 notes: "ui selftest",
 });
 });
 await step("list_budget_lines", () => api.listBudgetLines());
 const [y, m] = await api.currentMonth();
 await step("get_dashboard", () => api.getDashboard(y, m));
 await step("reopen_month", async () => {
 try { await api.reopenMonth(y, m); } catch { /* already open */ }
 });
 await step("resync_month", () => api.resyncMonth(y, m));
 await step("update_month_meta", () =>
 api.updateMonthMeta(y, m, 4800, "ui selftest", null)
 );
 const dash = await api.getDashboard(y, m);
 const actuals = dash.lines.map((l) => ({
 budgetLineId: l.budgetLineId,
 actualAmount: l.isFixed ? l.budgetAmount : l.budgetAmount * 0.95,
 notes: null as string | null,
 }));
 await step("save_actuals", () => api.saveActuals(y, m, actuals));
 await step("complete_check_in", () =>
 api.completeCheckIn(y, m, actuals, null, "ui selftest")
 );
 await step("list_history", () => api.listHistory());
 await step("export_json", () => api.exportJson());
 await step("import_legacy_json", async () => {
 const payload = JSON.stringify({
 income: { netMonthly: 1000 },
 categories: ["Food"],
 expenses: [
 { name: "Groceries", category: "Food", amount: 100, frequency: "month" },
 ],
 });
 await api.importLegacyJson(payload);
 });
 await step("dismiss_category_review", () => api.dismissCategoryReview());
 if (lineId) {
 await step("delete_budget_line", () => api.deleteBudgetLine(lineId));
 }
 await step("report_ui_selftest", () =>
 api.reportUiSelftest(true, lines.join("\n"))
 );
 app.innerHTML = `<div class="center-screen"><div class="card lock-card"><h2>UI Self-Test PASS</h2><pre style="text-align:left;font-size:12px;max-height:60vh;overflow:auto">${esc(lines.join("\n"))}</pre></div></div>`;
 } catch (e) {
 lines.push(`ERROR: ${e}`);
 try {
 await api.reportUiSelftest(false, lines.join("\n"));
 } catch {
 /* ignore */
 }
 app.innerHTML = `<div class="center-screen"><div class="card lock-card"><h2>UI Self-Test FAIL</h2><pre style="text-align:left;font-size:12px;color:var(--rose)">${esc(lines.join("\n"))}</pre></div></div>`;
 }
}

async function boot() {
 try {
 if (new URLSearchParams(location.search).has("selftest")) {
 await runUiSelfTest();
 return;
 }
 const startView = new URLSearchParams(location.search).get("view");
 if (startView === "dashboard" || startView === "checkin" || startView === "plan" || startView === "history" || startView === "settings" || startView === "help") {
 state.view = startView as View;
 }
 const [y, m] = await api.currentMonth();
 state.year = y;
 state.month = m;
 state.status = await api.getStatus();
 applyTheme(state.status.theme);
 state.showCategoryNotice = !!state.status.needsCategoryReview;
 if (state.status.locked) {
 renderLock();
 return;
 }
 if (!state.status.hasData) {
 renderOnboard();
 return;
 }
 await refreshAll();
 render();
 } catch (e) {
 app.innerHTML = `<div class="center-screen"><div class="card lock-card"><h2>Startup error</h2><p>${esc(String(e))}</p></div></div>`;
 }
}

function renderLock() {
 app.innerHTML = `
 <div class="center-screen">
 <div class="card lock-card stack">
 <div class="brand" style="padding:0 0 0.5rem">
 <img src="/icon.png" alt="" onerror="this.style.display='none'"/>
 <div><h1>Budget Master 9000</h1><p>App Lock</p></div>
 </div>
 <h2>Unlock your budget</h2>
 <p>Enter your passphrase to open this local database.</p>
 <div class="field"><label>Passphrase</label><input type="password" id="lock-pass" /></div>
 <button class="btn btn-primary" id="btn-unlock">Unlock</button>
 </div>
 </div>`;
 const input = app.querySelector<HTMLInputElement>("#lock-pass")!;
 const go = async () => {
 try {
 const ok = await api.unlock(input.value);
 if (!ok) {
 toast("Incorrect passphrase", true);
 return;
 }
 state.status = await api.getStatus();
 if (!state.status.hasData) {
 renderOnboard();
 return;
 }
 await refreshAll();
 render();
 } catch (e) {
 toast(String(e), true);
 }
 };
 app.querySelector("#btn-unlock")!.addEventListener("click", go);
 input.addEventListener("keydown", (e) => e.key === "Enter" && go());
 input.focus();
}

function renderOnboard() {
 app.innerHTML = `
 <div class="center-screen">
 <div class="card onboard-card stack">
 <div class="brand" style="padding:0 0 0.5rem">
 <img src="/icon.png" alt="" onerror="this.style.display='none'"/>
 <div><h1>Budget Master 9000</h1><p>Private  -  Offline  -  Local</p></div>
 </div>
 <h2>Welcome</h2>
 <p>Set up your monthly budget. Import an existing file, load a demo, or start blank.</p>
 <button class="btn btn-primary" id="ob-import">Import data.json</button>
 <button class="btn btn-ghost" id="ob-demo">Load demo budget</button>
 <button class="btn btn-ghost" id="ob-blank">Start blank</button>
 <input type="file" id="ob-file" accept=".json,application/json" class="hidden" />
 <p class="dim">Tip: open <strong>Help</strong> anytime from the left menu for plain-language guides.</p>
 </div>
 </div>`;
 const file = app.querySelector<HTMLInputElement>("#ob-file")!;
 app.querySelector("#ob-import")!.addEventListener("click", () => file.click());
 file.addEventListener("change", async () => {
 const f = file.files?.[0];
 if (!f) return;
 try {
 const text = await f.text();
 const msg = await api.importLegacyJson(text);
 await refreshAll();
 // Ensure current dashboard month shows Settings income after import.
 try {
   const inc = await api.getIncome();
   if (inc.netMonthly > 0) {
     await api.updateMonthMeta(state.year, state.month, inc.netMonthly, null, null);
     await refreshAll();
   }
 } catch {
   /* ignore */
 }
 state.showCategoryNotice = true;
 state.view = "plan";
 toast(msg);
 } catch (e) {
 toast(String(e), true);
 }
 });
 app.querySelector("#ob-demo")!.addEventListener("click", async () => {
 try {
 await api.loadDemoData();
 await refreshAll();
 state.view = "plan";
 toast("Demo budget loaded");
 } catch (e) {
 toast(String(e), true);
 }
 });
  app.querySelector("#ob-blank")!.addEventListener("click", async () => {
    try {
      // Minimal setup so the app is no longer "empty", then open Settings
      // so the user can enter net income and start building their plan.
      await api.upsertCategory({ id: null, name: "General", isFixed: false });
      await refreshAll();
      state.view = "settings";
      state.toast = "Blank budget ready. Set your net income here, then open Plan to add lines.";
      state.toastError = false;
      render();
      if (toastTimer) window.clearTimeout(toastTimer);
      toastTimer = window.setTimeout(() => {
        state.toast = null;
        render();
      }, 4000);
    } catch (e) {
      toast(String(e), true);
    }
  });
}

function categoryNoticeHtml(): string {
 if (!state.showCategoryNotice) return "";
 return `
 <div class="notice-overlay" id="cat-notice">
 <div class="card notice-card stack">
 <h2>Quick setup after import</h2>
 <p>Your old budget was imported. Please take one minute on the <strong>Plan</strong> screen to check each <strong>category</strong>:</p>
 <ul class="tips-list">
 <li><strong>Fixed</strong>: bills that are the same every month (housing, utilities, subscriptions). Check-In will skip these.</li>
 <li><strong>Flexible</strong>: spending that changes (food, fun, charging). Check-In will ask for actual amounts.</li>
 </ul>
 <p class="dim">We may have guessed some Fixed categories for you. Open each category with Edit and confirm.</p>
 <div class="flex-gap">
 <button class="btn btn-primary" id="notice-plan">Go to Plan &amp; review categories</button>
 <button class="btn btn-ghost" id="notice-dismiss">I'll do this later</button>
 </div>
 </div>
 </div>`;
}

function shell(content: string): string {
 const nav = (id: View, icon: string, label: string) =>
 `<button class="nav-btn ${state.view === id ? "active" : ""}" data-nav="${id}"><span class="nav-icon">${icon}</span>${label}</button>`;

 return `
 <div class="app-shell">
 <aside class="sidebar">
 <div class="brand">
 <img src="/icon.png" alt="" onerror="this.style.display='none'"/>
 <div>
 <h1>Budget Master 9000</h1>
 <p>${state.status?.portable ? "Portable mode" : "Local database"}</p>
 </div>
 </div>
 ${nav("dashboard", "D", "Dashboard")}
 ${nav("checkin", "C", "Check-In")}
 ${nav("plan", "P", "Plan")}
 ${nav("history", "H", "History")}
 ${nav("help", "?", "Help")}
 ${nav("settings", "S", "Settings")}
 <div class="sidebar-foot">
 <button class="btn btn-ghost btn-sm" id="btn-theme">${state.status?.theme === "light" ? "Dark mode" : "Light mode"}</button>
 ${state.status?.lockEnabled ? `<button class="btn btn-ghost btn-sm" id="btn-lock">Lock now</button>` : ""}
 </div>
 </aside>
 <main class="main">
 ${content}
 </main>
 </div>
 ${categoryNoticeHtml()}
 ${state.toast ? `<div class="toast ${state.toastError ? "error" : ""}">${esc(state.toast)}</div>` : ""}
 `;
}

function monthNavHtml(): string {
 return `
 <div class="month-nav">
 <button type="button" id="m-prev" title="Previous month">&lt;</button>
 <div class="month-label">${monthName(state.month)} ${state.year}</div>
 <button type="button" id="m-next" title="Next month">&gt;</button>
 </div>`;
}

function bindMonthNav() {
 app.querySelector("#m-prev")?.addEventListener("click", async () => {
 if (state.month === 1) {
 state.month = 12;
 state.year -= 1;
 } else state.month -= 1;
 await reloadDash();
 });
 app.querySelector("#m-next")?.addEventListener("click", async () => {
 if (state.month === 12) {
 state.month = 1;
 state.year += 1;
 } else state.month += 1;
 await reloadDash();
 });
}

function bindShell() {
 app.querySelectorAll<HTMLButtonElement>("[data-nav]").forEach((btn) => {
 btn.addEventListener("click", () => {
 state.view = btn.dataset.nav as View;
 if (state.view === "checkin") {
 state.checkinStep = 0;
 state.checkinResult = null;
 seedCheckinActuals();
 }
 render();
 });
 });
 app.querySelector("#btn-theme")?.addEventListener("click", async () => {
 const next = state.status?.theme === "light" ? "dark" : "light";
 await api.setTheme(next);
 applyTheme(next);
 if (state.status) state.status.theme = next;
 render();
 });
 app.querySelector("#btn-lock")?.addEventListener("click", async () => {
 await api.lockApp();
 state.status = await api.getStatus();
 renderLock();
 });
 app.querySelector("#notice-plan")?.addEventListener("click", async () => {
 try {
 await api.dismissCategoryReview();
 } catch {
 /* ignore */
 }
 state.showCategoryNotice = false;
 if (state.status) state.status.needsCategoryReview = false;
 state.view = "plan";
 render();
 });
 app.querySelector("#notice-dismiss")?.addEventListener("click", async () => {
 try {
 await api.dismissCategoryReview();
 } catch {
 /* ignore */
 }
 state.showCategoryNotice = false;
 if (state.status) state.status.needsCategoryReview = false;
 render();
 });
}

function render() {
 if (!state.status || state.status.locked) {
 renderLock();
 return;
 }
 if (!state.status.hasData) {
 renderOnboard();
 return;
 }
 let content = "";
 switch (state.view) {
 case "dashboard":
 content = viewDashboard();
 break;
 case "checkin":
 content = viewCheckin();
 break;
 case "plan":
 content = viewPlan();
 break;
 case "history":
 content = viewHistory();
 break;
 case "help":
 content = viewHelp();
 break;
 case "settings":
 content = viewSettings();
 break;
 }
 app.innerHTML = shell(content);
 bindShell();
 switch (state.view) {
 case "dashboard":
 bindDashboard();
 break;
 case "checkin":
 bindCheckin();
 break;
 case "plan":
 bindPlan();
 break;
 case "history":
 bindHistory();
 break;
 case "help":
 bindHelp();
 break;
 case "settings":
 bindSettings();
 break;
 }
}

/* DASHBOARD */
function viewDashboard(): string {
 const d = state.dash;
 if (!d) return `<div class="empty">Loading...</div>`;

 const hasActuals = d.lines.some((l) => l.actualAmount != null);
 const remaining =
 d.month.netIncome - (hasActuals ? d.actualTotal : d.budgetedTotal);
 const variance = hasActuals ? d.budgetedTotal - d.actualTotal : remaining;

 let lines = d.lines;
 if (state.filter !== "all") lines = lines.filter((l) => l.status === state.filter);
 if (state.search.trim()) {
 const q = state.search.toLowerCase();
 lines = lines.filter(
 (l) =>
 l.name.toLowerCase().includes(q) ||
 l.categoryName.toLowerCase().includes(q)
 );
 }

 const rows = lines
 .map((l) => {
 const pct = l.pctUsed ?? 0;
 const barPct = Math.min(pct, 100);
 const progClass = l.status === "over" ? "over" : pct > 90 ? "warn" : "";
 return `
 <tr>
 <td>
 <div style="font-weight:700">${esc(l.name)}</div>
 <div class="dim">${l.isFixed ? "Fixed bill" : "Flexible"}</div>
 </td>
 <td><span class="badge badge-cat" style="border-left:3px solid ${l.categoryColor}">${esc(l.categoryName)}</span></td>
 <td class="mono">${money(l.budgetAmount)}</td>
 <td class="mono">${l.actualAmount != null ? money(l.actualAmount) : "-"}</td>
 <td>
 <div class="progress ${progClass}" title="${pct.toFixed(0)}%"><span style="width:${barPct}%"></span></div>
 <div class="dim mt-1">${l.actualAmount != null ? pct.toFixed(0) + "%" : ": "}</div>
 </td>
 <td><span class="badge badge-${l.status}">${statusLabel(l.status)}</span></td>
 </tr>`;
 })
 .join("");

 return `
 <div class="page-header">
 <div>
 <h2>Monthly health</h2>
 <div class="sub">
 ${monthName(state.month)} ${state.year}  - 
 ${d.month.status === "reviewed" ? "Closed (reviewed)" : "Open (still editable)"}
 ${d.month.grade ? `  -  Grade <strong title="${esc(gradeExplain(d.month.grade))}">${esc(d.month.grade)}</strong>` : ""}
 ${d.savingsRate != null ? `  -  Savings ${d.savingsRate.toFixed(0)}%` : ""}
 </div>
 </div>
 ${monthNavHtml()}
 </div>

 <div class="info-callout mb-2">
 <strong>Plan vs this month:</strong> the Dashboard shows a <em>snapshot</em> of your Plan for
 <strong>${monthName(state.month)} ${state.year}</strong>.
 Edit recurring amounts on <strong>Plan</strong>. If this month is still open and you added new Plan lines,
 click <em>Update this month from Plan</em> below.
 <a href="#" data-goto-help="plan-vs-dashboard">Learn more in Help</a>
 </div>

 <div class="grid-5 mb-2">
 <div class="card stat-card emerald"><div class="stat-label">Net income</div><div class="stat-value">${money(d.month.netIncome)}</div></div>
 <div class="card stat-card"><div class="stat-label">Budgeted</div><div class="stat-value">${money(d.budgetedTotal)}</div></div>
 <div class="card stat-card rose"><div class="stat-label">${hasActuals ? "Actual spent" : "No actuals yet"}</div><div class="stat-value">${hasActuals ? money(d.actualTotal) : ": "}</div></div>
 <div class="card stat-card amber"><div class="stat-label">${hasActuals ? "Vs budget" : "Planned surplus"}</div><div class="stat-value ${variance >= 0 ? "pos" : "neg"}">${money(variance)}</div></div>
 <div class="card stat-card sky"><div class="stat-label">Remaining cash</div><div class="stat-value ${remaining >= 0 ? "pos" : "neg"}">${money(remaining)}</div></div>
 </div>

 <div class="grid-2 mb-2">
 <div class="card"><div class="card-pad section-title">Spending by category</div><div class="chart-box"><canvas id="chart-pie"></canvas></div></div>
 <div class="card"><div class="card-pad section-title">Income vs budget vs actual</div><div class="chart-box"><canvas id="chart-bar"></canvas></div></div>
 </div>

 <div class="card">
 <div class="card-pad">
 <div class="page-header" style="margin-bottom:0.75rem">
 <div class="section-title" style="margin:0">This month's lines</div>
 <div class="flex-gap">
 <input type="search" id="dash-search" placeholder="Search..." value="${esc(state.search)}"
 style="background:var(--bg);border:1px solid var(--border);border-radius:10px;padding:0.45rem 0.7rem;min-width:160px" />
 <button class="btn btn-ghost btn-sm" id="resync" title="Copy any new Plan lines into this open month">Update this month from Plan</button>
 </div>
 </div>
 <p class="dim mb-1"><strong>Update this month from Plan</strong> adds new Plan items into this month's snapshot if the month is still open. It will not change a closed month. <a href="#" data-goto-help="sync">Help</a></p>
 <div class="filters">
 ${["all", "under", "on_plan", "over", "unset"]
 .map(
 (f) =>
 `<button class="chip ${state.filter === f ? "active" : ""}" data-filter="${f}">${f === "all" ? "All" : statusLabel(f)}</button>`
 )
 .join("")}
 </div>
 </div>
 <div class="table-wrap">
 <table class="data">
 <thead><tr><th>Details</th><th>Category</th><th>Budget</th><th>Actual</th><th>Used</th><th>Status</th></tr></thead>
 <tbody>${rows || `<tr><td colspan="6" class="empty">No lines match.</td></tr>`}</tbody>
 </table>
 </div>
 </div>`;
}

function bindDashboard() {
 bindMonthNav();
 app.querySelectorAll<HTMLButtonElement>("[data-filter]").forEach((b) => {
 b.addEventListener("click", () => {
 state.filter = b.dataset.filter || "all";
 render();
 });
 });
 app.querySelector("#dash-search")?.addEventListener("input", (e) => {
 state.search = (e.target as HTMLInputElement).value;
 const start = (e.target as HTMLInputElement).selectionStart;
 render();
 const el = app.querySelector<HTMLInputElement>("#dash-search");
 if (el) {
 el.focus();
 el.setSelectionRange(start, start);
 }
 });
 app.querySelector("#resync")?.addEventListener("click", async () => {
 try {
 await api.resyncMonth(state.year, state.month);
 toast("This month was updated from your Plan");
 await reloadDash();
 } catch (e) {
 toast(String(e), true);
 }
 });
 app.querySelectorAll<HTMLAnchorElement>("[data-goto-help]").forEach((a) => {
 a.addEventListener("click", (e) => {
 e.preventDefault();
 state.view = "help";
 state.helpQuery = a.dataset.gotoHelp || "";
 render();
 });
 });
 paintCharts();
}

/** High-contrast pie palette (hues spaced so neighbors stay distinct). */
const PIE_PALETTE = [
  "#4f46e5", // indigo
  "#f59e0b", // amber
  "#059669", // emerald
  "#e11d48", // rose
  "#2563eb", // blue
  "#ca8a04", // yellow-gold
  "#7c3aed", // violet
  "#0d9488", // teal
  "#ea580c", // orange
  "#db2777", // pink
  "#0891b2", // cyan
  "#65a30d", // lime
];

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
  ];
}

function colorDistance(a: string, b: string): number {
  const [ar, ag, ab] = hexToRgb(a);
  const [br, bg, bb] = hexToRgb(b);
  // Weighted RGB distance (emphasize green/red differences people notice)
  const dr = ar - br;
  const dg = ag - bg;
  const db = ab - bb;
  return Math.sqrt(2 * dr * dr + 4 * dg * dg + 3 * db * db);
}

/**
 * Assign slice colors so each slice contrasts with its previous (and next
 * wraps to first). Avoids similar hues sitting side by side on the pie.
 */
function assignContrastingPieColors(count: number): string[] {
  if (count <= 0) return [];
  const palette = PIE_PALETTE;
  const result: string[] = new Array(count);
  const used = new Set<number>();

  result[0] = palette[0];
  used.add(0);

  for (let i = 1; i < count; i++) {
    let bestJ = -1;
    let bestScore = -1;
    for (let j = 0; j < palette.length; j++) {
      if (used.has(j) && used.size < palette.length) continue;
      // Prefer unused; if all used, allow reuse with max distance
      const distPrev = colorDistance(palette[j], result[i - 1]);
      // When closing the circle, also keep last away from first
      const distFirst =
        i === count - 1 ? colorDistance(palette[j], result[0]) : distPrev;
      const score = Math.min(distPrev, distFirst);
      if (score > bestScore) {
        bestScore = score;
        bestJ = j;
      }
    }
    if (bestJ < 0) bestJ = i % palette.length;
    result[i] = palette[bestJ];
    used.add(bestJ);
    if (used.size >= palette.length) used.clear();
  }
  return result;
}

function paintCharts() {
 const d = state.dash;
 if (!d) return;
 const pie = document.getElementById("chart-pie") as HTMLCanvasElement | null;
 const bar = document.getElementById("chart-bar") as HTMLCanvasElement | null;
 if (!pie || !bar) return;

 const catMap = new Map<string, number>();
 for (const l of d.lines) {
 const amt = l.actualAmount ?? l.budgetAmount;
 catMap.set(l.categoryName, (catMap.get(l.categoryName) || 0) + amt);
 }
 // Largest slice first so the contrast algorithm is stable and readable
 const labels = [...catMap.keys()].sort(
   (a, b) => (catMap.get(b) || 0) - (catMap.get(a) || 0)
 );
 const data = labels.map((k) => catMap.get(k)!);
 const colors = assignContrastingPieColors(labels.length);
 const muted =
 getComputedStyle(document.documentElement).getPropertyValue("--text-muted") ||
 "#9aa3c7";
 const border =
 getComputedStyle(document.documentElement).getPropertyValue("--bg-card")?.trim() ||
 "#1c2138";

 pieChart?.destroy();
 pieChart = new Chart(pie, {
 type: "doughnut",
 data: {
 labels,
 datasets: [
   {
     data,
     backgroundColor: colors,
     borderWidth: 2,
     borderColor: border,
     hoverBorderWidth: 2,
   },
 ],
 },
 options: {
 responsive: true,
 maintainAspectRatio: false,
 plugins: {
 legend: {
 position: "bottom",
 labels: { color: muted, boxWidth: 12 },
 },
 },
 },
 });

 const hasActuals = d.lines.some((l) => l.actualAmount != null);
 barChart?.destroy();
 barChart = new Chart(bar, {
 type: "bar",
 data: {
 labels: ["This month"],
 datasets: [
 { label: "Income", data: [d.month.netIncome], backgroundColor: "#10b981" },
 { label: "Budgeted", data: [d.budgetedTotal], backgroundColor: "#6366f1" },
 {
 label: "Actual",
 data: [hasActuals ? d.actualTotal : 0],
 backgroundColor: "#f43f5e",
 },
 ],
 },
 options: {
 responsive: true,
 maintainAspectRatio: false,
 scales: {
 x: { ticks: { color: muted }, grid: { display: false } },
 y: {
 beginAtZero: true,
 ticks: { color: muted },
 grid: { color: "rgba(128,128,128,0.15)" },
 },
 },
 plugins: { legend: { labels: { color: muted } } },
 },
 });
}

/* CHECK-IN */
function guideBox(title: string, html: string): string {
 return `<div class="guide-box mb-2"><div class="guide-title">${title}</div><div class="guide-body">${html}</div></div>`;
}

function checkinHeader(title: string, sub: string): string {
 return `
 <div class="page-header">
 <div>
 <h2>${title}</h2>
 <div class="sub">${sub}</div>
 </div>
 ${monthNavHtml()}
 </div>`;
}

function viewCheckin(): string {
 const d = state.dash;
 if (!d) return `<div class="empty">Loading...</div>`;

 if (state.checkinResult) {
 const r = state.checkinResult;
 return `
 ${checkinHeader("Check-In complete", `${monthName(state.month)} ${state.year}`)}
 <div class="card scorecard">
 <div class="grade-ring" style="--pct:${Math.min(r.score, 100)}%"><span>${esc(r.grade)}</span></div>
 <p class="dim" style="max-width:420px;margin:0 auto">${esc(gradeExplain(r.grade))}</p>
 <div class="stat-value" style="font-size:1.1rem;margin-top:0.75rem">Score ${r.score.toFixed(0)}  -  Savings ${r.savingsRate.toFixed(0)}%</div>
 <div class="dim mt-1">${r.counts.under} under  -  ${r.counts.onPlan} on plan  -  ${r.counts.over} over</div>
 <div class="grid-2 mt-2" style="text-align:left">
 <div>
 <div class="section-title">Where you did well</div>
 <ul class="insight-list">${r.wins.map((w) => `<li class="win">OK ${esc(w)}</li>`).join("") || "<li class=\"win\">Keep going: every month builds the habit.</li>"}</ul>
 </div>
 <div>
 <div class="section-title">Needs attention</div>
 <ul class="insight-list">${r.attention.map((w) => `<li class="attn">! ${esc(w)}</li>`).join("") || "<li class=\"win\">No major overspends.</li>"}</ul>
 </div>
 </div>
 ${r.trends.length ? `<div class="mt-2" style="text-align:left"><div class="section-title">Trends</div><ul class="insight-list">${r.trends.map((t) => `<li class="trend">* ${esc(t)}</li>`).join("")}</ul></div>` : ""}
 ${r.suggestion ? `<p class="mt-2" style="text-align:left"><strong>Suggestion:</strong> ${esc(r.suggestion)}</p>` : ""}
 <div class="flex-gap mt-2" style="justify-content:center">
 <button class="btn btn-primary" id="ci-done">Back to dashboard</button>
 </div>
 </div>`;
 }

 const steps = ["1. Income", "2. Flexible spending", "3. Finish"];
 const stepHtml = steps
 .map(
 (s, i) =>
 `<div class="wizard-step ${state.checkinStep === i ? "active" : ""} ${state.checkinStep > i ? "done" : ""}">${s}</div>`
 )
 .join("");

 const fixedCount = fixedLines(d.lines).length;
 const vars = variableLines(d.lines);

 if (state.checkinStep === 0) {
 return `
 ${checkinHeader("Monthly Check-In", `Closing out ${monthName(state.month)} ${state.year}`)}
 <div class="wizard-steps">${stepHtml}</div>
 ${guideBox(
 "What this step means",
 `You are recording how <strong>${monthName(state.month)}</strong> really went, so the app can compare it to your Plan.
 <br/><br/><strong>Net income</strong> is take-home pay for this month only (after taxes). If you got a bonus or missed a paycheck, change the number.
 <br/><br/>Notes are optional: for your own memory later (e.g. "vacation week").`
 )}
 <div class="card card-pad stack" style="max-width:560px">
 <div class="field">
 <label>Net income for ${monthName(state.month)} ($)</label>
 <input type="number" step="0.01" id="ci-income" value="${d.month.netIncome}" />
 </div>
 <div class="field">
 <label>Notes (optional)</label>
 <textarea id="ci-notes" rows="3" placeholder="Anything unusual this month?">${esc(state.checkinNotes)}</textarea>
 </div>
 <p class="dim">Next you'll enter flexible spending only (${vars.length} line${vars.length === 1 ? "" : "s"}). ${fixedCount} fixed bill${fixedCount === 1 ? "" : "s"} are applied automatically.</p>
 <button class="btn btn-primary" id="ci-next">Continue to spending -></button>
 </div>`;
 }

 if (state.checkinStep === 1) {
 const rows = vars
 .map((l) => {
 const val =
 state.checkinActuals[l.budgetLineId] ?? l.budgetAmount.toFixed(2);
 return `
 <tr>
 <td>
 <div style="font-weight:700">${esc(l.name)}</div>
 <span class="badge badge-cat">${esc(l.categoryName)}</span>
 </td>
 <td class="mono">${money(l.budgetAmount)}</td>
 <td>
 <div class="money-input">
 <span class="money-prefix">$</span>
 <input type="number" step="0.01" class="ci-amt" data-id="${l.budgetLineId}" value="${esc(val)}" />
 </div>
 </td>
 </tr>`;
 })
 .join("");

 return `
 ${checkinHeader("Enter flexible spending", `${monthName(state.month)} ${state.year}`)}
 <div class="wizard-steps">${stepHtml}</div>
 ${guideBox(
 "What this step means",
 `Only <strong>flexible</strong> categories appear here (food, fun, etc.). Fixed bills (rent, utilities, subscriptions) are already assumed "on plan": you do not need to type them.
 <br/><br/>Each row shows your <strong>Plan amount</strong> and an <strong>Actual</strong> box. Actual starts equal to the plan. Change a number only if you spent more or less.
 <br/><br/>Example: Plan groceries $400, you spent $450 -> type <strong>450.00</strong>.`
 )}
 ${
 fixedCount
 ? `<p class="dim mb-1"><strong>${fixedCount}</strong> fixed bill${fixedCount === 1 ? "" : "s"} auto-applied at plan amount.</p>`
 : ""
 }
 <div class="flex-gap mb-1">
 <button class="btn btn-ghost btn-sm" id="ci-reset">Reset flexible lines to plan amounts</button>
 </div>
 <div class="card">
 <div class="table-wrap">
 <table class="data">
 <thead><tr><th>What you track</th><th>Plan (budget)</th><th>What you actually spent</th></tr></thead>
 <tbody>${
 rows ||
 `<tr><td colspan="3" class="empty">No flexible lines. On Plan, mark some categories as Flexible, or add spending lines under those categories.</td></tr>`
 }</tbody>
 </table>
 </div>
 </div>
 <div class="flex-gap mt-2">
 <button class="btn btn-ghost" id="ci-back"><- Back</button>
 <button class="btn btn-primary" id="ci-next">Review totals -></button>
 </div>`;
 }

 // step 2
 let budget = 0;
 let actual = 0;
 for (const l of d.lines) {
 const a = parseFloat(state.checkinActuals[l.budgetLineId] || "0") || 0;
 budget += l.budgetAmount;
 actual += a;
 }
 const varPreview = vars.map((l) => {
 const a = parseFloat(state.checkinActuals[l.budgetLineId] || "0") || 0;
 const over = a > l.budgetAmount * 1.02;
 return `<li class="${over ? "attn" : "win"}">${esc(l.name)}: plan ${money(l.budgetAmount)}  -  actual ${money(a)}</li>`;
 });

 return `
 ${checkinHeader("Review & finish", `${monthName(state.month)} ${state.year}`)}
 <div class="wizard-steps">${stepHtml}</div>
 ${guideBox(
 "What this step means",
 `You're about to <strong>close</strong> this month. Totals include fixed bills (auto) plus the flexible amounts you entered.
 <br/><br/>Clicking <strong>Complete Check-In</strong> saves a grade and scorecard to History. You can still reopen the month later if you made a mistake.`
 )}
 <div class="grid-2">
 <div class="card card-pad">
 <div class="section-title">Totals</div>
 <p>Budgeted: <strong class="mono">${money(budget)}</strong></p>
 <p>Actual: <strong class="mono">${money(actual)}</strong></p>
 <p>Difference: <strong class="mono" style="color:${budget - actual >= 0 ? "var(--emerald)" : "var(--rose)"}">${money(budget - actual)}</strong>
 <span class="dim">(${budget - actual >= 0 ? "under / on plan" : "over plan"})</span></p>
 <p class="dim mt-1">${fixedCount} fixed bill${fixedCount === 1 ? "" : "s"} included automatically.</p>
 </div>
 <div class="card card-pad">
 <div class="section-title">Flexible spending you entered</div>
 <ul class="insight-list" style="max-height:240px;overflow:auto">${
 varPreview.join("") || "<li class=\"win\">No flexible lines.</li>"
 }</ul>
 </div>
 </div>
 <div class="flex-gap mt-2">
 <button class="btn btn-ghost" id="ci-back"><- Back</button>
 <button class="btn btn-success" id="ci-finish">Complete Check-In</button>
 </div>`;
}

function bindCheckin() {
 bindMonthNav();
 app.querySelector("#ci-next")?.addEventListener("click", async () => {
 if (state.checkinStep === 0) {
 const income = parseFloat(
 (app.querySelector("#ci-income") as HTMLInputElement).value
 );
 state.checkinNotes =
 (app.querySelector("#ci-notes") as HTMLTextAreaElement)?.value || "";
 try {
 await api.updateMonthMeta(
 state.year,
 state.month,
 income,
 state.checkinNotes,
 null
 );
 state.dash = await api.getDashboard(state.year, state.month);
 seedCheckinActuals();
 state.checkinStep = 1;
 render();
 } catch (e) {
 toast(String(e), true);
 }
 return;
 }
 if (state.checkinStep === 1) {
 collectActualsFromDom();
 if (state.dash) {
 for (const l of fixedLines(state.dash.lines)) {
 state.checkinActuals[l.budgetLineId] = l.budgetAmount.toFixed(2);
 }
 }
 state.checkinStep = 2;
 render();
 }
 });
 app.querySelector("#ci-back")?.addEventListener("click", () => {
 collectActualsFromDom();
 state.checkinStep = Math.max(0, state.checkinStep - 1);
 render();
 });
 app.querySelector("#ci-reset")?.addEventListener("click", () => {
 if (!state.dash) return;
 for (const l of variableLines(state.dash.lines)) {
 state.checkinActuals[l.budgetLineId] = l.budgetAmount.toFixed(2);
 }
 for (const l of fixedLines(state.dash.lines)) {
 state.checkinActuals[l.budgetLineId] = l.budgetAmount.toFixed(2);
 }
 render();
 });
 app.querySelectorAll<HTMLInputElement>(".ci-amt").forEach((inp) => {
 inp.addEventListener("change", () => {
 state.checkinActuals[Number(inp.dataset.id)] = inp.value;
 });
 });
 app.querySelector("#ci-finish")?.addEventListener("click", async () => {
 collectActualsFromDom();
 if (!state.dash) return;
 for (const l of fixedLines(state.dash.lines)) {
 state.checkinActuals[l.budgetLineId] = l.budgetAmount.toFixed(2);
 }
 const actuals = state.dash.lines.map((l) => ({
 budgetLineId: l.budgetLineId,
 actualAmount:
 parseFloat(state.checkinActuals[l.budgetLineId] || "0") || 0,
 notes: null as string | null,
 }));
 try {
 state.checkinResult = await api.completeCheckIn(
 state.year,
 state.month,
 actuals,
 null,
 state.checkinNotes
 );
 await refreshAll();
 render();
 } catch (e) {
 toast(String(e), true);
 }
 });
 app.querySelector("#ci-done")?.addEventListener("click", () => {
 state.view = "dashboard";
 state.checkinResult = null;
 render();
 });
}

function collectActualsFromDom() {
 app.querySelectorAll<HTMLInputElement>(".ci-amt").forEach((inp) => {
 state.checkinActuals[Number(inp.dataset.id)] = inp.value;
 });
}

/* PLAN */
function viewPlan(): string {
 const edit = state.editLine;
 const editCat = state.editCategory;
 const totalMonthly = state.lines.reduce((s, l) => s + l.monthlyAmount, 0);

 const rows = state.lines
 .map(
 (l) => `
 <tr>
 <td>
 <div style="font-weight:700">${esc(l.name)}</div>
 <div class="dim">${l.frequency}${l.isFixed ? "  -  fixed category" : "  -  flexible category"}</div>
 </td>
 <td><span class="badge badge-cat">${esc(l.categoryName)}</span></td>
 <td class="mono">${money(l.amount)}</td>
 <td class="mono">${money(l.monthlyAmount)}</td>
 <td class="flex-gap">
 <button class="btn btn-ghost btn-sm" data-edit="${l.id}">Edit</button>
 <button class="btn btn-danger btn-sm" data-del="${l.id}">Delete</button>
 </td>
 </tr>`
 )
 .join("");

 return `
 <div class="page-header">
 <div>
 <h2>Budget plan</h2>
 <div class="sub">Your recurring template  -  ${money(totalMonthly)} / month planned</div>
 </div>
 </div>

 <div class="info-callout mb-2">
 The Plan is the master list used when a new month starts.
 Categories are either <strong>Fixed</strong> (same every month: skipped in Check-In) or <strong>Flexible</strong> (you enter actuals).
 <a href="#" data-goto-help="categories-fixed">Help: Fixed vs Flexible</a>
 </div>

 <div class="card card-pad mb-2">
 <div class="section-title">${edit ? "Edit budget line" : "Add budget line"}</div>
 <form id="line-form" class="form-row">
 <input type="hidden" id="line-id" value="${edit?.id ?? ""}" />
 <div class="field">
 <label>Name</label>
 <input id="line-name" required value="${esc(edit?.name ?? "")}" placeholder="e.g. Groceries" />
 </div>
 <div class="field">
 <label>Category</label>
 <select id="line-cat">${state.categories
 .map(
 (c) =>
 `<option value="${c.id}" ${edit?.categoryId === c.id ? "selected" : ""}>${esc(c.name)} (${c.isFixed ? "Fixed" : "Flexible"})</option>`
 )
 .join("")}</select>
 </div>
 <div class="field">
 <label>Amount ($)</label>
 <div class="money-input">
 <span class="money-prefix">$</span>
 <input type="number" step="0.01" id="line-amt" required
 value="${edit ? amountInput(edit.amount) : ""}" placeholder="0.00" />
 </div>
 </div>
 <div class="field">
 <label>How often</label>
 <select id="line-freq">
 <option value="week" ${edit?.frequency === "week" ? "selected" : ""}>Weekly</option>
 <option value="month" ${!edit || edit.frequency === "month" ? "selected" : ""}>Monthly</option>
 <option value="year" ${edit?.frequency === "year" ? "selected" : ""}>Yearly</option>
 </select>
 </div>
 <button class="btn btn-primary" type="submit">${edit ? "Save line" : "Add line"}</button>
 ${edit ? `<button class="btn btn-ghost" type="button" id="line-cancel">Cancel</button>` : ""}
 </form>
 <p class="dim mt-1">Fixed vs flexible is set on the <strong>category</strong>, not on each line.</p>
 </div>

 <div class="grid-2 mb-2">
 <div class="card">
 <div class="card-pad section-title">Budget lines</div>
 <div class="table-wrap">
 <table class="data">
 <thead><tr><th>Name</th><th>Category</th><th>Amount</th><th>Monthly</th><th></th></tr></thead>
 <tbody>${rows || `<tr><td colspan="5" class="empty">No budget lines yet.</td></tr>`}</tbody>
 </table>
 </div>
 </div>
 <div class="card card-pad">
 <div class="section-title">${editCat ? "Edit category" : "Categories"}</div>
 <form id="cat-form" class="stack mb-1">
 <input type="hidden" id="cat-id" value="${editCat?.id ?? ""}" />
 <div class="field">
 <label>Category name</label>
 <input id="cat-name" required placeholder="e.g. Housing" value="${esc(editCat?.name ?? "")}" />
 </div>
 <div class="field">
 <label>Type</label>
 <select id="cat-fixed">
 <option value="0" ${!editCat?.isFixed ? "selected" : ""}>Flexible: amounts change (food, fun...)</option>
 <option value="1" ${editCat?.isFixed ? "selected" : ""}>Fixed: same every month (bills, subscriptions...)</option>
 </select>
 </div>
 <div class="flex-gap">
 <button class="btn btn-primary btn-sm" type="submit">${editCat ? "Save category" : "Add category"}</button>
 ${editCat ? `<button class="btn btn-ghost btn-sm" type="button" id="cat-cancel">Cancel</button>` : ""}
 </div>
 </form>
 <ul class="insight-list">
 ${state.categories
 .map(
 (c) =>
 `<li class="win" style="display:flex;justify-content:space-between;align-items:center;gap:0.5rem">
 <span>
 <span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${c.color};margin-right:8px"></span>
 ${esc(c.name)}
 <span class="badge ${c.isFixed ? "badge-on_plan" : "badge-under"}" style="margin-left:6px">${c.isFixed ? "Fixed" : "Flexible"}</span>
 </span>
 <span class="flex-gap">
 <button class="btn btn-ghost btn-sm" data-editcat="${c.id}">Edit</button>
 <button class="btn btn-danger btn-sm" data-delcat="${c.id}">X</button>
 </span>
 </li>`
 )
 .join("")}
 </ul>
 </div>
 </div>`;
}

function bindPlan() {
 app.querySelectorAll<HTMLAnchorElement>("[data-goto-help]").forEach((a) => {
 a.addEventListener("click", (e) => {
 e.preventDefault();
 state.view = "help";
 state.helpQuery = a.dataset.gotoHelp || "";
 render();
 });
 });
 app.querySelector("#line-form")?.addEventListener("submit", async (e) => {
 e.preventDefault();
 const idRaw = (app.querySelector("#line-id") as HTMLInputElement).value;
 const payload = {
 id: idRaw ? Number(idRaw) : null,
 name: (app.querySelector("#line-name") as HTMLInputElement).value.trim(),
 categoryId: Number(
 (app.querySelector("#line-cat") as HTMLSelectElement).value
 ),
 amount:
 parseFloat((app.querySelector("#line-amt") as HTMLInputElement).value) ||
 0,
 frequency: (app.querySelector("#line-freq") as HTMLSelectElement).value,
 notes: null as string | null,
 };
 try {
 await api.upsertBudgetLine(payload);
 state.editLine = null;
 await refreshAll();
 try {
 await api.resyncMonth(state.year, state.month);
 } catch {
 /* closed month */
 }
 toast(payload.id ? "Line updated" : "Line added");
 render();
 } catch (err) {
 toast(String(err), true);
 }
 });
 app.querySelector("#line-cancel")?.addEventListener("click", () => {
 state.editLine = null;
 render();
 });
 app.querySelectorAll<HTMLButtonElement>("[data-edit]").forEach((b) => {
 b.addEventListener("click", () => {
 state.editLine =
 state.lines.find((l) => l.id === Number(b.dataset.edit)) || null;
 render();
 });
 });
 app.querySelectorAll<HTMLButtonElement>("[data-del]").forEach((b) => {
 b.addEventListener("click", async () => {
 if (!confirm("Delete this budget line?")) return;
 try {
 await api.deleteBudgetLine(Number(b.dataset.del));
 await refreshAll();
 toast("Deleted");
 render();
 } catch (e) {
 toast(String(e), true);
 }
 });
 });
 app.querySelector("#cat-form")?.addEventListener("submit", async (e) => {
 e.preventDefault();
 const idRaw = (app.querySelector("#cat-id") as HTMLInputElement).value;
 const name = (app.querySelector("#cat-name") as HTMLInputElement).value.trim();
 const isFixed =
 (app.querySelector("#cat-fixed") as HTMLSelectElement).value === "1";
 try {
 await api.upsertCategory({
 id: idRaw ? Number(idRaw) : null,
 name,
 isFixed,
 });
 state.editCategory = null;
 await refreshAll();
 toast(idRaw ? "Category updated" : "Category added");
 render();
 } catch (err) {
 toast(String(err), true);
 }
 });
 app.querySelector("#cat-cancel")?.addEventListener("click", () => {
 state.editCategory = null;
 render();
 });
 app.querySelectorAll<HTMLButtonElement>("[data-editcat]").forEach((b) => {
 b.addEventListener("click", () => {
 state.editCategory =
 state.categories.find((c) => c.id === Number(b.dataset.editcat)) || null;
 render();
 });
 });
 app.querySelectorAll<HTMLButtonElement>("[data-delcat]").forEach((b) => {
 b.addEventListener("click", async () => {
 try {
 await api.deleteCategory(Number(b.dataset.delcat));
 await refreshAll();
 toast("Category removed");
 render();
 } catch (e) {
 toast(String(e), true);
 }
 });
 });
}

/* HISTORY */
function viewHistory(): string {
 const items = state.history
 .map((h) => {
 const closed = h.status === "reviewed";
 return `
 <div class="history-item">
 <div style="flex:1">
 <div style="font-weight:800">${monthName(h.month)} ${h.year}</div>
 <div class="dim">
 ${closed ? "Closed: Check-In finished" : "Open: still editable"}
 ${h.closedAt ? `  -  finished ${esc(h.closedAt.slice(0, 10))}` : ""}
 </div>
 ${
 h.grade
 ? `<div class="mt-1"><span class="grade-pill" title="${esc(gradeExplain(h.grade))}">${esc(h.grade)}</span>
 <span class="dim" style="margin-left:0.5rem">${esc(gradeExplain(h.grade))}</span></div>`
 : `<div class="dim mt-1">No grade yet (finish Check-In to score this month).</div>`
 }
 </div>
 <div class="flex-gap" style="flex-direction:column;align-items:stretch">
 <button class="btn btn-primary btn-sm" data-open-m="${h.year}-${h.month}"
 title="Show this month on the Dashboard (same as using the month arrows)">View on Dashboard</button>
 ${
 closed
 ? `<button class="btn btn-ghost btn-sm" data-reopen="${h.year}-${h.month}"
 title="Unlock this month on the Dashboard so you can fix numbers, then run Check-In again">Reopen for editing</button>`
 : `<span class="dim" style="font-size:0.75rem;text-align:center">Already open</span>`
 }
 </div>
 </div>`;
 })
 .join("");

 return `
 <div class="page-header">
 <div>
 <h2>History</h2>
 <div class="sub">Past months and Check-In grades</div>
 </div>
 </div>
 <div class="info-callout mb-2">
 <strong>View on Dashboard</strong> only changes which month you're looking at (like the month selector).
 <strong>Reopen for editing</strong> unlocks a closed month and opens it on the Dashboard. Edit there, then run Check-In again when finished.
 <a href="#" data-goto-help="history">Full explanation in Help</a>
 </div>
 <div class="card">
 ${items || `<div class="empty">No months yet: open the Dashboard to create the current month, then use Check-In.</div>`}
 </div>`;
}

function bindHistory() {
 app.querySelectorAll<HTMLAnchorElement>("[data-goto-help]").forEach((a) => {
 a.addEventListener("click", (e) => {
 e.preventDefault();
 state.view = "help";
 state.helpQuery = a.dataset.gotoHelp || "history";
 render();
 });
 });
 app.querySelectorAll<HTMLButtonElement>("[data-open-m]").forEach((b) => {
 b.addEventListener("click", async () => {
 const [y, m] = (b.dataset.openM || "").split("-").map(Number);
 state.year = y;
 state.month = m;
 state.view = "dashboard";
 await reloadDash();
 });
 });
  app.querySelectorAll<HTMLButtonElement>("[data-reopen]").forEach((b) => {
    b.addEventListener("click", async () => {
      const [y, m] = (b.dataset.reopen || "").split("-").map(Number);
      if (
        !confirm(
          `Reopen ${monthName(m)} ${y}?\n\nThis unlocks the month so you can edit numbers on the Dashboard. The old grade is cleared. When you are done editing, open Check-In and run it again to save a new score.`
        )
      )
        return;
      try {
        await api.reopenMonth(y, m);
        state.year = y;
        state.month = m;
        state.view = "dashboard";
        state.checkinResult = null;
        await refreshAll();
        toast(
          `${monthName(m)} ${y} is open on the Dashboard. Edit as needed, then run Check-In when finished.`
        );
        render();
      } catch (e) {
        toast(String(e), true);
      }
    });
  });
}

/* HELP */
function viewHelp(): string {
 const sections = searchHelp(state.helpQuery);
 return `
 <div class="page-header">
 <div>
 <h2>Help</h2>
 <div class="sub">Plain-language guides: search anything</div>
 </div>
 </div>
 <div class="field mb-2" style="max-width:480px">
 <label>Search help</label>
 <input type="search" id="help-q" placeholder="e.g. fixed, check-in, grade, sync..." value="${esc(state.helpQuery)}" />
 </div>
 <div class="help-list">
 ${
 sections.length
 ? sections
 .map(
 (s) => `
 <article class="card card-pad mb-2" id="help-${s.id}">
 <h3 class="section-title">${esc(s.title)}</h3>
 <div class="help-body">${s.body}</div>
 </article>`
 )
 .join("")
 : `<div class="empty">No topics match "${esc(state.helpQuery)}".</div>`
 }
 </div>
 <p class="dim mt-1">${HELP_SECTIONS.length} topics  -  data never leaves this computer</p>`;
}

function bindHelp() {
  const input = app.querySelector<HTMLInputElement>("#help-q");
  input?.addEventListener("input", () => {
    state.helpQuery = input.value;
    render();
    const el = app.querySelector<HTMLInputElement>("#help-q");
    if (el) {
      el.focus();
      const len = el.value.length;
      el.setSelectionRange(len, len);
    }
  });
  // Scroll to focused section (from in-app help links using section ids)
  if (state.helpQuery) {
    const id = state.helpQuery.trim();
    const el =
      document.getElementById(`help-${id}`) ||
      document.querySelector(".help-list article");
    el?.scrollIntoView({ behavior: "smooth", block: "start" });
  }
}

/* SETTINGS */
function viewSettings(): string {
 const inc = state.income || {
 annualSalary: 0,
 taxBracket: 0,
 grossMonthly: 0,
 netMonthly: 0,
 biweeklyPay: 0,
 };
 return `
 <div class="page-header">
 <div><h2>Settings</h2><div class="sub">Income, security, backup</div></div>
 </div>
 <div class="card card-pad mb-2">
 <div class="section-title">Income</div>
 <p class="dim mb-1">Only <strong>Net monthly</strong> drives the Dashboard. Other fields are optional helpers.</p>
 <form id="inc-form" class="form-row">
 <div class="field"><label>Net monthly (primary)</label><input type="number" step="0.01" id="inc-net" value="${inc.netMonthly}" /></div>
 <div class="field"><label>Gross monthly</label><input type="number" step="0.01" id="inc-gross" value="${inc.grossMonthly}" /></div>
 <div class="field"><label>Bi-weekly pay</label><input type="number" step="0.01" id="inc-bi" value="${inc.biweeklyPay}" /></div>
 <div class="field"><label>Annual salary</label><input type="number" step="0.01" id="inc-ann" value="${inc.annualSalary}" /></div>
 <div class="field"><label>Tax bracket %</label><input type="number" step="0.1" id="inc-tax" value="${inc.taxBracket}" /></div>
 <button class="btn btn-success" type="submit">Save income</button>
 </form>
 </div>
 <div class="card card-pad mb-2">
 <div class="section-title">App Lock</div>
 <p class="dim mb-1">Optional passphrase for shared computers. No recovery if forgotten: export a backup first.</p>
 <div class="form-row">
 <div class="field"><label>Passphrase</label><input type="password" id="lock-new" autocomplete="new-password" /></div>
 <button class="btn btn-primary" id="lock-enable">${state.status?.lockEnabled ? "Update passphrase" : "Enable App Lock"}</button>
 ${state.status?.lockEnabled ? `<button class="btn btn-danger" id="lock-disable">Disable lock</button>` : ""}
 </div>
 </div>
    <div class="card card-pad mb-2">
      <div class="section-title">Backup and import</div>
      <div class="flex-gap">
        <button class="btn btn-primary" id="btn-export">Export JSON backup</button>
        <button class="btn btn-ghost" id="btn-import">Import data.json</button>
        <button class="btn btn-ghost" id="btn-demo">Load demo data</button>
        <input type="file" id="imp-file" accept=".json" class="hidden" />
      </div>
      <p class="dim mt-1">Database: <span class="mono">${esc(state.status?.dbPath || "")}</span></p>
    </div>
    <div class="card card-pad mb-2 danger-zone">
      <div class="section-title" style="color:var(--rose)">Clear all budget data</div>
      <p class="dim mb-1">
        This permanently deletes your plan, categories, monthly actuals, history, and income from this app.
        <strong>It cannot be undone.</strong> If you do not have an export backup, the data is gone forever.
      </p>
      <p class="dim mb-1">Export a backup first if you might need the data later.</p>
      <div class="field" style="max-width:320px">
        <label>Type DELETE DATA to enable the button</label>
        <input type="text" id="clear-confirm" placeholder="DELETE DATA" autocomplete="off" spellcheck="false" />
      </div>
      <button class="btn btn-danger mt-1" id="btn-clear-data" disabled>Clear database permanently</button>
    </div>
    <div class="card card-pad">
      <div class="section-title">About</div>
      <p>Budget Master 9000 v1.0: freeware, offline, private.</p>
      <p class="dim">MIT License. See Help for how everything works, including Security and privacy.</p>
    </div>`;
}

function bindSettings() {
 app.querySelector("#inc-form")?.addEventListener("submit", async (e) => {
 e.preventDefault();
 const income: IncomeSettings = {
 netMonthly:
 parseFloat((app.querySelector("#inc-net") as HTMLInputElement).value) ||
 0,
 grossMonthly:
 parseFloat(
 (app.querySelector("#inc-gross") as HTMLInputElement).value
 ) || 0,
 biweeklyPay:
 parseFloat((app.querySelector("#inc-bi") as HTMLInputElement).value) ||
 0,
 annualSalary:
 parseFloat((app.querySelector("#inc-ann") as HTMLInputElement).value) ||
 0,
 taxBracket:
 parseFloat((app.querySelector("#inc-tax") as HTMLInputElement).value) ||
 0,
 };
 try {
 await api.saveIncome(income);
 await api.updateMonthMeta(
 state.year,
 state.month,
 income.netMonthly,
 null,
 null
 );
 state.income = income;
 toast("Income saved");
 await refreshAll();
 } catch (err) {
 toast(String(err), true);
 }
 });
 app.querySelector("#lock-enable")?.addEventListener("click", async () => {
 const pw = (app.querySelector("#lock-new") as HTMLInputElement).value;
 try {
 await api.setAppLock(pw, true);
 state.status = await api.getStatus();
 toast("App Lock enabled");
 render();
 } catch (e) {
 toast(String(e), true);
 }
 });
 app.querySelector("#lock-disable")?.addEventListener("click", async () => {
 try {
 await api.setAppLock("", false);
 state.status = await api.getStatus();
 toast("App Lock disabled");
 render();
 } catch (e) {
 toast(String(e), true);
 }
 });
 app.querySelector("#btn-export")?.addEventListener("click", async () => {
 try {
 const json = await api.exportJson();
 const blob = new Blob([json], { type: "application/json" });
 const a = document.createElement("a");
 a.href = URL.createObjectURL(blob);
 a.download = `BM9000_backup_${new Date().toISOString().slice(0, 10)}.json`;
 a.click();
 toast("Backup downloaded");
 } catch (e) {
 toast(String(e), true);
 }
 });
 const file = app.querySelector<HTMLInputElement>("#imp-file")!;
 app.querySelector("#btn-import")?.addEventListener("click", () => file.click());
 file.addEventListener("change", async () => {
 const f = file.files?.[0];
 if (!f) return;
 try {
 const text = await f.text();
 const msg = await api.importLegacyJson(text);
 await refreshAll();
 try {
   const inc = await api.getIncome();
   if (inc.netMonthly > 0) {
     await api.updateMonthMeta(state.year, state.month, inc.netMonthly, null, null);
     await refreshAll();
   }
 } catch {
   /* ignore */
 }
 state.showCategoryNotice = true;
 state.view = "dashboard";
 toast(msg);
 } catch (e) {
 toast(String(e), true);
 }
 });
  app.querySelector("#btn-demo")?.addEventListener("click", async () => {
    if (!confirm("Load demo data? This replaces active budget lines.")) return;
    try {
      await api.loadDemoData();
      await refreshAll();
      toast("Demo loaded");
      render();
    } catch (e) {
      toast(String(e), true);
    }
  });

  const clearInput = app.querySelector<HTMLInputElement>("#clear-confirm");
  const clearBtn = app.querySelector<HTMLButtonElement>("#btn-clear-data");
  const syncClearBtn = () => {
    if (!clearBtn || !clearInput) return;
    clearBtn.disabled = clearInput.value.trim() !== "DELETE DATA";
  };
  clearInput?.addEventListener("input", syncClearBtn);
  syncClearBtn();
  clearBtn?.addEventListener("click", async () => {
    const phrase = clearInput?.value.trim() || "";
    if (phrase !== "DELETE DATA") {
      toast("Type DELETE DATA exactly to confirm.", true);
      return;
    }
    if (
      !confirm(
        "FINAL WARNING\n\nThis permanently erases all budget data in this app.\nWithout a backup file, recovery is impossible.\n\nContinue?"
      )
    )
      return;
    try {
      await api.clearAllData(phrase);
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
      return;
    } catch (e) {
      toast(String(e), true);
    }
  });
}

/* styles for new UI bits injected once */
const extraCss = document.createElement("style");
extraCss.textContent = `
.info-callout {
 background: var(--bg-elevated);
 border: 1px solid var(--border);
 border-left: 4px solid var(--primary);
 border-radius: 12px;
 padding: 0.85rem 1rem;
 font-size: 0.9rem;
 color: var(--text-muted);
 line-height: 1.5;
}
.info-callout a { color: var(--primary-hover); font-weight: 600; }
.guide-box {
 background: rgba(99, 102, 241, 0.08);
 border: 1px solid rgba(99, 102, 241, 0.25);
 border-radius: 12px;
 padding: 1rem 1.15rem;
 max-width: 720px;
}
.guide-title { font-weight: 800; margin-bottom: 0.4rem; color: var(--primary-hover); }
.guide-body { font-size: 0.92rem; color: var(--text-muted); line-height: 1.55; }
.money-input {
 display: flex; align-items: center; gap: 0.25rem;
 background: var(--bg); border: 1px solid var(--border); border-radius: 10px; padding: 0 0.5rem;
}
.money-input:focus-within { border-color: var(--primary); box-shadow: 0 0 0 3px rgba(99,102,241,0.2); }
.money-prefix { color: var(--text-dim); font-weight: 700; }
.money-input input {
 border: none !important; background: transparent !important; box-shadow: none !important;
 padding: 0.55rem 0.35rem; width: 100%; min-width: 5rem;
}
.notice-overlay {
 position: fixed; inset: 0; background: rgba(0,0,0,0.55); z-index: 200;
 display: grid; place-items: center; padding: 1.5rem;
}
.notice-card { max-width: 520px; width: 100%; padding: 1.5rem; }
.help-body { color: var(--text-muted); line-height: 1.55; font-size: 0.95rem; }
.help-body p { margin: 0 0 0.85rem; }
.help-body p:last-child { margin-bottom: 0; }
.help-body ul { margin: 0 0 0.85rem; padding-left: 1.25rem; }
.help-body li { margin-bottom: 0.5rem; }
.help-body li:last-child { margin-bottom: 0; }
.help-body strong { color: var(--text); }
.help-body code { font-family: var(--mono); font-size: 0.85em; background: var(--bg); padding: 0.1em 0.35em; border-radius: 4px; }
.help-body em { font-style: italic; }
.danger-zone {
  border-color: rgba(244, 63, 94, 0.45) !important;
  box-shadow: 0 0 0 1px rgba(244, 63, 94, 0.12);
}
.danger-zone .btn-danger:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
`;
document.head.appendChild(extraCss);

boot();
