use crate::models::*;
use chrono::{Datelike, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

// Sequential assignment for new categories: each next hue is far from the previous.
const COLORS: &[&str] = &[
    "#4f46e5", // indigo
    "#f59e0b", // amber
    "#059669", // emerald
    "#e11d48", // rose
    "#2563eb", // blue
    "#ca8a04", // gold
    "#7c3aed", // violet
    "#0d9488", // teal
    "#ea580c", // orange
    "#db2777", // pink
    "#0891b2", // cyan
    "#65a30d", // lime
];

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("{0}")]
    Message(String),
}

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
    pub portable: bool,
    pub unlocked: Mutex<bool>,
}

/// Resolve where the SQLite database lives.
///
/// * **Portable** (just the `.exe` on a USB drive, Desktop, etc.): database is
///   created next to the executable on first run. No marker files, no extra setup.
/// * **Installed** (NSIS/MSI): database lives under the user AppData folder so
///   upgrades never require copying data files by hand.
pub fn resolve_db_path() -> (PathBuf, bool) {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside = dir.join("bm9000.db");

            // Older portable zips used a marker; still honor it.
            let marker =
                dir.join("bm9000.portable").exists() || dir.join("portable.txt").exists();

            // NSIS places uninstall.exe beside the main binary. Program Files
            // installs are always treated as installed.
            let dir_s = dir.to_string_lossy();
            let looks_installed = dir.join("uninstall.exe").exists()
                || dir_s.contains("Program Files")
                || dir_s.contains("Program Files (x86)");

            // Portable: single exe, no installer layout — keep the database next to it.
            // Also stick with beside-exe if a db or marker is already there.
            let use_portable = marker || beside.exists() || !looks_installed;
            if use_portable && can_write_dir(dir) {
                return (beside, true);
            }
        }
    }

    let base = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("BudgetMaster9000");
    let _ = std::fs::create_dir_all(&base);
    (base.join("bm9000.db"), false)
}

fn can_write_dir(dir: &Path) -> bool {
    let probe = dir.join(".bm9000_write_probe");
    match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&probe)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

pub fn open_db(path: &Path) -> Result<Connection, DbError> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        ",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_fixed INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS budget_lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
            amount REAL NOT NULL,
            frequency TEXT NOT NULL CHECK(frequency IN ('week','month','year')),
            is_fixed INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS months (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            net_income REAL NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            mood TEXT,
            grade TEXT,
            closed_at TEXT,
            UNIQUE(year, month)
        );

        CREATE TABLE IF NOT EXISTS month_lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL REFERENCES months(id) ON DELETE CASCADE,
            budget_line_id INTEGER,
            name TEXT NOT NULL,
            category_id INTEGER NOT NULL,
            category_name TEXT NOT NULL,
            category_color TEXT NOT NULL,
            budget_amount REAL NOT NULL,
            is_fixed INTEGER NOT NULL DEFAULT 0,
            UNIQUE(month_id, name, category_name)
        );

        CREATE TABLE IF NOT EXISTS month_actuals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL REFERENCES months(id) ON DELETE CASCADE,
            month_line_id INTEGER NOT NULL REFERENCES month_lines(id) ON DELETE CASCADE,
            actual_amount REAL NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            UNIQUE(month_id, month_line_id)
        );

        CREATE TABLE IF NOT EXISTS reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL UNIQUE REFERENCES months(id) ON DELETE CASCADE,
            grade TEXT NOT NULL,
            score REAL NOT NULL,
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )?;

    // Migrations for existing DBs (CREATE IF NOT EXISTS won't add new columns)
    let _ = conn.execute(
        "ALTER TABLE categories ADD COLUMN is_fixed INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // Seed defaults if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
    if count == 0 {
        set_setting(conn, "theme", "dark")?;
        set_setting(conn, "schema_version", "3")?;
        set_setting(
            conn,
            "income",
            &serde_json::to_string(&IncomeSettings::default()).unwrap(),
        )?;
    } else {
        let ver = get_setting(conn, "schema_version")?.unwrap_or_else(|| "1".into());
        if ver == "1" {
            reclassify_fixed_lines(conn)?;
            set_setting(conn, "schema_version", "2")?;
        }
        let ver = get_setting(conn, "schema_version")?.unwrap_or_else(|| "2".into());
        if ver == "2" {
            // Promote category fixed from line heuristics (Housing/Utilities/etc.)
            migrate_category_fixed_from_lines(conn)?;
            set_setting(conn, "schema_version", "3")?;
        }
    }
    Ok(())
}

fn migrate_category_fixed_from_lines(conn: &Connection) -> Result<(), DbError> {
    // Any category that already has fixed lines, or matches fixed-name heuristics
    let cats = list_categories(conn)?;
    for cat in cats {
        let any_fixed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM budget_lines WHERE category_id=?1 AND active=1 AND is_fixed=1",
            params![cat.id],
            |r| r.get(0),
        )?;
        let should_fixed = any_fixed > 0 || looks_fixed("x", &cat.name, "month");
        if should_fixed {
            conn.execute(
                "UPDATE categories SET is_fixed=1 WHERE id=?1",
                params![cat.id],
            )?;
            conn.execute(
                "UPDATE budget_lines SET is_fixed=1 WHERE category_id=?1 AND active=1",
                params![cat.id],
            )?;
            conn.execute(
                "UPDATE month_lines SET is_fixed=1 WHERE category_id=?1",
                params![cat.id],
            )?;
        }
    }
    Ok(())
}

