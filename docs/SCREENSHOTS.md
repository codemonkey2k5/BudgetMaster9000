# Screenshot inventory (for GitHub publish)

These files are the **canonical** marketing screenshots for README and releases.

## Location

| Path | Role |
|------|------|
| `docs/screenshots/` | Primary web-sized images (linked from README, ~1600px wide) |
| `docs/screenshots/full-res/` | Full maximized desktop captures (archive for marketing) |
| `docs/assets/screenshots/` | Mirror of web-sized images |
| `docs/assets/icon.png` | App icon for README header |
| `docs/screenshots/icon.png` | Same icon next to shots |

Captures are **maximized, DPI-aware, full-window** (not a top-left crop).

## Files

| File | Feature highlighted |
|------|---------------------|
| `01-dashboard.png` | Monthly health cards, charts, month context |
| `02-plan.png` | Plan editor, categories Fixed/Flexible |
| `03-checkin.png` | Guided monthly Check-In |
| `04-history.png` | Past months and grades |
| `05-help.png` | Searchable in-app help |
| `06-settings.png` | Income, App Lock, backup |

## Regenerate

1. Build a current release exe (`npm run release` or `npx tauri build`).
2. Run:

```bash
python scripts/capture_docs_screenshots.py
```

That seeds a clean demo database (no import modal), **maximizes** the window, captures with **DPI-aware full-frame** screenshots, writes web-sized PNGs (~1600px wide) into `docs/screenshots/`, archives full desktop resolution under `docs/screenshots/full-res/`, and mirrors web sizes to `docs/assets/screenshots/`.

## Notes

- Demo data is used so screenshots stay free of personal financial details.
- Do not commit real user `bm9000.db` or personal `data.json` to the public repo.
