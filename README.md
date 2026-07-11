<p align="center">
  <img src="docs/screenshots/icon.png" alt="Budget Master 9000" width="96" height="96">
</p>

<h1 align="center">Budget Master 9000</h1>

<p align="center">
  <strong>Your monthly budget, finally clear.</strong><br>
  Private. Offline. One Windows app. No accounts. No cloud. No guilt trip from a subscription.
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-lightgrey">
  <img alt="Privacy" src="https://img.shields.io/badge/telemetry-none-success">
  <img alt="Offline" src="https://img.shields.io/badge/offline-first-indigo">
</p>

---

## Why this exists

Most budget tools want your bank login, your email, and a monthly fee.  
**Budget Master 9000** wants none of that.

You get a calm desktop app that answers the only questions that matter:

- What did I *plan* to spend this month?
- What did I *actually* spend?
- Where did I do well, and where should I tighten up?

Built for people who like a simple plan, a monthly ritual, and data that never leaves their PC.

![Dashboard: monthly health, income, budget, charts](docs/screenshots/01-dashboard.png)

*See this month at a glance: income, budgeted totals, actuals, and high-contrast spending charts.*

---

## Features people actually use

| | |
|---|---|
| **Month-first dashboard** | Switch months, see health cards, progress bars, and category charts. |
| **Plan vs Actual** | Recurring plan is your template. Each month is a snapshot you can still update while open. |
| **Monthly Check-In** | A guided walkthrough for flexible spending only. Fixed bills auto-fill. Finish with a letter grade and plain-English tips. |
| **Fixed vs Flexible categories** | Mark housing and subscriptions Fixed. Mark food and fun Flexible. Check-In only asks about what changes. |
| **History** | Closed months keep their grades. Reopen only when you need to fix something. |
| **Import / export** | Bring in an older `data.json`, or export a full backup. |
| **Optional App Lock** | Passphrase gate for shared computers (Argon2). |
| **Portable mode** | Drop on a USB drive with `bm9000.portable` next to the exe. Database travels with you. |
| **Built-in Help** | Searchable guides written for humans, not accountants. |

---

## Screenshots

### Dashboard: this month’s story

![Dashboard overview](docs/screenshots/01-dashboard.png)

Net income, budgeted total, actuals, variance, remaining cash, and charts with high-contrast slices so categories stay easy to tell apart.

### Plan: set it once, reuse every month

![Plan editor](docs/screenshots/02-plan.png)

Add recurring lines (weekly, monthly, or yearly). Categories own Fixed vs Flexible so the whole plan stays consistent.

### Check-In: close the month with a real scorecard

![Monthly Check-In](docs/screenshots/03-checkin.png)

Step-by-step: confirm income, enter flexible spending, review totals, finish. Fixed bills are already treated as on plan.

### History: grades you can understand

![History of months](docs/screenshots/04-history.png)

Browse past months, jump to any month on the Dashboard, or reopen a closed month when you need to correct numbers (then run Check-In again).

### Help: answers in plain language

![In-app Help](docs/screenshots/05-help.png)

Search topics like Fixed vs Flexible, Plan vs Dashboard, grades, import, and security.

### Settings: income, lock, backup, and a safe reset

![Settings](docs/screenshots/06-settings.png)

Net monthly income drives the math. Optional App Lock. Export backups. Clear data only if you type `DELETE DATA`.

---

## Download

1. Get the latest **Windows** release from [GitHub Releases](../../releases)  
   (portable `.exe` or installer)
2. Run it
3. First launch: **Import** your old file, **Load demo**, or **Start blank**

### Portable (thumb drive)

Put a file named `bm9000.portable` (or `portable.txt`) next to the executable.  
Your database `bm9000.db` is created in that same folder.

Without that marker, data lives under your user AppData folder (`BudgetMaster9000`).

---

## Privacy, on purpose

- **No accounts**
- **No cloud sync**
- **No ads**
- **No telemetry**

Everything is local SQLite. Optional App Lock uses Argon2id hashing for a passphrase gate. That is for casual privacy on a shared PC, not a substitute for full-disk encryption (BitLocker) or good device hygiene. Export backups before major changes. Clearing the database is permanent without a backup.

---

## How a typical month looks

1. **Plan** once (or import last year’s file).
2. Live the month.
3. Open **Check-In**, enter flexible actuals, finish.
4. Read the grade and the “needs attention” list.
5. Adjust next month’s Plan if something consistently overruns.

Fixed bills (rent, insurance, streaming) fill themselves on the Dashboard so they do not show as “not entered.”

---

## Develop

### Requirements

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable
- Windows with WebView2 (included on modern Windows 10/11)

### Commands

```bash
npm install
npm run desktop    # tauri dev
npm run release    # production build (exe + installer)
```

### Useful scripts

| Script | Purpose |
|--------|---------|
| `scripts/capture_docs_screenshots.py` | Re-capture README screenshots into `docs/screenshots/` |
| `scripts/check_load.py` | Smoke-test that the portable exe serves the UI |
| `npm run release` then copy from `src-tauri/target/release/` | Ship a fresh binary |

### Project layout

```
src/                 Frontend (TypeScript + Vite)
src-tauri/           Rust host, SQLite, App Lock
docs/screenshots/    README images (canonical)
docs/assets/         Mirror of assets for packaging
samples/             Demo / import samples
legacy/              Original HTML prototype (reference only)
release/portable/    Local portable package for testing
```

### Tests

```bash
cd src-tauri
cargo test --lib
```

There is also a full UI self-test path used in development (`BM9000_UI_SELFTEST=1`).

---

## Screenshot assets (for publishing)

Canonical files for GitHub live here:

```
docs/screenshots/
  icon.png
  01-dashboard.png
  02-plan.png
  03-checkin.png
  04-history.png
  05-help.png
  06-settings.png
```

A copy is also kept under `docs/assets/screenshots/` for convenience.  
Regenerate anytime:

```bash
python scripts/capture_docs_screenshots.py
```

---

## License

[MIT](LICENSE). Free for personal and commercial use.

---

<p align="center">
  <em>Budget Master 9000: private by default, useful every month.</em>
</p>
