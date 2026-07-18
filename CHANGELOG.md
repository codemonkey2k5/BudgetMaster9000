# Changelog

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
