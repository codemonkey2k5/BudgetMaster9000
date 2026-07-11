# Budget Master 9000

**Private. Offline. One app. Your budget, every month.**

Budget Master 9000 is a free, local-first monthly budget app for Windows. It helps you see this month’s budget health in seconds, then run a **Monthly Check-In** that scores where you followed the plan and where you slipped.

![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![Privacy](https://img.shields.io/badge/telemetry-none-success)

## Features

- **Month-centric dashboard** — income, budgeted, actual, variance, remaining cash
- **Budget vs actual** progress bars per line
- **Monthly Check-In wizard** — enter actuals, get a grade + plain-English scorecard
- **Plan editor** — recurring weekly / monthly / yearly lines (normalized to monthly)
- **History** — closed months with grades; reopen if needed
- **Import** classic `data.json` from the earlier HTML version
- **Export** full JSON backups
- **Optional App Lock** (Argon2 passphrase)
- **Dark / light theme**
- **No accounts, no cloud, no ads, no telemetry**

## Download (end users)

1. Grab the latest release `.exe` / installer from [GitHub Releases](../../releases)
2. Run it
3. On first launch: import your old `data.json`, load demo data, or start blank

### Portable mode (thumb drive)

Place a file named `bm9000.portable` or `portable.txt` next to the executable.  
The database `bm9000.db` will be created in the same folder.

Otherwise the database lives under your user AppData folder (`BudgetMaster9000`).

## Develop

### Requirements

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable
- Windows with WebView2 (preinstalled on Windows 10/11)

### Commands

```bash
npm install
npm run desktop    # tauri dev
npm run release    # production build
```

### Project layout

```
src/                 Frontend (TypeScript + Vite)
src-tauri/           Rust host + SQLite
legacy/              Original HTML/PowerShell prototype
samples/             Demo import files
```

## Migrating from the old HTML app

1. Launch Budget Master 9000  
2. Choose **Import data.json** (onboarding or Settings)  
3. Select your existing `data.json`  

Recurring expenses become **budget plan lines**. Use **Monthly Check-In** to record actuals going forward.

## Privacy & security

- All data is local SQLite
- Optional App Lock hashes your passphrase with Argon2id
- App Lock is **not** a substitute for full-disk encryption
- See [SECURITY.md](SECURITY.md)

## License

MIT — free for personal and commercial use. See [LICENSE](LICENSE).

## Credits

Built as freeware for people who want a simple monthly budget cockpit without a subscription.
