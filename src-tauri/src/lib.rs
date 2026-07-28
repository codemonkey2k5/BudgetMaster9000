mod acl_check;
mod crypto;
mod db;
mod models;
/// Public so the local `bm9000-update-tester` binary can call the same logic as the app.
pub mod update;

pub use models::UpdateCheck;

use db::{DbError, DbState};
use models::*;
use std::sync::Mutex;
use tauri::{Manager, State, WebviewUrl};
use tauri::webview::WebviewWindowBuilder;

fn require_unlocked(state: &DbState) -> Result<(), String> {
    let lock_enabled = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::get_setting(&conn, "lock_hash")
            .map_err(|e| e.to_string())?
            .is_some()
    };
    if !lock_enabled {
        return Ok(());
    }
    let unlocked = *state.unlocked.lock().map_err(|e| e.to_string())?;
    if unlocked {
        Ok(())
    } else {
        Err("App is locked. Unlock first.".into())
    }
}

fn map_err(e: DbError) -> String {
    e.to_string()
}

#[tauri::command]
fn get_status(state: State<DbState>) -> Result<AppStatus, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let lock_hash = db::get_setting(&conn, "lock_hash").map_err(map_err)?;
    let lock_enabled = lock_hash.is_some();
    let unlocked = *state.unlocked.lock().map_err(|e| e.to_string())?;
    let theme = db::get_setting(&conn, "theme")
        .map_err(map_err)?
        .unwrap_or_else(|| "dark".into());
    let has_data = db::has_budget_data(&conn).map_err(map_err)?;
    let needs_category_review = db::get_setting(&conn, "needs_category_review")
        .map_err(map_err)?
        .map(|v| v == "1")
        .unwrap_or(false);
    Ok(AppStatus {
        locked: lock_enabled && !unlocked,
        lock_enabled,
        db_path: state.path.display().to_string(),
        portable: state.portable,
        has_data,
        theme,
        needs_category_review,
    })
}

#[tauri::command]
fn unlock(state: State<DbState>, password: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let hash = db::get_setting(&conn, "lock_hash")
        .map_err(map_err)?
        .ok_or_else(|| "Lock is not enabled".to_string())?;
    let ok = crypto::verify_password(&password, &hash).map_err(|e| e.to_string())?;
    if ok {
        *state.unlocked.lock().map_err(|e| e.to_string())? = true;
    }
    Ok(ok)
}

#[tauri::command]
fn lock_app(state: State<DbState>) -> Result<(), String> {
    *state.unlocked.lock().map_err(|e| e.to_string())? = false;
    Ok(())
}

#[tauri::command]
fn set_app_lock(state: State<DbState>, password: String, enable: bool) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if enable {
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".into());
        }
        let hash = crypto::hash_password(&password).map_err(|e| e.to_string())?;
        db::set_setting(&conn, "lock_hash", &hash).map_err(map_err)?;
        *state.unlocked.lock().map_err(|e| e.to_string())? = true;
    } else {
        // Require current password already verified by being unlocked if lock was on
        conn.execute("DELETE FROM settings WHERE key = 'lock_hash'", [])
            .map_err(|e| e.to_string())?;
        *state.unlocked.lock().map_err(|e| e.to_string())? = true;
    }
    Ok(())
}

#[tauri::command]
fn set_theme(state: State<DbState>, theme: String) -> Result<(), String> {
    // Allow theme even when locked for unlock screen styling
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let t = if theme == "light" { "light" } else { "dark" };
    db::set_setting(&conn, "theme", t).map_err(map_err)
}

#[tauri::command]
fn get_income(state: State<DbState>) -> Result<IncomeSettings, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_income(&conn).map_err(map_err)
}

#[tauri::command]
fn save_income(state: State<DbState>, income: IncomeSettings) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::save_income(&conn, &income).map_err(map_err)
}

#[tauri::command]
fn list_categories(state: State<DbState>) -> Result<Vec<Category>, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_categories(&conn).map_err(map_err)
}

#[tauri::command]
fn add_category(state: State<DbState>, name: String) -> Result<Category, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::add_category(&conn, &name).map_err(map_err)
}

#[tauri::command]
fn upsert_category(state: State<DbState>, category: CategoryInput) -> Result<Category, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::upsert_category(&conn, &category).map_err(map_err)
}

#[tauri::command]
fn delete_category(state: State<DbState>, id: i64) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_category(&conn, id).map_err(map_err)
}

#[tauri::command]
fn dismiss_category_review(state: State<DbState>) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::dismiss_category_review(&conn).map_err(map_err)
}