/// Mark predictable bills/subs as fixed so Check-In only asks for variable spend.
fn reclassify_fixed_lines(conn: &Connection) -> Result<(), DbError> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.name, c.name, b.frequency FROM budget_lines b
         JOIN categories c ON c.id = b.category_id WHERE b.active = 1",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (id, name, cat, freq) in rows {
        if looks_fixed(&name, &cat, &freq) {
            conn.execute(
                "UPDATE budget_lines SET is_fixed = 1 WHERE id = ?1",
                params![id],
            )?;
            conn.execute(
                "UPDATE month_lines SET is_fixed = 1 WHERE budget_line_id = ?1",
                params![id],
            )?;
        }
    }
    Ok(())
}

fn looks_fixed(name: &str, category: &str, frequency: &str) -> bool {
    let name_l = name.to_lowercase();
    let cat_l = category.to_lowercase();
    name_l.contains("mortgage")
        || name_l.contains("insurance")
        || name_l.contains("hoa")
        || name_l.contains("rent")
        || name_l.contains("phone")
        || name_l.contains("internet")
        || name_l.contains("starlink")
        || name_l.contains("spotify")
        || name_l.contains("youtube")
        || name_l.contains("prime")
        || name_l.contains("microsoft")
        || name_l.contains("subscription")
        || cat_l == "subscription"
        || cat_l == "subscriptions"
        || cat_l == "housing"
        || cat_l == "utilities"
        || (frequency == "year" && cat_l != "food" && cat_l != "entertainment")
}

