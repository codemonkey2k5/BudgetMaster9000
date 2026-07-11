/** Searchable in-app help content for beginners. Use plain ASCII only. */

export interface HelpSection {
  id: string;
  title: string;
  tags: string[];
  /** HTML body with <p>, <ul>, <li>, <strong>, <code>, <br/> only. */
  body: string;
}

export const HELP_SECTIONS: HelpSection[] = [
  {
    id: "overview",
    title: "What is Budget Master 9000?",
    tags: ["start", "beginner", "overview"],
    body: `
<p>This app helps you see how your month is going compared to a simple plan.</p>
<ul>
<li><strong>Plan:</strong> your recurring budget template (what you expect every month).</li>
<li><strong>Dashboard:</strong> one month at a time: planned amounts vs what you actually spent.</li>
<li><strong>Check-In:</strong> once a month, enter flexible spending and get a scorecard.</li>
<li><strong>History:</strong> past months you already closed, with grades.</li>
</ul>
<p>Your data stays on this computer. There is no cloud account.</p>`,
  },
  {
    id: "plan-vs-dashboard",
    title: "Why Plan and Dashboard look different",
    tags: ["plan", "dashboard", "sync", "month", "snapshot"],
    body: `
<p>The <strong>Plan</strong> is your master list of recurring costs (mortgage, groceries, Netflix, and so on).</p>
<p>When you open a month (for example June), the app takes a <strong>snapshot</strong> of the Plan for that month. That snapshot is what the Dashboard shows.</p>
<p>That way, if you change next month's Plan later, June's history stays accurate.</p>
<p>If you add or change lines on the Plan and want the <em>current open month</em> to pick them up, use <strong>Update this month from Plan</strong>. Closed (reviewed) months are not changed.</p>`,
  },
  {
    id: "sync",
    title: 'What "Update this month from Plan" does',
    tags: ["sync", "dashboard", "plan"],
    body: `
<p>It copies any <em>new</em> Plan lines into the month you are viewing (if that month is still open).</p>
<p>It does <strong>not</strong> rewrite a month you already finished with Check-In.</p>
<p>Use it after you add something new on the Plan (like a new subscription) and you want it to appear on this month's Dashboard.</p>`,
  },
  {
    id: "categories-fixed",
    title: "Categories: Fixed vs Flexible",
    tags: ["category", "categories", "fixed", "flexible", "bills"],
    body: `
<p>Each category is either:</p>
<ul>
<li><strong>Fixed:</strong> same every month (rent, insurance, subscriptions). Check-In skips these; they are assumed "on plan."</li>
<li><strong>Flexible:</strong> amounts change (food, fun, charging). Check-In asks you for actuals.</li>
</ul>
<p>When you create or edit a category, choose Fixed or Flexible there. Every budget line in that category follows that setting.</p>
<p>If you imported old data, review your categories once and mark bill categories as Fixed.</p>`,
  },
  {
    id: "checkin",
    title: "Monthly Check-In (step by step)",
    tags: ["check-in", "checkin", "actuals", "grade"],
    body: `
<p><strong>Step 1: Income</strong></p>
<p>Confirm your take-home pay for this month. Optionally add a short note (bonus, unpaid time off, etc.).</p>
<p><strong>Step 2: Flexible spending</strong></p>
<p>Only lines in Flexible categories appear. Enter what you actually spent. Numbers start at your Plan amount. Change only what was different.</p>
<p><strong>Step 3: Review and finish</strong></p>
<p>See totals, then complete Check-In. The app grades the month and saves it to History.</p>
<p>You can reopen a finished month later if you need to fix a number. After reopening, make your edits on the Dashboard, then run Check-In again when you are done.</p>`,
  },
  {
    id: "grades",
    title: "What the letter grade means",
    tags: ["grade", "history", "score", "A", "B", "C"],
    body: `
<p>After Check-In you get a letter grade (A, B+, B, and so on). It blends:</p>
<ul>
<li>How many dollars stayed on plan (weighted by size of each line), and</li>
<li>Your savings rate (income minus actual spending).</li>
</ul>
<p><strong>A</strong> = excellent. <strong>B</strong> range = good. <strong>C</strong> = mixed. <strong>D/F</strong> = needs a reset.</p>
<p>The scorecard also lists wins and things that need attention in plain language.</p>`,
  },
  {
    id: "history",
    title: "History: View month vs Reopen",
    tags: ["history", "reopen", "open", "reviewed"],
    body: `
<p><strong>View on Dashboard:</strong> jumps the Dashboard month selector to that month so you can look at the numbers. Same as changing the month on the Dashboard. Safe; does not change data.</p>
<p><strong>Reopen for editing:</strong> only for months marked Reviewed. Unlocks the month and opens it on the Dashboard so you can change numbers. When your edits are done, go to <strong>Check-In</strong> and run it again to save a new grade.</p>
<p>Why both? Viewing is for looking. Reopening is for fixing a closed month.</p>`,
  },
  {
    id: "income",
    title: "Income settings",
    tags: ["income", "settings", "net"],
    body: `
<p>Only <strong>Net monthly</strong> drives the Dashboard math (take-home pay).</p>
<p>Other fields (annual, tax %, bi-weekly) are optional notes/helpers. Edit them under Settings.</p>`,
  },
  {
    id: "import-export",
    title: "Import, export, backups, and clearing data",
    tags: ["import", "export", "backup", "json", "delete", "clear"],
    body: `
<p>Use Settings, then Export JSON to save a backup file.</p>
<p>Import can load that backup or an older BudgetMaster <code>data.json</code> file.</p>
<p>After importing old data, you will see a short reminder to review which categories are Fixed vs Flexible.</p>
<p><strong>Clear all budget data</strong> (under Settings) permanently erases plans, months, history, and income from this app's database. You must type <code>DELETE DATA</code> to confirm. There is no undo. If you do not have a backup file, the data cannot be recovered.</p>`,
  },
  {
    id: "security",
    title: "Security and privacy",
    tags: ["password", "lock", "security", "privacy", "encryption", "local", "offline"],
    body: `
<p><strong>Privacy first</strong></p>
<p>Budget Master 9000 is built to keep money data on your machine. There is no account sign-in, no cloud sync, and no advertising trackers in the app. Your numbers are not sent to a server for analysis.</p>
<p><strong>Where data lives</strong></p>
<p>All budget information is stored in a local database file on this computer. If you use the portable download, that file is created next to the app automatically. If someone can copy that file and open the app without App Lock, they can see your data.</p>
<p><strong>App Lock</strong></p>
<p>Optional App Lock asks for a passphrase when you open the app. It is meant to stop casual snooping on a shared PC (family, office). It is not the same as bank-level security. Choose a strong passphrase you will remember. There is no password reset by email. If you forget it, you need a backup you exported earlier, or you may lose access to locked data.</p>
<p><strong>What App Lock does not replace</strong></p>
<ul>
<li>Full-disk encryption such as BitLocker on Windows laptops</li>
<li>A good device password or Windows Hello</li>
<li>Safe storage of exported JSON backups (treat them like bank statements)</li>
<li>Protection against malware already running as you</li>
</ul>
<p><strong>Backups and clearing data</strong></p>
<p>Export backups before major changes. Clearing the database is permanent without a backup. Do not share backup files on public drives or chat apps unless they are protected.</p>
<p><strong>Updates and freeware</strong></p>
<p>Download the app only from sources you trust (for example your own GitHub release). The project is offline-first freeware: your privacy is part of the product, not a paid add-on.</p>`,
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
