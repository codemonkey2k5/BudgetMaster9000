BudgetMaster9000 v2 - Automatic File Sync (thumb drive ready)
===============================================================

HOW TO USE (DO THIS)
--------------------
1. Copy the whole "BudgetMaster9000" folder to your thumb drive.
2. On any Windows PC, DOUBLE-CLICK **start.bat** (not index.html).
   - This uses built-in PowerShell (no install needed) to start a local server and open the app.
3. FIRST TIME on a new PC or new thumb drive location:
   - You will see a "Connect Folder" button/pill.
   - Click it and select the BudgetMaster9000 folder containing this file.
   - This is ONE-TIME per computer/location.
4. After connecting, every "Update Income", "Add", and Delete (✕) button automatically writes your data to data.json in the same folder.
5. Close the browser or the PowerShell window anytime — your work is safe in data.json.
6. Move the folder to another PC → double-click start.bat again. It usually remembers the folder.

DIRECTLY OPENING index.html
---------------------------
Still works (falls back to browser storage + manual Export/Import), but you will see a warning.
For real automatic saving to the file next to the HTML, always use start.bat.

FILES IN THIS FOLDER
--------------------
- start.bat     → Double-click this to run (recommended)
- start.ps1     → The actual launcher (PowerShell, native to Windows)
- index.html    → The budget app
- data.json     → Your live data file (created automatically)
- README.txt    → This file

NO PYTHON. NO EXTRA RUNTIMES. Everything is self-contained.

WHICH BUTTONS SAVE TO THE FILE
------------------------------
All the important ones now persist to data.json automatically when the app is launched via start.bat:
- Update Income
- Add expense
- Add category (+)
- Delete expense (✕)
- Delete category (✕)

Live updates + charts work exactly as before.

TROUBLESHOOTING
---------------
- "Not connected" pill appears → click it or the header "Connect Folder" button once.
- If the stored connection breaks (rare, e.g. after moving folder far away), just reconnect once.
- PowerShell policy warning: the .bat already uses Bypass for the local script.

Your data is now protected even if you edit for hours and forget everything. Close the browser with confidence.

2026 — Built for reliability on thumb drives.

DATA STORAGE & PORTABILITY
--------------------------
- Your data is **automatically saved** in the browser using localStorage.
- It loads instantly every time you open the page.
- Changes are live: add/edit/delete and the dashboard + charts update immediately.

To move your data to another computer or keep a backup next to the HTML file:
- Click the **"Export .json"** button.
- Save the downloaded file as `BudgetMaster9000_DATA.json` (or any name) **in the same folder** as index.html.
- On the new computer (or after clearing browser data), click **"Import .json"** and select that file.

This gives you a portable data file right next to your app.

LIBRARIES / FIRST-TIME INTERNET NOTE
------------------------------------
The page uses three small CDN libraries for the beautiful design and charts:
- Tailwind CSS
- Chart.js
- Lucide icons

On a brand new computer/browser profile, you need internet **once** so these can download and cache. 
After that, everything works 100% offline forever — even with no internet and from a thumb drive.

No other dependencies. No server needed. No login.

TIPS
----
- Fill in your Income Settings and click "Update Income".
- Add recurring expenses (weekly/monthly/yearly are normalized to monthly automatically).
- Categories are fully customizable.
- The two charts update live as you add expenses.
- Use the collapsible "Income Settings" section to save space.

Created 2026 — Enjoy!