#[tauri::command]
fn clear_all_data(state: State<DbState>, confirm_phrase: String) -> Result<(), String> {
    require_unlocked(&state)?;
    if confirm_phrase.trim() != "DELETE DATA" {
        return Err("Type DELETE DATA exactly to confirm.".into());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::clear_all_data(&conn).map_err(map_err)
}

#[tauri::command]
fn list_budget_lines(state: State<DbState>) -> Result<Vec<BudgetLine>, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_budget_lines(&conn).map_err(map_err)
}

#[tauri::command]
fn upsert_budget_line(state: State<DbState>, line: BudgetLineInput) -> Result<i64, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::upsert_budget_line(&conn, &line).map_err(map_err)
}

#[tauri::command]
fn delete_budget_line(state: State<DbState>, id: i64) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_budget_line(&conn, id).map_err(map_err)
}

#[tauri::command]
fn get_dashboard(state: State<DbState>, year: i32, month: i32) -> Result<MonthDashboard, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_dashboard(&conn, year, month).map_err(map_err)
}

#[tauri::command]
fn save_actuals(
    state: State<DbState>,
    year: i32,
    month: i32,
    actuals: Vec<ActualInput>,
) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::save_actuals(&conn, year, month, &actuals).map_err(map_err)
}

#[tauri::command]
fn update_month_meta(
    state: State<DbState>,
    year: i32,
    month: i32,
    net_income: Option<f64>,
    notes: Option<String>,
    mood: Option<String>,
) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_month_meta(&conn, year, month, net_income, notes, mood).map_err(map_err)
}

#[tauri::command]
fn complete_check_in(
    state: State<DbState>,
    year: i32,
    month: i32,
    actuals: Vec<ActualInput>,
    mood: Option<String>,
    notes: Option<String>,
) -> Result<CheckInResult, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::complete_check_in(&conn, year, month, &actuals, mood, notes).map_err(map_err)
}

#[tauri::command]
fn list_history(state: State<DbState>) -> Result<Vec<MonthInfo>, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_history(&conn).map_err(map_err)
}

#[tauri::command]
fn reopen_month(state: State<DbState>, year: i32, month: i32) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::reopen_month(&conn, year, month).map_err(map_err)
}

#[tauri::command]
fn resync_month(state: State<DbState>, year: i32, month: i32) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::resync_open_month(&conn, year, month).map_err(map_err)
}

#[tauri::command]
fn add_to_actual(
    state: State<DbState>,
    year: i32,
    month: i32,
    budget_line_id: i64,
    amount: f64,
    notes: Option<String>,
    occurred_on: Option<String>,
) -> Result<f64, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::add_to_actual(
        &conn,
        year,
        month,
        budget_line_id,
        amount,
        notes,
        occurred_on,
    )
    .map_err(map_err)
}

#[tauri::command]
fn list_line_transactions(
    state: State<DbState>,
    year: Option<i32>,
    month: Option<i32>,
    budget_line_id: Option<i64>,
    from_date: Option<String>,
    to_date: Option<String>,
    category_id: Option<i64>,
) -> Result<Vec<LineTransaction>, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_line_transactions(
        &conn,
        year,
        month,
        budget_line_id,
        from_date,
        to_date,
        category_id,
    )
    .map_err(map_err)
}

#[tauri::command]
fn delete_month_line(
    state: State<DbState>,
    year: i32,
    month: i32,
    budget_line_id: i64,
) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_month_line(&conn, year, month, budget_line_id).map_err(map_err)
}

#[tauri::command]
fn import_legacy_json(state: State<DbState>, json: String) -> Result<String, String> {
    require_unlocked(&state)?;
    let data: LegacyImport = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::import_legacy(&conn, &data).map_err(map_err)
}

