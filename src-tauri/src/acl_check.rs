// Ensures every #[tauri::command] is listed in permissions/app.toml.
// Prevents "command X not allowed by ACL" at runtime after adding commands.
#[cfg(test)]
mod acl_tests {
    #[test]
    fn app_toml_allows_all_registered_commands() {
        let app_toml = include_str!("../permissions/app.toml");
        let lib_rs = include_str!("lib.rs");
        let mut commands = Vec::new();
        let mut lines = lib_rs.lines().peekable();
        while let Some(line) = lines.next() {
            if line.trim() == "#[tauri::command]" {
                while let Some(next) = lines.peek() {
                    let t = next.trim();
                    if t.starts_with("fn ") {
                        let name = t
                            .trim_start_matches("fn ")
                            .split(|c: char| c == '(' || c.is_whitespace())
                            .next()
                            .unwrap_or("");
                        if !name.is_empty() {
                            commands.push(name.to_string());
                        }
                        break;
                    }
                    if t.starts_with("#[") || t.starts_with("pub ") {
                        break;
                    }
                    lines.next();
                }
            }
        }
        assert!(!commands.is_empty(), "no tauri commands found in lib.rs");
        let mut missing = Vec::new();
        for c in &commands {
            let needle = format!("\"{c}\"");
            if !app_toml.contains(&needle) {
                missing.push(c.clone());
            }
        }
        assert!(
            missing.is_empty(),
            "commands missing from permissions/app.toml (ACL): {:?}\nAdd them to commands.allow or they fail at runtime.",
            missing
        );
    }
}
