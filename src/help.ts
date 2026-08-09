/** Searchable in-app help for first-time users. Plain language only. */

export interface HelpSection {
  id: string;
  title: string;
  tags: string[];
  /** HTML body with <p>, <ul>, <li>, <ol>, <strong>, <code>, <br/> only. */
  body: string;
}

export const HELP_SECTIONS: HelpSection[] = [
  {
    id: "overview",
    title: "Getting started",
    tags: ["start", "beginner", "overview", "first"],
    body: `
<p>Budget Master 9000 is a private desktop app for planning a monthly budget and tracking what you actually spend. Everything stays on your computer—no account, no cloud, no ads.</p>
<p>A simple monthly rhythm:</p>
<ol>
<li>Set your take-home pay under <strong>Settings</strong>.</li>
<li>Build your recurring budget under <strong>Plan</strong> (rent, groceries, subscriptions, and so on).</li>
<li>During the month, record spending under <strong>Transactions</strong> (each submit has a date).</li>
<li>Watch the big picture on the <strong>Dashboard</strong>.</li>
<li>Review dated lists and summaries under <strong>Reports</strong>.</li>
<li>When the month is over, use <strong>Close-Month</strong> to verify totals and save a grade to <strong>History</strong>.</li>
</ol>
<p>On first launch you can start blank, load demo data, or import a backup.</p>`,
  },
  {
    id: "dashboard",
    title: "Dashboard",
    tags: ["dashboard", "charts", "health", "overview", "month"],
    body: `
<p>The Dashboard is your month at a glance for the month shown at the top (use the arrows to change months).</p>
<ul>
<li><strong>Income, budgeted, actual, variance, and remaining cash</strong> summarize how the month is going.</li>
<li><strong>Charts</strong> show spending by category and how income compares to budget and actual.</li>
<li><strong>Pace and runway</strong> track flexible spending only (fixed bills are excluded): whether you are ahead or behind a steady pace, and flexible budget left per remaining day.</li>
<li><strong>Needs attention</strong> and <strong>On track</strong> list flexible lines only (over budget, or under/on plan). Fixed bills are excluded.</li>
</ul>
<p>To record spending, open <strong>Transactions</strong>. To finish the month, open <strong>Close-Month</strong>.</p>`,
  },
  {
    id: "plan",
    title: "Plan",
    tags: ["plan", "budget", "recurring", "template", "lines", "categories"],
    body: `
<p>The Plan is your list of recurring budget items—what you expect every month.</p>
<p>For each item, enter a name, category, amount, and how often it occurs (weekly, monthly, or yearly). The app converts everything to a monthly total for you.</p>
<p><strong>Categories</strong> group similar items and mark each group as:</p>
<ul>
<li><strong>Fixed</strong> — usually the same amount (housing, insurance, many subscriptions).</li>
<li><strong>Flexible</strong> — amounts that change (food, entertainment, gas).</li>
</ul>
<p>When you change the Plan, the current open month is updated automatically so new or renamed items appear where you track spending. <strong>Deleting</strong> a Plan line removes it from the Plan and from the open month. Months you already closed keep their original numbers.</p>
<p>Use <strong>Edit</strong> on a line to load it into the form at the top, change it, and save.</p>`,
  },
  {
    id: "transactions",
    title: "Transactions",
    tags: ["transactions", "spending", "actuals", "submit", "log", "date"],
    body: `
<p>Use Transactions to record what you spent during the month you have selected.</p>
<p><strong>Add to a total</strong></p>
<ol>
<li>Choose the budget item from the <strong>Line</strong> list.</li>
<li>Enter the amount you spent.</li>
<li>Confirm the <strong>Date</strong> (defaults to today).</li>
<li>Click <strong>Submit</strong>.</li>
</ol>
<p>Each submission adds to that item’s total for the month and stores a dated transaction for <strong>Reports</strong>. For example, if groceries already show $100 and you submit $20, the total becomes $120. The same line stays selected so you can enter several purchases in a row.</p>
<p>In the table you can <strong>Edit</strong> a total to set an exact amount (this replaces the line’s event list so totals always stay correct), or <strong>Delete</strong> an item from this month only if you do not want it counted here.</p>
<p>Switch months with the arrows at the top if you need to correct an earlier open month.</p>`,
  },
  {
    id: "reports",
    title: "Reports",
    tags: ["reports", "print", "dates", "list", "category", "daily"],
    body: `
<p>Reports shows dated transactions and summaries. Totals match the same math as the Dashboard for each line and month.</p>
<p><strong>Scope</strong></p>
<ul>
<li><strong>Current month</strong> — all dated events for the month shown in the app (same month as Dashboard / Transactions).</li>
<li><strong>All transactions</strong> — optional date range across months.</li>
<li><strong>By budget line</strong> — one line, optional date range.</li>
</ul>
<p><strong>Report types</strong></p>
<ul>
<li>Transaction list (sortable by date, name, type, category, amount, month)</li>
<li>By category · Daily spend · Fixed vs flexible · Over budget · Month comparison</li>
</ul>
<p>Use <strong>Print</strong> to print (or save as PDF) the report currently on screen.</p>`,
  },
  {
    id: "checkin",
    title: "Close-Month",
    tags: ["close-month", "close", "check-in", "checkin", "grade", "finish"],
    body: `
<p>Close-Month is the guided finish for a month. Use it when you are ready to lock numbers and save a score—not necessarily on the last calendar day, but when you are done adjusting.</p>
<p><strong>Step 1 — Income</strong><br/>Confirm take-home pay for that month. Add an optional note if something unusual happened.</p>
<p><strong>Step 2 — Flexible spending</strong><br/>Review items whose amounts can change. Fixed bills are already filled from your plan. Flexible amounts start from what you already recorded; change any that still need fixing.</p>
<p><strong>Step 3 — Review and finish</strong><br/>Check the totals, then close the month. You get a letter grade and short notes about what went well and what needs attention. That result is saved under History.</p>
<p>If you find a mistake later, reopen the month from History, fix it on Transactions or elsewhere, and run Close-Month again.</p>`,
  },
  {
    id: "grades",
    title: "Letter grades",
    tags: ["grade", "score", "A", "B", "C", "scorecard"],
    body: `
<p>After you close a month you receive a letter grade (A, B+, B, and so on). It combines:</p>
<ul>
<li>How closely actual spending matched your plan (larger items count more), and</li>
<li>Your savings rate (income minus what you spent).</li>
</ul>
<p><strong>A</strong> means an excellent month. <strong>B</strong> range means solid. <strong>C</strong> means mixed results. <strong>D</strong> or <strong>F</strong> means the month was far off plan—use the tips on the scorecard when you plan the next month.</p>`,
  },
  {
    id: "history",
    title: "History",
    tags: ["history", "past", "reopen", "closed", "reviewed"],
    body: `
<p>History lists months the app knows about, with status and grade when available.</p>
<ul>
<li><strong>View on Dashboard</strong> — jump to that month to look at charts and totals. This does not change your data.</li>
<li><strong>Reopen for editing</strong> — unlock a closed month so you can correct numbers. When you are finished, run Close-Month again to save a new grade.</li>
</ul>`,
  },
  {
    id: "income",
    title: "Income settings",
    tags: ["income", "settings", "net", "salary", "pay"],
    body: `
<p>Open <strong>Settings</strong> to set your income. Only <strong>Net monthly</strong> (take-home pay) drives the Dashboard math.</p>
<p>Annual salary, tax %, and bi-weekly pay are optional helpers for your own notes. They do not replace net monthly.</p>`,
  },
  {
    id: "import-export",
    title: "Backups, import, and clearing data",
    tags: ["import", "export", "backup", "json", "delete", "clear"],
    body: `
<p>Under Settings you can <strong>export</strong> a full backup as a JSON file. Store that file somewhere safe—it can restore your plan, months, and history.</p>
<p><strong>Import</strong> loads a backup you exported earlier, or an older Budget Master data file if you have one.</p>
<p><strong>Clear all budget data</strong> permanently erases your plan, months, history, and income from this app. You must type <code>DELETE DATA</code> to confirm. There is no undo unless you still have a backup file.</p>`,
  },
  {
    id: "install-upgrade",
    title: "Installing and upgrading",
    tags: ["install", "upgrade", "portable", "installer", "update", "version"],
    body: `
<p>There are two ways to run Budget Master 9000:</p>
<ul>
<li><strong>Installer</strong> — run the setup program. Open the app from the Start menu or desktop shortcut. Your data is stored in your Windows user folder so upgrades do not require moving files.</li>
<li><strong>Portable</strong> — a single <code>BudgetMaster9000.exe</code> you can put anywhere. Double-click to run. Your database file is created next to the program.</li>
</ul>
<p><strong>Upgrading the installer:</strong> download the new setup and run it. Your budget stays in place automatically.</p>
<p><strong>Upgrading portable:</strong> download the new program file and replace the old one in the same folder. Do not move or rename your database file.</p>
<p>Your version number appears at the bottom of the left menu. When a newer release is available online, an update notice appears there automatically. Click it to download the new package (with short upgrade instructions).</p>`,
  },
  {
    id: "security",
    title: "Privacy and security",
    tags: ["password", "lock", "security", "privacy", "local", "offline"],
    body: `
<p>Budget Master 9000 is built to keep your budget on this computer. There is no sign-in account, no cloud sync, and no advertising trackers. Your spending numbers are not uploaded for analysis.</p>
<p>Optional <strong>App Lock</strong> (under Settings) asks for a passphrase when you open the app. That helps on a shared computer. It is not a substitute for a strong Windows password or full-disk encryption such as BitLocker.</p>
<p>There is no email reset for App Lock. If you forget the passphrase, you will need a backup you exported earlier, or you may lose access to locked data.</p>
<p>Treat exported backup files like private documents. Do not share them in public places.</p>`,
  },
];

export function searchHelp(query: string): HelpSection[] {
  const q = query.trim().toLowerCase();
  if (!q) return HELP_SECTIONS;

  const byId = HELP_SECTIONS.find((s) => s.id.toLowerCase() === q);
  if (byId) return [byId];

  return HELP_SECTIONS.filter(
    (s) =>
      s.title.toLowerCase().includes(q) ||
      s.body.toLowerCase().includes(q) ||
      s.tags.some((t) => t.includes(q) || q.includes(t))
  );
}