#[tauri::command]
fn export_json(state: State<DbState>) -> Result<String, String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let bundle = db::export_bundle(&conn).map_err(map_err)?;
    serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_demo_data(state: State<DbState>) -> Result<(), String> {
    require_unlocked(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::load_demo(&conn).map_err(map_err)
}

#[tauri::command]
fn current_month() -> (i32, i32) {
    db::current_year_month()
}

#[tauri::command]
fn check_for_update() -> Result<UpdateCheck, String> {
    // Allowed while locked: version banner only; no budget data.
    Ok(update::check_for_update())
}

#[tauri::command]
fn download_update_package() -> Result<String, String> {
    // Network download to the user's Downloads folder.
    update::download_update_package()
}

/// Frontend self-test writes results next to the executable.
#[tauri::command]
fn report_ui_selftest(ok: bool, report: String) -> Result<String, String> {
    let path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "no exe parent".to_string())?
        .join("ui-selftest-result.txt");
    let body = format!(
        "{}\n{}\n",
        if ok { "UI_SELFTEST PASS" } else { "UI_SELFTEST FAIL" },
        report
    );
    std::fs::write(&path, &body).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// CLI: seed a demo database at the given path (for tests / portable setup).
pub fn seed_demo_at(path: &std::path::Path) -> Result<String, String> {
    let conn = db::open_db(path).map_err(|e| e.to_string())?;
    db::load_demo(&conn).map_err(|e| e.to_string())?;
    let n = db::list_budget_lines(&conn).map_err(|e| e.to_string())?.len();
    Ok(format!("Seeded demo database at {} with {n} budget lines", path.display()))
}

/// CLI: full backend feature self-test (no UI).
pub fn run_self_test() -> Result<String, String> {
    use db::*;
    use models::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bm9000_selftest_{nanos}.db"));
    let _ = std::fs::remove_file(&path);
    let conn = open_db(&path).map_err(|e| e.to_string())?;

    load_demo(&conn).map_err(|e| e.to_string())?;
    let lines = list_budget_lines(&conn).map_err(|e| e.to_string())?;
    if lines.is_empty() {
        return Err("no lines after demo".into());
    }
    let cats = list_categories(&conn).map_err(|e| e.to_string())?;
    if cats.is_empty() {
        return Err("no categories".into());
    }
    let income = get_income(&conn).map_err(|e| e.to_string())?;
    if income.net_monthly <= 0.0 {
        return Err("income missing".into());
    }

    let food = cats
        .iter()
        .find(|c| c.name == "Food")
        .ok_or("Food category missing")?;
    let id = upsert_budget_line(
        &conn,
        &BudgetLineInput {
            id: None,
            name: "SelfTest Snack".into(),
            category_id: food.id,
            amount: 25.0,
            frequency: "month".into(),
            is_fixed: None,
            notes: None,
        },
    )
    .map_err(|e| e.to_string())?;

    let (y, m) = current_year_month();
    update_month_meta(&conn, y, m, Some(income.net_monthly), Some("selftest".into()), Some("ok".into()))
        .map_err(|e| e.to_string())?;
    resync_open_month(&conn, y, m).map_err(|e| e.to_string())?;
    let dash = get_dashboard(&conn, y, m).map_err(|e| e.to_string())?;
    if !dash.lines.iter().any(|l| l.name == "SelfTest Snack") {
        return Err("resync failed to add line".into());
    }

    // Mid-month add (must be ACL-allowed in production builds)
    let snack = dash
        .lines
        .iter()
        .find(|l| l.name == "SelfTest Snack")
        .ok_or("snack line missing")?;
    let t1 = add_to_actual(&conn, y, m, snack.budget_line_id, 20.0, None, None)
        .map_err(|e| e.to_string())?;
    if (t1 - 20.0).abs() > 0.01 {
        return Err(format!("add_to_actual expected 20, got {t1}"));
    }
    let t2 = add_to_actual(&conn, y, m, snack.budget_line_id, 5.0, None, None)
        .map_err(|e| e.to_string())?;
    if (t2 - 25.0).abs() > 0.01 {
        return Err(format!("add_to_actual expected 25, got {t2}"));
    }

    let dash = get_dashboard(&conn, y, m).map_err(|e| e.to_string())?;
    let actuals: Vec<ActualInput> = dash
        .lines
        .iter()
        .map(|l| ActualInput {
            budget_line_id: l.budget_line_id,
            actual_amount: if l.is_fixed {
                l.budget_amount
            } else {
                l.budget_amount * 0.9
            },
            notes: None,
        })
        .collect();
    let result = complete_check_in(&conn, y, m, &actuals, Some("easy".into()), Some("ok".into()))
        .map_err(|e| e.to_string())?;
    if result.grade.is_empty() {
        return Err("empty grade".into());
    }

    let hist = list_history(&conn).map_err(|e| e.to_string())?;
    if hist.is_empty() {
        return Err("empty history".into());
    }
    reopen_month(&conn, y, m).map_err(|e| e.to_string())?;

    let bundle = export_bundle(&conn).map_err(|e| e.to_string())?;
    if bundle.budget_lines.is_empty() {
        return Err("export empty".into());
    }

    delete_budget_line(&conn, id).map_err(|e| e.to_string())?;
    set_setting(&conn, "theme", "dark").map_err(|e| e.to_string())?;

    // Import legacy-shaped JSON
    let legacy = r#"{"income":{"netMonthly":1000},"categories":["X"],"expenses":[{"name":"Y","category":"X","amount":10,"frequency":"month"}]}"#;
    let data: LegacyImport = serde_json::from_str(legacy).map_err(|e| e.to_string())?;
    // Don't wipe user's mental model hard — this is temp db
    import_legacy(&conn, &data).map_err(|e| e.to_string())?;

    drop(conn);
    let _ = std::fs::remove_file(&path);

    // Validate ACL permission file lists critical + new commands
    let perm = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("permissions/app.toml");
    if !perm.exists() {
        return Err("permissions/app.toml missing".into());
    }
    let perm_txt = std::fs::read_to_string(&perm).map_err(|e| e.to_string())?;
    for cmd in [
        "current_month",
        "get_status",
        "get_dashboard",
        "complete_check_in",
        "load_demo_data",
        "import_legacy_json",
        "export_json",
        "add_to_actual",
        "list_line_transactions",
        "delete_month_line",
        "resync_month",
        "check_for_update",
        "download_update_package",
    ] {
        if !perm_txt.contains(cmd) {
            return Err(format!("ACL missing command {cmd}"));
        }
    }

    Ok(format!(
        "demo+crud+add_to_actual+checkin+history+export+import+acl ok; grade={}",
        result.grade
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (path, portable) = db::resolve_db_path();
    let conn = db::open_db(&path).expect("failed to open database");
    let lock_enabled = db::get_setting(&conn, "lock_hash")
        .ok()
        .flatten()
        .is_some();

    // Serve embedded UI over real loopback HTTP.
    // Avoids Windows WebView2 failures with the custom tauri.localhost protocol
    // ("localhost refused the connection").
    let port = portpicker::pick_unused_port().expect("no free port for UI server");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .manage(DbState {
            conn: Mutex::new(conn),
            path,
            portable,
            unlocked: Mutex::new(!lock_enabled),
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            unlock,
            lock_app,
            set_app_lock,
            set_theme,
            get_income,
            save_income,
            list_categories,
            add_category,
            upsert_category,
            delete_category,
            dismiss_category_review,
            clear_all_data,
            list_budget_lines,
            upsert_budget_line,
            delete_budget_line,
            get_dashboard,
            save_actuals,
            update_month_meta,
            complete_check_in,
            list_history,
            reopen_month,
            resync_month,
            add_to_actual,
            list_line_transactions,
            delete_month_line,
            import_legacy_json,
            export_json,
            load_demo_data,
            current_month,
            report_ui_selftest,
            check_for_update,
            download_update_package,
        ])
        .setup(move |app| {
            if let Ok(dir) = app.path().app_data_dir() {
                let _ = std::fs::create_dir_all(dir);
            }

            let ui_selftest = std::env::var("BM9000_UI_SELFTEST").is_ok();
            let start_view = std::env::var("BM9000_START_VIEW").unwrap_or_default();
            let query = {
                let mut parts: Vec<String> = Vec::new();
                if ui_selftest {
                    parts.push("selftest=1".into());
                }
                if matches!(
                    start_view.as_str(),
                    "dashboard"
                        | "transactions"
                        | "checkin"
                        | "plan"
                        | "history"
                        | "reports"
                        | "settings"
                        | "help"
                ) {
                    parts.push(format!("view={start_view}"));
                }
                if parts.is_empty() {
                    String::new()
                } else {
                    format!("?{}", parts.join("&"))
                }
            };

            // Production: real loopback server with embedded assets.
            // Dev (`tauri dev`): Vite at 1420 (beforeDevCommand).
            #[cfg(dev)]
            let url = {
                let u = format!("http://localhost:1420/{query}");
                // fix double slash if query empty
                let u = u.replace("1420//", "1420/");
                WebviewUrl::External(u.parse().unwrap())
            };

            #[cfg(not(dev))]
            let url = {
                // Use "localhost" (not 127.0.0.1): the UI server may bind IPv6 ::1 only.
                let u = format!("http://localhost:{port}/{query}");
                let u = u.replace(&format!("{port}//"), &format!("{port}/"));
                WebviewUrl::External(u.parse().expect("valid UI url"))
            };

            WebviewWindowBuilder::new(app, "main", url)
                .title("Budget Master 9000")
                .inner_size(1280.0, 860.0)
                .min_inner_size(960.0, 640.0)
                .resizable(true)
                .maximizable(true)
                .center()
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Budget Master 9000");
}