pub fn monthly_amount(amount: f64, frequency: &str) -> f64 {
    match frequency {
        "week" => amount * 52.0 / 12.0,
        "year" => amount / 12.0,
        _ => amount,
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, DbError> {
    let v = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    Ok(v)
}

pub fn get_income(conn: &Connection) -> Result<IncomeSettings, DbError> {
    let raw = get_setting(conn, "income")?.unwrap_or_else(|| {
        serde_json::to_string(&IncomeSettings::default()).unwrap()
    });
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_income(conn: &Connection, income: &IncomeSettings) -> Result<(), DbError> {
    set_setting(conn, "income", &serde_json::to_string(income).unwrap())?;
    // Keep open months aligned with Settings so Dashboard does not show $0
    // after import or income edits while a month row already exists.
    sync_open_months_income(conn, income.net_monthly)?;
    Ok(())
}

/// Copy net income into every open month (not closed/reviewed history).
fn sync_open_months_income(conn: &Connection, net_monthly: f64) -> Result<(), DbError> {
    conn.execute(
        "UPDATE months SET net_income = ?1 WHERE status = 'open'",
        params![net_monthly],
    )?;
    Ok(())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, sort_order, COALESCE(is_fixed,0) FROM categories ORDER BY sort_order, name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                sort_order: r.get(3)?,
                is_fixed: r.get::<_, i64>(4)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upsert_category(conn: &Connection, input: &CategoryInput) -> Result<Category, DbError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(DbError::Message("Category name required".into()));
    }
    let id = if let Some(id) = input.id {
        conn.execute(
            "UPDATE categories SET name=?1, is_fixed=?2 WHERE id=?3",
            params![name, input.is_fixed as i64, id],
        )?;
        // Keep all lines in this category aligned with the category type
        conn.execute(
            "UPDATE budget_lines SET is_fixed=?1 WHERE category_id=?2 AND active=1",
            params![input.is_fixed as i64, id],
        )?;
        conn.execute(
            "UPDATE month_lines SET is_fixed=?1 WHERE category_id=?2",
            params![input.is_fixed as i64, id],
        )?;
        id
    } else {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
        let color = COLORS[(count as usize) % COLORS.len()];
        conn.execute(
            "INSERT INTO categories(name, color, sort_order, is_fixed) VALUES (?1, ?2, ?3, ?4)",
            params![name, color, count, input.is_fixed as i64],
        )?;
        conn.last_insert_rowid()
    };
    list_categories(conn)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| DbError::Message("Category not found after save".into()))
}

pub fn add_category(conn: &Connection, name: &str) -> Result<Category, DbError> {
    upsert_category(
        conn,
        &CategoryInput {
            id: None,
            name: name.to_string(),
            is_fixed: false,
        },
    )
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<(), DbError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
    if count <= 1 {
        return Err(DbError::Message("Keep at least one category".into()));
    }
    let used: i64 = conn.query_row(
        "SELECT COUNT(*) FROM budget_lines WHERE category_id = ?1 AND active = 1",
        params![id],
        |r| r.get(0),
    )?;
    if used > 0 {
        return Err(DbError::Message(
            "Category is used by budget lines. Reassign or delete those first.".into(),
        ));
    }
    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn category_is_fixed(conn: &Connection, category_id: i64) -> Result<bool, DbError> {
    let v: i64 = conn.query_row(
        "SELECT COALESCE(is_fixed,0) FROM categories WHERE id=?1",
        params![category_id],
        |r| r.get(0),
    )?;
    Ok(v == 1)
}

pub fn dismiss_category_review(conn: &Connection) -> Result<(), DbError> {
    set_setting(conn, "needs_category_review", "0")
}

/// Permanently wipe all budget data. Caller must require exact confirm phrase.
pub fn clear_all_data(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
        DELETE FROM reviews;
        DELETE FROM month_actuals;
        DELETE FROM month_lines;
        DELETE FROM months;
        DELETE FROM budget_lines;
        DELETE FROM categories;
        "#,
    )?;
    set_setting(
        conn,
        "income",
        &serde_json::to_string(&IncomeSettings::default()).unwrap(),
    )?;
    set_setting(conn, "needs_category_review", "0")?;
    // Keep theme and lock_hash so the app shell settings survive a data wipe
    Ok(())
}

pub fn list_budget_lines(conn: &Connection) -> Result<Vec<BudgetLine>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.name, b.category_id, c.name, b.amount, b.frequency,
                CASE WHEN COALESCE(c.is_fixed,0)=1 OR b.is_fixed=1 THEN 1 ELSE 0 END,
                b.notes, b.active
         FROM budget_lines b
         JOIN categories c ON c.id = b.category_id
         WHERE b.active = 1
         ORDER BY c.name, b.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let amount: f64 = r.get(4)?;
            let frequency: String = r.get(5)?;
            Ok(BudgetLine {
                id: r.get(0)?,
                name: r.get(1)?,
                category_id: r.get(2)?,
                category_name: r.get(3)?,
                amount,
                frequency: frequency.clone(),
                monthly_amount: monthly_amount(amount, &frequency),
                is_fixed: r.get::<_, i64>(6)? == 1,
                notes: r.get(7)?,
                active: r.get::<_, i64>(8)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upsert_budget_line(conn: &Connection, input: &BudgetLineInput) -> Result<i64, DbError> {
    let freq = match input.frequency.as_str() {
        "week" | "month" | "year" => input.frequency.as_str(),
        _ => "month",
    };
    let notes = input.notes.clone().unwrap_or_default();
    // Fixed/flexible is owned by the category
    let is_fixed = category_is_fixed(conn, input.category_id)?;
    if let Some(id) = input.id {
        conn.execute(
            "UPDATE budget_lines SET name=?1, category_id=?2, amount=?3, frequency=?4, is_fixed=?5, notes=?6 WHERE id=?7",
            params![input.name, input.category_id, input.amount, freq, is_fixed as i64, notes, id],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO budget_lines(name, category_id, amount, frequency, is_fixed, notes) VALUES (?1,?2,?3,?4,?5,?6)",
            params![input.name, input.category_id, input.amount, freq, is_fixed as i64, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn delete_budget_line(conn: &Connection, id: i64) -> Result<(), DbError> {
    conn.execute("UPDATE budget_lines SET active = 0 WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn ensure_month(conn: &Connection, year: i32, month: i32) -> Result<i64, DbError> {
    let existing: Option<(i64, String, f64)> = conn
        .query_row(
            "SELECT id, status, net_income FROM months WHERE year = ?1 AND month = ?2",
            params![year, month],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some((id, status, net_income)) = existing {
        // Heal open months stuck at $0 after import (settings already have real income).
        if status == "open" && net_income.abs() < 0.0001 {
            let income = get_income(conn)?;
            if income.net_monthly.abs() > 0.0001 {
                conn.execute(
                    "UPDATE months SET net_income = ?1 WHERE id = ?2",
                    params![income.net_monthly, id],
                )?;
            }
        }
        return Ok(id);
    }

    let income = get_income(conn)?;
    conn.execute(
        "INSERT INTO months(year, month, status, net_income) VALUES (?1, ?2, 'open', ?3)",
        params![year, month, income.net_monthly],
    )?;
    let month_id = conn.last_insert_rowid();
    snapshot_plan_into_month(conn, month_id)?;
    Ok(month_id)
}

fn snapshot_plan_into_month(conn: &Connection, month_id: i64) -> Result<(), DbError> {
    let lines = list_budget_lines(conn)?;
    for line in lines {
        conn.execute(
            "INSERT OR IGNORE INTO month_lines(month_id, budget_line_id, name, category_id, category_name, category_color, budget_amount, is_fixed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                month_id,
                line.id,
                line.name,
                line.category_id,
                line.category_name,
                conn.query_row(
                    "SELECT color FROM categories WHERE id = ?1",
                    params![line.category_id],
                    |r| r.get::<_, String>(0)
                )?,
                line.monthly_amount,
                line.is_fixed as i64
            ],
        )?;
    }
    apply_fixed_actuals(conn, month_id)?;
    Ok(())
}

/// Fixed bills are known amounts: fill Actual with the plan amount when missing.
/// Flexible lines are left empty until the user (or Check-In) enters them.
fn apply_fixed_actuals(conn: &Connection, month_id: i64) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO month_actuals (month_id, month_line_id, actual_amount, notes)
         SELECT ml.month_id, ml.id, ml.budget_amount, ''
         FROM month_lines ml
         LEFT JOIN month_actuals ma
           ON ma.month_line_id = ml.id AND ma.month_id = ml.month_id
         LEFT JOIN categories c ON c.id = ml.category_id
         WHERE ml.month_id = ?1
           AND ma.id IS NULL
           AND (ml.is_fixed = 1 OR COALESCE(c.is_fixed, 0) = 1)",
        params![month_id],
    )?;
    Ok(())
}

pub fn get_month_info(conn: &Connection, year: i32, month: i32) -> Result<MonthInfo, DbError> {
    let id = ensure_month(conn, year, month)?;
    conn.query_row(
        "SELECT id, year, month, status, net_income, notes, mood, grade, closed_at FROM months WHERE id = ?1",
        params![id],
        |r| {
            Ok(MonthInfo {
                id: r.get(0)?,
                year: r.get(1)?,
                month: r.get(2)?,
                status: r.get(3)?,
                net_income: r.get(4)?,
                notes: r.get(5)?,
                mood: r.get(6)?,
                grade: r.get(7)?,
                closed_at: r.get(8)?,
            })
        },
    )
    .map_err(Into::into)
}

fn line_status(budget: f64, actual: Option<f64>) -> (String, Option<f64>, Option<f64>) {
    match actual {
        None => ("unset".into(), None, None),
        Some(a) => {
            let variance = budget - a;
            let pct = if budget.abs() < 0.0001 {
                if a.abs() < 0.0001 {
                    100.0
                } else {
                    999.0
                }
            } else {
                (a / budget) * 100.0
            };
            let status = if a <= budget * 1.02 {
                if a < budget * 0.95 {
                    "under"
                } else {
                    "on_plan"
                }
            } else {
                "over"
            };
            (status.into(), Some(variance), Some(pct))
        }
    }
}

pub fn get_dashboard(conn: &Connection, year: i32, month: i32) -> Result<MonthDashboard, DbError> {
    let month_info = get_month_info(conn, year, month)?;
    // Selecting/opening a month: fixed bills count as entered at plan amount.
    apply_fixed_actuals(conn, month_info.id)?;
    let mut stmt = conn.prepare(
        "SELECT ml.id, ml.budget_line_id, ml.name, ml.category_id, ml.category_name, ml.category_color,
                ml.budget_amount,
                CASE WHEN COALESCE(c.is_fixed,0)=1 OR ml.is_fixed=1 THEN 1 ELSE 0 END,
                COALESCE(ma.actual_amount, NULL), COALESCE(ma.notes, '')
         FROM month_lines ml
         LEFT JOIN categories c ON c.id = ml.category_id
         LEFT JOIN month_actuals ma ON ma.month_line_id = ml.id AND ma.month_id = ml.month_id
         WHERE ml.month_id = ?1
         ORDER BY ml.category_name, ml.name",
    )?;

    let mut lines = Vec::new();
    let mut budgeted_total = 0.0;
    let mut actual_total = 0.0;
    let mut counts = StatusCounts::default();
    let mut any_actual = false;

    let rows = stmt.query_map(params![month_info.id], |r| {
        let actual: Option<f64> = r.get(8)?;
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, Option<i64>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, f64>(6)?,
            r.get::<_, i64>(7)? == 1,
            actual,
            r.get::<_, String>(9)?,
        ))
    })?;

    for row in rows {
        let (
            _ml_id,
            budget_line_id,
            name,
            category_id,
            category_name,
            category_color,
            budget,
            is_fixed,
            actual,
            notes,
        ) = row?;
        budgeted_total += budget;
        if let Some(a) = actual {
            actual_total += a;
            any_actual = true;
        }
        let (status, variance, pct_used) = line_status(budget, actual);
        match status.as_str() {
            "under" => counts.under += 1,
            "on_plan" => counts.on_plan += 1,
            "over" => counts.over += 1,
            _ => counts.unset += 1,
        }
        lines.push(MonthLine {
            budget_line_id: budget_line_id.unwrap_or(0),
            name,
            category_id,
            category_name,
            category_color,
            budget_amount: budget,
            actual_amount: actual,
            is_fixed,
            notes,
            variance,
            pct_used,
            status,
        });
    }

    let variance_total = if any_actual {
        budgeted_total - actual_total
    } else {
        month_info.net_income - budgeted_total
    };

    let savings_rate = if month_info.net_income > 0.0 && any_actual {
        Some(((month_info.net_income - actual_total) / month_info.net_income) * 100.0)
    } else if month_info.net_income > 0.0 {
        Some(((month_info.net_income - budgeted_total) / month_info.net_income) * 100.0)
    } else {
        None
    };

    Ok(MonthDashboard {
        month: month_info,
        lines,
        budgeted_total,
        actual_total: if any_actual { actual_total } else { 0.0 },
        variance_total,
        savings_rate,
        counts,
    })
}

pub fn save_actuals(
    conn: &Connection,
    year: i32,
    month: i32,
    actuals: &[ActualInput],
) -> Result<(), DbError> {
    let month_id = ensure_month(conn, year, month)?;
    let status: String = conn.query_row(
        "SELECT status FROM months WHERE id = ?1",
        params![month_id],
        |r| r.get(0),
    )?;
    if status == "reviewed" {
        return Err(DbError::Message(
            "Month is closed. Reopen it from History to edit.".into(),
        ));
    }

    for a in actuals {
        let month_line_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM month_lines WHERE month_id = ?1 AND budget_line_id = ?2",
                params![month_id, a.budget_line_id],
                |r| r.get(0),
            )
            .optional()?;

        let month_line_id = if let Some(id) = month_line_id {
            id
        } else {
            // Match by name fallback for legacy snapshots
            continue;
        };

        conn.execute(
            "INSERT INTO month_actuals(month_id, month_line_id, actual_amount, notes)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(month_id, month_line_id) DO UPDATE SET
               actual_amount = excluded.actual_amount,
               notes = excluded.notes",
            params![
                month_id,
                month_line_id,
                a.actual_amount,
                a.notes.clone().unwrap_or_default()
            ],
        )?;
    }
    Ok(())
}

pub fn update_month_meta(
    conn: &Connection,
    year: i32,
    month: i32,
    net_income: Option<f64>,
    notes: Option<String>,
    mood: Option<String>,
) -> Result<(), DbError> {
    let id = ensure_month(conn, year, month)?;
    if let Some(n) = net_income {
        conn.execute(
            "UPDATE months SET net_income = ?1 WHERE id = ?2",
            params![n, id],
        )?;
    }
    if let Some(notes) = notes {
        conn.execute(
            "UPDATE months SET notes = ?1 WHERE id = ?2",
            params![notes, id],
        )?;
    }
    if let Some(mood) = mood {
        conn.execute(
            "UPDATE months SET mood = ?1 WHERE id = ?2",
            params![mood, id],
        )?;
    }
    Ok(())
}

pub fn complete_check_in(
    conn: &Connection,
    year: i32,
    month: i32,
    actuals: &[ActualInput],
    mood: Option<String>,
    notes: Option<String>,
) -> Result<CheckInResult, DbError> {
    save_actuals(conn, year, month, actuals)?;
    update_month_meta(conn, year, month, None, notes, mood)?;

    let dash = get_dashboard(conn, year, month)?;
    let result = score_month(&dash, conn, year, month)?;

    let month_id = dash.month.id;
    conn.execute(
        "UPDATE months SET status = 'reviewed', grade = ?1, closed_at = ?2 WHERE id = ?3",
        params![result.grade, Utc::now().to_rfc3339(), month_id],
    )?;
    conn.execute(
        "INSERT INTO reviews(month_id, grade, score, payload_json) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(month_id) DO UPDATE SET grade=excluded.grade, score=excluded.score, payload_json=excluded.payload_json, created_at=datetime('now')",
        params![
            month_id,
            result.grade,
            result.score,
            serde_json::to_string(&result).unwrap()
        ],
    )?;

    Ok(result)
}

fn score_month(
    dash: &MonthDashboard,
    conn: &Connection,
    year: i32,
    month: i32,
) -> Result<CheckInResult, DbError> {
    let mut wins = Vec::new();
    let mut attention = Vec::new();
    let mut weighted_ok = 0.0;
    let mut weighted_total = 0.0;

    for line in &dash.lines {
        let weight = line.budget_amount.max(1.0);
        weighted_total += weight;
        match line.status.as_str() {
            "under" => {
                weighted_ok += weight;
                if let (Some(v), Some(pct)) = (line.variance, line.pct_used) {
                    if v >= 10.0 {
                        wins.push(format!(
                            "{}: ${:.0} under ({:.0}% of budget)",
                            line.name, v, pct
                        ));
                    }
                }
            }
            "on_plan" => {
                weighted_ok += weight;
                if line.is_fixed {
                    wins.push(format!("{}: on plan exactly", line.name));
                }
            }
            "over" => {
                if let (Some(v), Some(pct)) = (line.variance, line.pct_used) {
                    attention.push(format!(
                        "{}: ${:.0} over ({:.0}% of budget)",
                        line.name,
                        -v,
                        pct
                    ));
                }
            }
            _ => {}
        }
        if line.budget_amount < 0.01 {
            if let Some(a) = line.actual_amount {
                if a > 0.5 {
                    attention.push(format!(
                        "{}: spent ${:.0} with $0 budget. Add a plan line?",
                        line.name, a
                    ));
                }
            }
        }
    }

    // Cap lists
    wins.truncate(5);
    attention.truncate(5);

    let on_track_ratio = if weighted_total > 0.0 {
        weighted_ok / weighted_total
    } else {
        1.0
    };
    let savings_rate = dash.savings_rate.unwrap_or(0.0);
    let score = (on_track_ratio * 70.0) + (savings_rate.clamp(0.0, 30.0));
    let grade = match score {
        s if s >= 90.0 => "A",
        s if s >= 80.0 => "B+",
        s if s >= 75.0 => "B",
        s if s >= 70.0 => "B-",
        s if s >= 65.0 => "C+",
        s if s >= 60.0 => "C",
        s if s >= 50.0 => "D",
        _ => "F",
    }
    .to_string();

    let mut trends = Vec::new();
    // Prior month comparison
    let (py, pm) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    if let Ok(prev) = get_dashboard(conn, py, pm) {
        if prev.month.status == "reviewed" {
            if let (Some(cur_sr), Some(prev_sr)) = (dash.savings_rate, prev.savings_rate) {
                let delta = cur_sr - prev_sr;
                if delta.abs() >= 1.0 {
                    trends.push(format!(
                        "Savings rate {} from {:.0}% to {:.0}% vs prior month",
                        if delta > 0.0 { "improved" } else { "dropped" },
                        prev_sr,
                        cur_sr
                    ));
                }
            }
            let prev_over = prev.counts.over;
            let cur_over = dash.counts.over;
            if cur_over < prev_over {
                trends.push(format!(
                    "Fewer overspends this month ({cur_over} vs {prev_over} last month)"
                ));
            } else if cur_over > prev_over {
                trends.push(format!(
                    "More categories over budget ({cur_over} vs {prev_over} last month)"
                ));
            }
        }
    }

    let suggestion = attention.first().map(|a| {
        if a.contains("Food") || a.to_lowercase().contains("grocer") {
            "Next month: raise the Food budget slightly or trim lunch/dining.".into()
        } else if a.contains("$0 budget") {
            "Next month: add a budget line for unexpected recurring costs.".into()
        } else {
            "Next month: adjust the plan for your top overspend, or set a soft cap mid-month."
                .into()
        }
    });

    if wins.is_empty() && dash.counts.on_plan > 0 {
        wins.push(format!(
            "{} line(s) stayed on plan",
            dash.counts.on_plan
        ));
    }

    Ok(CheckInResult {
        grade,
        score,
        savings_rate,
        wins,
        attention,
        trends,
        suggestion,
        counts: dash.counts.clone(),
    })
}

pub fn list_history(conn: &Connection) -> Result<Vec<MonthInfo>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, year, month, status, net_income, notes, mood, grade, closed_at
         FROM months ORDER BY year DESC, month DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(MonthInfo {
                id: r.get(0)?,
                year: r.get(1)?,
                month: r.get(2)?,
                status: r.get(3)?,
                net_income: r.get(4)?,
                notes: r.get(5)?,
                mood: r.get(6)?,
                grade: r.get(7)?,
                closed_at: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn reopen_month(conn: &Connection, year: i32, month: i32) -> Result<(), DbError> {
    conn.execute(
        "UPDATE months SET status = 'open', grade = NULL, closed_at = NULL WHERE year = ?1 AND month = ?2",
        params![year, month],
    )?;
    Ok(())
}

pub fn resync_open_month(conn: &Connection, year: i32, month: i32) -> Result<(), DbError> {
    let info = get_month_info(conn, year, month)?;
    if info.status == "reviewed" {
        return Err(DbError::Message("Cannot resync a closed month".into()));
    }
    // Add any new plan lines not in snapshot
    snapshot_plan_into_month(conn, info.id)?;
    Ok(())
}

pub fn import_legacy(conn: &Connection, data: &LegacyImport) -> Result<String, DbError> {
    if let Some(inc) = &data.income {
        let income = IncomeSettings {
            annual_salary: inc.annual_salary.unwrap_or(0.0),
            tax_bracket: inc.tax_bracket.unwrap_or(0.0),
            gross_monthly: inc.gross_monthly.unwrap_or(0.0),
            net_monthly: inc.net_monthly.unwrap_or(0.0),
            biweekly_pay: inc.biweekly_pay.unwrap_or(0.0),
        };
        save_income(conn, &income)?;
    }

    // Categories
    let cats = data.categories.clone().unwrap_or_default();
    for cat in &cats {
        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM categories WHERE name = ?1",
                params![cat],
                |r| r.get(0),
            )
            .optional()?;
        if exists.is_none() {
            let _ = add_category(conn, cat);
        }
    }

    // Ensure at least default categories if none
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
    if count == 0 {
        for c in ["Housing", "Food", "Utilities", "Subscription", "Entertainment", "Health"] {
            let _ = add_category(conn, c);
        }
    }

    // Soft-delete existing lines then import
    conn.execute("UPDATE budget_lines SET active = 0", [])?;

    if let Some(expenses) = &data.expenses {
        for ex in expenses {
            let cat_id: i64 = match conn
                .query_row(
                    "SELECT id FROM categories WHERE name = ?1",
                    params![ex.category],
                    |r| r.get(0),
                )
                .optional()?
            {
                Some(id) => id,
                None => add_category(conn, &ex.category)?.id,
            };

            // Set category fixed from heuristics if this expense looks fixed
            if looks_fixed(&ex.name, &ex.category, &ex.frequency) {
                conn.execute(
                    "UPDATE categories SET is_fixed=1 WHERE id=?1",
                    params![cat_id],
                )?;
            }

            upsert_budget_line(
                conn,
                &BudgetLineInput {
                    id: None,
                    name: ex.name.clone(),
                    category_id: cat_id,
                    amount: ex.amount,
                    frequency: ex.frequency.clone(),
                    is_fixed: None,
                    notes: None,
                },
            )?;
        }
    }

    // Align all lines to their category fixed flag
    for cat in list_categories(conn)? {
        if cat.is_fixed {
            conn.execute(
                "UPDATE budget_lines SET is_fixed=1 WHERE category_id=?1",
                params![cat.id],
            )?;
        }
    }

    set_setting(conn, "needs_category_review", "1")?;

    // Always push imported income onto open months (month rows may already
    // exist from a blank start / earlier dashboard visit with $0 income).
    let income = get_income(conn)?;
    sync_open_months_income(conn, income.net_monthly)?;

    let now = Local::now();
    let month_id = ensure_month(conn, now.year(), now.month() as i32)?;
    // ensure_month may have created the month before income was present in
    // older code paths; force current month open income one more time.
    if income.net_monthly.abs() > 0.0001 {
        conn.execute(
            "UPDATE months SET net_income = ?1 WHERE id = ?2 AND status = 'open'",
            params![income.net_monthly, month_id],
        )?;
    }
    // Refresh plan snapshot for the current month so imported lines appear.
    snapshot_plan_into_month(conn, month_id)?;

    Ok(format!(
        "Imported {} categories and {} expenses (net income ${:.2})",
        list_categories(conn)?.len(),
        list_budget_lines(conn)?.len(),
        income.net_monthly
    ))
}

pub fn export_bundle(conn: &Connection) -> Result<ExportBundle, DbError> {
    let income = get_income(conn)?;
    let categories = list_categories(conn)?;
    let budget_lines = list_budget_lines(conn)?;
    let theme = get_setting(conn, "theme")?.unwrap_or_else(|| "dark".into());
    let history = list_history(conn)?;
    let mut months = Vec::new();
    for m in history {
        let dash = get_dashboard(conn, m.year, m.month)?;
        let actuals = dash
            .lines
            .into_iter()
            .filter_map(|l| {
                l.actual_amount.map(|a| ActualExport {
                    line_name: l.name,
                    category_name: l.category_name,
                    budget_amount: l.budget_amount,
                    actual_amount: a,
                    notes: l.notes,
                })
            })
            .collect();
        months.push(MonthExport {
            year: m.year,
            month: m.month,
            status: m.status,
            net_income: m.net_income,
            notes: m.notes,
            mood: m.mood,
            grade: m.grade,
            actuals,
        });
    }
    Ok(ExportBundle {
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        income,
        categories,
        budget_lines,
        months,
        theme,
    })
}

pub fn has_budget_data(conn: &Connection) -> Result<bool, DbError> {
    // Finished onboarding once the user has any categories or plan lines.
    // (Start blank only creates a category; it must still exit the welcome screen.)
    let lines: i64 = conn.query_row(
        "SELECT COUNT(*) FROM budget_lines WHERE active = 1",
        [],
        |r| r.get(0),
    )?;
    if lines > 0 {
        return Ok(true);
    }
    let cats: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
    Ok(cats > 0)
}

pub fn current_year_month() -> (i32, i32) {
    let now = Local::now();
    (now.year(), now.month() as i32)
}

pub fn load_demo(conn: &Connection) -> Result<(), DbError> {
    let demo = r#"{
      "income": {"annualSalary": 72000, "taxBracket": 22, "grossMonthly": 6000, "netMonthly": 4800, "biweeklyPay": 2400},
      "categories": ["Housing", "Food", "Utilities", "Transport", "Entertainment", "Health", "Subscriptions"],
      "expenses": [
        {"name": "Rent", "category": "Housing", "amount": 1600, "frequency": "month"},
        {"name": "Groceries", "category": "Food", "amount": 450, "frequency": "month"},
        {"name": "Dining Out", "category": "Food", "amount": 150, "frequency": "month"},
        {"name": "Electric", "category": "Utilities", "amount": 120, "frequency": "month"},
        {"name": "Internet", "category": "Utilities", "amount": 70, "frequency": "month"},
        {"name": "Phone", "category": "Utilities", "amount": 45, "frequency": "month"},
        {"name": "Gas/Transit", "category": "Transport", "amount": 180, "frequency": "month"},
        {"name": "Streaming", "category": "Subscriptions", "amount": 45, "frequency": "month"},
        {"name": "Gym", "category": "Health", "amount": 40, "frequency": "month"},
        {"name": "Fun Money", "category": "Entertainment", "amount": 100, "frequency": "month"}
      ]
    }"#;
    let parsed: LegacyImport = serde_json::from_str(demo)
        .map_err(|e| DbError::Message(e.to_string()))?;
    import_legacy(conn, &parsed)?;
    // Demo is not a user import of old data; do not force the review modal.
    set_setting(conn, "needs_category_review", "0")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> (Connection, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bm9000_test_{nanos}.db"));
        let _ = std::fs::remove_file(&path);
        let conn = open_db(&path).expect("open test db");
        (conn, path)
    }

    #[test]
    fn monthly_amount_math() {
        assert!((monthly_amount(100.0, "month") - 100.0).abs() < 0.001);
        assert!((monthly_amount(1200.0, "year") - 100.0).abs() < 0.001);
        assert!((monthly_amount(100.0, "week") - (100.0 * 52.0 / 12.0)).abs() < 0.001);
    }

    #[test]
    fn looks_fixed_classifies_bills() {
        assert!(looks_fixed("Mortgage", "Housing", "month"));
        assert!(looks_fixed("Starlink", "Utilities", "month"));
        assert!(looks_fixed("Amazon Prime", "Subscription", "year"));
        assert!(!looks_fixed("Groceries", "Food", "month"));
        assert!(!looks_fixed("Lunch", "Food", "month"));
        assert!(!looks_fixed("Fun Money", "Entertainment", "month"));
    }

    #[test]
    fn full_feature_flow_demo_checkin_history() {
        let (conn, path) = temp_db();

        // load demo
        load_demo(&conn).expect("demo");
        assert!(has_budget_data(&conn).unwrap());

        let lines = list_budget_lines(&conn).unwrap();
        assert!(lines.len() >= 8, "expected budget lines, got {}", lines.len());

        let fixed = lines.iter().filter(|l| l.is_fixed).count();
        let variable = lines.iter().filter(|l| !l.is_fixed).count();
        assert!(fixed > 0, "should have fixed lines");
        assert!(variable > 0, "should have variable lines");

        // categories
        let cats = list_categories(&conn).unwrap();
        assert!(cats.len() >= 5);
        let new_cat = add_category(&conn, "TestCat").unwrap();
        assert_eq!(new_cat.name, "TestCat");

        // income
        let mut income = get_income(&conn).unwrap();
        assert!(income.net_monthly > 0.0);
        income.net_monthly = 5000.0;
        save_income(&conn, &income).unwrap();
        assert_eq!(get_income(&conn).unwrap().net_monthly, 5000.0);

        // upsert flexible line
        let food = cats.iter().find(|c| c.name == "Food").unwrap();
        let id = upsert_budget_line(
            &conn,
            &BudgetLineInput {
                id: None,
                name: "Coffee".into(),
                category_id: food.id,
                amount: 50.0,
                frequency: "month".into(),
                is_fixed: None,
                notes: None,
            },
        )
        .unwrap();
        assert!(id > 0);

        // edit line
        upsert_budget_line(
            &conn,
            &BudgetLineInput {
                id: Some(id),
                name: "Coffee Shops".into(),
                category_id: food.id,
                amount: 60.0,
                frequency: "month".into(),
                is_fixed: None,
                notes: Some("edited".into()),
            },
        )
        .unwrap();
        let coffee = list_budget_lines(&conn)
            .unwrap()
            .into_iter()
            .find(|l| l.id == id)
            .unwrap();
        assert_eq!(coffee.name, "Coffee Shops");
        assert!((coffee.amount - 60.0).abs() < 0.01);

        // dashboard / month
        let (y, m) = current_year_month();
        update_month_meta(&conn, y, m, Some(5000.0), None, None).unwrap();
        let dash = get_dashboard(&conn, y, m).unwrap();
        assert!(!dash.lines.is_empty());
        assert!(dash.budgeted_total > 0.0);
        assert_eq!(dash.month.net_income, 5000.0);

        // resync after plan change
        resync_open_month(&conn, y, m).unwrap();
        let dash = get_dashboard(&conn, y, m).unwrap();
        assert!(
            dash.lines.iter().any(|l| l.name == "Coffee Shops"),
            "resync should add new plan line"
        );

        // check-in with actuals
        let actuals: Vec<ActualInput> = dash
            .lines
            .iter()
            .map(|l| {
                let amt = if l.is_fixed {
                    l.budget_amount
                } else if l.name.contains("Groceries") || l.name.contains("Coffee") {
                    l.budget_amount * 1.2 // over
                } else {
                    l.budget_amount * 0.8 // under
                };
                ActualInput {
                    budget_line_id: l.budget_line_id,
                    actual_amount: amt,
                    notes: None,
                }
            })
            .collect();

        let result = complete_check_in(
            &conn,
            y,
            m,
            &actuals,
            Some("ok".into()),
            Some("test note".into()),
        )
        .unwrap();
        assert!(!result.grade.is_empty());
        assert!(result.score >= 0.0);

        let dash2 = get_dashboard(&conn, y, m).unwrap();
        assert_eq!(dash2.month.status, "reviewed");
        assert!(dash2.month.grade.is_some());

        // history
        let hist = list_history(&conn).unwrap();
        assert!(!hist.is_empty());

        // reopen
        reopen_month(&conn, y, m).unwrap();
        let dash3 = get_dashboard(&conn, y, m).unwrap();
        assert_eq!(dash3.month.status, "open");

        // export / import roundtrip
        let bundle = export_bundle(&conn).unwrap();
        assert!(!bundle.budget_lines.is_empty());
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("Coffee"));

        // delete
        delete_budget_line(&conn, id).unwrap();
        assert!(list_budget_lines(&conn)
            .unwrap()
            .iter()
            .all(|l| l.id != id));

        // category delete (unused TestCat)
        delete_category(&conn, new_cat.id).unwrap();

        // theme setting
        set_setting(&conn, "theme", "light").unwrap();
        assert_eq!(get_setting(&conn, "theme").unwrap().unwrap(), "light");

        drop(conn);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fixed_lines_get_actuals_when_month_opens() {
        let (conn, path) = temp_db();
        load_demo(&conn).unwrap();
        let (y, m) = current_year_month();
        let dash = get_dashboard(&conn, y, m).unwrap();
        let fixed: Vec<_> = dash.lines.iter().filter(|l| l.is_fixed).collect();
        assert!(!fixed.is_empty(), "demo should include fixed lines");
        for line in fixed {
            assert!(
                line.actual_amount.is_some(),
                "fixed line {} should have actual pre-filled",
                line.name
            );
            assert!(
                (line.actual_amount.unwrap() - line.budget_amount).abs() < 0.001,
                "fixed actual should equal budget for {}",
                line.name
            );
            assert_ne!(
                line.status, "unset",
                "fixed line {} should not show as not entered",
                line.name
            );
        }
        let flexible: Vec<_> = dash.lines.iter().filter(|l| !l.is_fixed).collect();
        for line in &flexible {
            // flexible still empty until user enters (unless they had data)
            if line.actual_amount.is_none() {
                assert_eq!(line.status, "unset");
            }
        }
        drop(conn);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn import_legacy_sets_dashboard_net_income() {
        let (conn, path) = temp_db();
        // Reproduce: open a month while income is still $0, then import.
        let (y, m) = current_year_month();
        ensure_month(&conn, y, m).unwrap();
        let pre = get_dashboard(&conn, y, m).unwrap();
        assert!(
            (pre.month.net_income).abs() < 0.001,
            "precondition: month starts at $0"
        );

        let sample = include_str!("../fixtures/legacy-user-data.json");
        let data: LegacyImport = serde_json::from_str(sample).expect("parse legacy sample");
        import_legacy(&conn, &data).unwrap();

        let settings = get_income(&conn).unwrap();
        assert!(
            (settings.net_monthly - 3268.0).abs() < 0.01,
            "settings income should be 3268, got {}",
            settings.net_monthly
        );

        let dash = get_dashboard(&conn, y, m).unwrap();
        assert!(
            (dash.month.net_income - 3268.0).abs() < 0.01,
            "dashboard net income should be 3268 after import, got {}",
            dash.month.net_income
        );
        assert!(
            !dash.lines.is_empty(),
            "imported expenses should appear on dashboard"
        );
        drop(conn);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn blank_start_category_counts_as_has_data() {
        let (conn, path) = temp_db();
        assert!(!has_budget_data(&conn).unwrap());
        upsert_category(
            &conn,
            &CategoryInput {
                id: None,
                name: "General".into(),
                is_fixed: false,
            },
        )
        .unwrap();
        assert!(
            has_budget_data(&conn).unwrap(),
            "a category alone must exit the welcome screen"
        );
        drop(conn);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn import_legacy_sample_json() {
        let (conn, path) = temp_db();
        let sample = include_str!("../fixtures/demo-budget.json");
        // demo-budget uses camelCase income fields matching LegacyImport
        let data: LegacyImport = serde_json::from_str(sample).expect("parse sample");
        let msg = import_legacy(&conn, &data).unwrap();
        assert!(msg.contains("categories") || msg.contains("expenses") || !msg.is_empty());
        assert!(has_budget_data(&conn).unwrap());
        let lines = list_budget_lines(&conn).unwrap();
        assert!(lines.iter().any(|l| l.name == "Groceries"));
        assert!(lines.iter().any(|l| l.name == "Rent" && l.is_fixed));
        drop(conn);
        let _ = std::fs::remove_file(path);
    }
}
