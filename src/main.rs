use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use device_query::{DeviceQuery, DeviceState, Keycode};
use egui::{CentralPanel, Color32, Pos2, Vec2};
use egui_winit::State;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowLevel},
};

fn monitor_hotkey(shortcuts: Arc<Vec<String>>) {
    let device_state = DeviceState::new();
    loop {
        let keys = device_state.get_keys();
        if keys.contains(&Keycode::LControl) && keys.contains(&Keycode::LShift) {
            show_wheel(&shortcuts);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn show_wheel(shortcuts: &Arc<Vec<String>>) {
    let shortcuts = Arc::clone(shortcuts); // Clone the Arc for safe sharing
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("App Wheel")
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&event_loop);
    let ctx = egui::Context::default();
    let mut redraw_requested = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { ref event, .. } => {
                if state.on_event(&ctx, event).consumed {
                    redraw_requested = true;
                }
            }
            Event::RedrawRequested(_) if redraw_requested => {
                let input = state.take_egui_input(&window);
                ctx.begin_frame(input);

                CentralPanel::default().show(&ctx, |ui| {
                    draw_wheel(ui, &shortcuts);
                });

                let full_output = ctx.end_frame();
                let _clipped_primitives = ctx.tessellate(full_output.shapes);

                state.handle_platform_output(&window, &ctx, full_output.platform_output);
                redraw_requested = false;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn draw_wheel(ui: &mut egui::Ui, shortcuts: &Vec<String>) {
    let center = Pos2::new(300.0, 300.0);
    let radius = 200.0;
    let slot_count = shortcuts.len();

    for i in 0..slot_count {
        let angle = std::f32::consts::PI * 2.0 * (i as f32 / slot_count as f32);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();

        let slot_rect = egui::Rect::from_center_size(Pos2::new(x, y), Vec2::new(80.0, 30.0));
        ui.painter().rect_filled(slot_rect, 5.0, Color32::from_gray(100));

        if let Some(shortcut) = shortcuts.get(i) {
            ui.painter().text(
                slot_rect.center(),
                egui::Align2::CENTER_CENTER,
                shortcut,
                egui::TextStyle::Body.resolve(&ui.style()),
                Color32::WHITE,
            );
        }
    }
}

fn launch_application(path: &str) {
    if let Err(e) = Command::new(path).spawn() {
        eprintln!("Failed to launch application {}: {}", path, e);
    }
}

fn main() {
    let filepath = "shortcuts.txt";
    let shortcuts = Arc::new(
        fs::read_to_string(filepath)
            .unwrap_or_else(|_| {
                eprintln!("File {} not found or unreadable. Exiting...", filepath);
                std::process::exit(1);
            })
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>(),
    );

    let shortcuts_clone = Arc::clone(&shortcuts);
    let hotthread = thread::spawn(move || monitor_hotkey(shortcuts_clone));

    hotthread.join().expect("Hotkey thread panicked");
}
