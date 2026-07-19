//! Local-only tool for Tony: simulate an installed app version and exercise the
//! same update-check + download code as Budget Master 9000 1.2.0.
//!
//! Build (from src-tauri):
//!   cargo build --release --bin bm9000-update-tester --features update-tester
//!
//! Not included in the product installer / portable release.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use budget_master_9000_lib::update::{
    check_for_update_with_current, current_version, download_update_package_with_current,
};
use budget_master_9000_lib::UpdateCheck;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 420.0])
            .with_min_inner_size([420.0, 360.0])
            .with_title("BM9000 Update Tester (local only)"),
        ..Default::default()
    };
    eframe::run_native(
        "BM9000 Update Tester",
        options,
        Box::new(|_cc| Ok(Box::new(UpdateTesterApp::default()))),
    )
}

struct UpdateTesterApp {
    /// Simulated installed version (what the real app would report as current).
    version_input: String,
    last_check: Option<UpdateCheck>,
    status: String,
    busy: bool,
    last_download: Option<String>,
}

impl Default for UpdateTesterApp {
    fn default() -> Self {
        Self {
            version_input: "1.0.0".into(),
            last_check: None,
            status: format!(
                "Uses the same check/download code as Budget Master 9000 {}.\nEnter a fake installed version (e.g. 1.0.0), then Check.",
                current_version()
            ),
            busy: false,
            last_download: None,
        }
    }
}

impl eframe::App for UpdateTesterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Update notification tester");
            ui.label("Local tool only — not for end users.");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Simulate installed version:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.version_input)
                        .desired_width(120.0)
                        .hint_text("1.0.0"),
                );
            });

            ui.add_space(6.0);
            ui.label(format!(
                "Binary / library version (real product build): {}",
                current_version()
            ));

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.busy, egui::Button::new("Check for updates"))
                    .clicked()
                {
                    self.busy = true;
                    self.status = "Checking GitHub…".into();
                    let v = self.version_input.clone();
                    // Same function path as production (with override for current version).
                    let result = check_for_update_with_current(&v);
                    self.last_check = Some(result.clone());
                    if let Some(err) = &result.error {
                        self.status = format!("Check finished with error: {err}");
                    } else if result.update_available {
                        self.status = format!(
                            "Update available: simulated {} → latest {}",
                            result.current_version, result.latest_version
                        );
                    } else {
                        self.status = format!(
                            "No update: simulated {} is current or newer than latest {}.",
                            result.current_version, result.latest_version
                        );
                    }
                    self.busy = false;
                }

                let can_download = self
                    .last_check
                    .as_ref()
                    .map(|c| c.update_available)
                    .unwrap_or(false);

                if ui
                    .add_enabled(
                        !self.busy && can_download,
                        egui::Button::new("Download update package"),
                    )
                    .clicked()
                {
                    self.busy = true;
                    self.status = "Downloading…".into();
                    let v = self.version_input.clone();
                    match download_update_package_with_current(&v) {
                        Ok(path) => {
                            self.last_download = Some(path.clone());
                            self.status = format!("Saved: {path}");
                        }
                        Err(e) => {
                            self.status = format!("Download failed: {e}");
                        }
                    }
                    self.busy = false;
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // Mirrors the in-app alert styling intent: only show when an update is available.
            if let Some(check) = &self.last_check {
                if check.update_available {
                    ui.colored_label(
                        egui::Color32::from_rgb(245, 158, 11),
                        format!(
                            "Update v{} available  (you simulated v{})",
                            check.latest_version, check.current_version
                        ),
                    );
                    ui.label("In the full app this appears in the left menu under the version number.");
                    ui.label(format!("Release page: {}", check.release_url));
                    if let Some(zip) = &check.zip_asset_name {
                        ui.label(format!("Zip asset on release: {zip}"));
                    }
                    ui.add_space(6.0);
                    if ui.button("Click alert → download (same as app)").clicked() {
                        self.busy = true;
                        let v = self.version_input.clone();
                        match download_update_package_with_current(&v) {
                            Ok(path) => {
                                self.last_download = Some(path.clone());
                                self.status = format!("Saved: {path}");
                            }
                            Err(e) => self.status = format!("Download failed: {e}"),
                        }
                        self.busy = false;
                    }
                } else {
                    ui.label("No in-app style alert (no newer version than the simulated one).");
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label("Status:");
            ui.add(
                egui::TextEdit::multiline(&mut self.status)
                    .desired_width(f32::INFINITY)
                    .desired_rows(5)
                    .interactive(false),
            );

            if let Some(path) = &self.last_download {
                ui.add_space(6.0);
                ui.label(format!("Last download path: {path}"));
            }
        });
    }
}
