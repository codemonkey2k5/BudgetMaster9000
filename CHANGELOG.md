# Changelog

## 1.3.2

1. Pace, runway, needs-attention, and on-track summaries cover flexible spending only (not fixed bills).

## 1.3.1

1. Pace and runway on the month overview cover flexible spending only.
2. Over-budget callouts list flexible items only.

## 1.3.0

### Fixes
- **Plan delete:** removing a Plan line now also removes it from the open month (Dashboard / Transactions) after sync. Previously the Plan list updated but the live month kept the stale line.
- **Plan edit:** Edit focuses the form at the top of the page so changes are obvious.

### Features
- **Dated transactions:** each Submit on Transactions stores a date (default today). Line totals stay correct: actual = sum of dated events (absolute Edit replaces events so math never diverges).
- **Reports tab:** dated transaction list with scopes (current month / all / by budget line), column sort, and print.
- **Extra reports:** by category, daily spend, fixed vs flexible, over-budget lines, month comparison.
- Schema migrates to v5 on first open (`line_transactions` event log). Dashboard / Close-Month / History math unchanged and stay consistent with line totals.

## 1.2.0

### Fixes
- Plan form label: **Budgeted amount ($)** (was Amount ($)).
- Plan **Budget lines** table uses available vertical space.
- History lists **closed months only**.
- Dashboard Fixed vs flexible: removed redundant **Fixed budget** line.
- **Update available** notice: more reliable GitHub check (retries, scans recent releases, re-checks when opening Settings and periodically in-session).

### Features
- Transactions **This month's entries**: clickable column headers sort (Details, Category, Status A–Z/Z–A; Budget, Actual, Used low–high/high–low).
- Plan **Budget lines**: clickable column headers sort (Name, Category A–Z/Z–A; Amount, Monthly low–high/high–low).

## 1.1.1

- **Installer fix:** Desktop shortcut is recreated on every install and upgrade. Previously PREINSTALL removed the icon, then POSTINSTALL only restored it if it still existed — which fails on upgrade because the old uninstall already deleted it.
- Start Menu shortcut behavior unchanged (always recreated). Explorer icon cache refresh retained.

## 1.1.0

### Distribution (same two choices as 1.0.0)

- **Installer** (NSIS setup / MSI): new install or upgrade in place; budget data stays in user AppData automatically.
- **Portable** (`BudgetMaster9000.exe`): single file; database created next to the exe. Upgrade by replacing the exe in the same folder — no data copy or config edits.

### App changes

- **Bug fix:** Updating an open month from Plan after category or budget-line edits no longer creates double dashboard entries (resync keys by plan line id and updates in place).
- **Transactions tab:** "This month's lines" moved off the Dashboard into its own tab with edit, delete, add-to-total, Multi-Entry, and Update from Plan.
- **Mid-month spending:** Add amounts to a line total any time while the month is open (e.g. Lunch $100 + $20 → $120).
- **Multi-Entry:** Pick a category (and line if needed), submit several amounts quickly; each amount rolls into the running total (no per-item history).
- **Close-Month:** Check-In renamed; same verify-and-close wizard when you are ready.
- **Dashboard:** Month command center with richer KPIs, charts, attention/wins, pace, cash runway, and history strip.
- **UI:** Full-app polish (layout, forms, charts, dark/light).
- Schema auto-migrates on first open after upgrade (no user steps).

## 1.0.0

- Windows desktop app with installer and portable single-file download
- Local SQLite database created automatically on first run
- Month-centric dashboard, Plan vs Actual, Monthly Check-In scorecard
- History of closed months, import/export backups, optional App Lock
- Built-in Help, Fixed vs Flexible categories, demo data option
