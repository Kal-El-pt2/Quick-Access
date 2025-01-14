use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use eframe::egui;

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
                                println!("Clicked alias: {}", alias);
                            }
                            ui.label(format!("Path: {}", path));
                        });
                    }
                }
            });
    }
}


fn draw_list(ui: &mut egui::Ui, shortcuts: &[(String, String)]) {
    // Add a heading and align content in the center
    ui.vertical_centered(|ui| {
        ui.heading("Shortcut List");
        // Render each shortcut with a button and label
        for (path, alias) in shortcuts {
            ui.horizontal(|ui| {
                if ui.button(alias).clicked() {
                    println!("Clicked alias: {}", alias);
                }
                ui.label(format!("Path: {}", path));
            });
        }
    });
}

fn read_shortcuts(file_path: &str) -> Vec<(String, String)> {
    let mut shortcuts = Vec::new();
    match std::fs::read_to_string(file_path) {
        Ok(contents) => {
            for line in contents.lines() {
                let parts: Vec<&str> = line.splitn(2, "::").collect(); // Use "::" as the delimiter
                if parts.len() == 2 {
                    shortcuts.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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