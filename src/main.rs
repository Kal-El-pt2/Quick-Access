use eframe::{egui, NativeOptions};  // No need to import Error
use device_query::{DeviceQuery, Keycode, DeviceState};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
enum AppError {
    IoError(io::Error),
    FileParseError(String),
    EframeError(eframe::Error),  // Add EframeError variant to capture eframe::Error
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::FileParseError(err)
    }
}

impl From<eframe::Error> for AppError {
    fn from(err: eframe::Error) -> Self {
        AppError::EframeError(err)  // Convert eframe::Error to AppError
    }
}

struct ShortcutApp {
    shortcuts: Vec<(String, String)>, // (path, alias)
    visible: Arc<Mutex<bool>>,         // Shared state to control visibility
}

impl eframe::App for ShortcutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_visible = *self.visible.lock().unwrap();
        if is_visible {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Shortcut List");
                for (path, alias) in &self.shortcuts {
                    if ui.button(alias).clicked() {
                        // Run the corresponding path when the alias is clicked
                        match Command::new(path).spawn() {
                            Ok(_) => eprintln!("Successfully launched: {}", path),
                            Err(e) => eprintln!("Failed to launch {}: {}", alias, e),
                        }
                    }
                }
            });
        } else {
            ctx.request_repaint_after(Duration::from_millis(16)); // Request a repaint
        }
    }
}

fn read_shortcuts_from_file(file_path: &str) -> Result<Vec<(String, String)>, AppError> {
    let path = Path::new(file_path);
    let file = File::open(path).map_err(AppError::IoError)?;
    let reader = io::BufReader::new(file);
    let mut shortcuts = Vec::new();
    
    for line in reader.lines() {
        if let Ok(line_content) = line {
            if let Some((path, alias)) = line_content.split_once("::") {
                let path = path.trim_matches('"').to_string();
                let alias = alias.to_string();
                shortcuts.push((path, alias));
            } else {
                return Err(AppError::FileParseError(format!("Invalid line format: {}", line_content)));
            }
        }
    }
    
    Ok(shortcuts)
}

fn main() -> Result<(), AppError> {  // Use AppError instead of eframe::Error
    let file_path = "shortcuts.txt"; // Path to your shortcuts.txt file
    match read_shortcuts_from_file(file_path) {
        Ok(shortcuts) => {
            let visible = Arc::new(Mutex::new(false)); // Control visibility of the UI
            let visible_clone = Arc::clone(&visible);

            // Start a thread to listen for hotkeys
            thread::spawn(move || {
                let device_query = DeviceState::new();
                loop {
                    let keys = device_query.get_keys();
                    let ctrl_pressed = keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl);
                    let shift_pressed = keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift);

                    if ctrl_pressed && shift_pressed {
                        *visible_clone.lock().unwrap() = true; // Show UI when both Ctrl and Shift are pressed
                    } else {
                        *visible_clone.lock().unwrap() = false; // Hide UI when keys are released
                    }

                    thread::sleep(Duration::from_millis(50)); // Avoid high CPU usage
                }
            });

            let options = NativeOptions {
                vsync: true,
                ..Default::default()
            };

            eframe::run_native(
                "Shortcut Listener",
                options,
                Box::new(|_| {
                    Box::new(ShortcutApp {
                        shortcuts,
                        visible,
                    })
                }),
            ).map_err(AppError::EframeError)?; // Convert eframe::Error to AppError using map_err
        }
        Err(e) => {
            eprintln!("Error reading shortcuts: {:?}", e);
            return Err(e);  // Return the custom error
        }
    }

    Ok(())
}
