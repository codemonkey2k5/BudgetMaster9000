// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Write CLI results next to the exe (no console in release Windows subsystem).
    let result_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("cli-result.txt")))
        .unwrap_or_else(|| std::path::PathBuf::from("cli-result.txt"));

    if args.iter().any(|a| a == "--self-test") {
        match budget_master_9000_lib::run_self_test() {
            Ok(msg) => {
                let body = format!("SELFTEST PASS: {msg}\n");
                let _ = std::fs::write(&result_path, &body);
                println!("{body}");
                std::process::exit(0);
            }
            Err(e) => {
                let body = format!("SELFTEST FAIL: {e}\n");
                let _ = std::fs::write(&result_path, &body);
                eprintln!("{body}");
                std::process::exit(1);
            }
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--seed-demo") {
        let path = args
            .get(i + 1)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("bm9000.db"));
        match budget_master_9000_lib::seed_demo_at(&path) {
            Ok(msg) => {
                let body = format!("SEED PASS: {msg}\n");
                let _ = std::fs::write(&result_path, &body);
                println!("{body}");
                std::process::exit(0);
            }
            Err(e) => {
                let body = format!("SEED FAIL: {e}\n");
                let _ = std::fs::write(&result_path, &body);
                eprintln!("{body}");
                std::process::exit(1);
            }
        }
    }
    budget_master_9000_lib::run()
}
