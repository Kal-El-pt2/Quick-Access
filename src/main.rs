use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use eframe::egui;
use std::process::Command;

struct ShortcutApp {
    shortcuts: Vec<(String, String)>,
}

impl eframe::App for ShortcutApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::LIGHT_BLUE))
            .show(ctx, |ui| {
                ui.heading("Shortcut List");
                if self.shortcuts.is_empty() {
                    ui.label("No shortcuts found!");
                } else {
                    for (path, alias) in &self.shortcuts {
                        ui.horizontal(|ui| {
                            if ui.button(alias).clicked() {
                                if let Err(err) = open_file(path) {
                                    eprintln!("Failed to open file: {}", err);
                                }
                            }
                            ui.label(format!("Path: {}", path));
                        });
                    }
                }
            });
    }
}

/// Opens the given file or directory using the default system application.
fn open_file(path: &str) -> io::Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "start", path]).spawn()?;
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()?;
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(path).spawn()?;
    } else {
        eprintln!("Unsupported operating system");
    }
    Ok(())
}

fn read_shortcuts(file_path: &str) -> Vec<(String, String)> {
    let mut shortcuts = Vec::new();
    match std::fs::read_to_string(file_path) {
        Ok(contents) => {
            for line in contents.lines() {
                let parts: Vec<&str> = line.splitn(2, "::").collect(); // Use "::" as the delimiter
                if parts.len() == 2 {
                    let path = parts[0].trim().trim_matches('"').to_string(); // Remove extra quotes
                    let alias = parts[1].trim().to_string();
                    shortcuts.push((path, alias));
                } else {
                    println!("Skipping invalid line: {}", line);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to read {}: {}", file_path, err);
        }
    }
    println!("Loaded shortcuts: {:?}", shortcuts); // Debug
    shortcuts
}


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    let shortcuts = read_shortcuts("shortcuts.txt");
    eframe::run_native(
        "Shortcut List",
        options,
        Box::new(|_| Box::new(ShortcutApp { shortcuts })),
    )
}
