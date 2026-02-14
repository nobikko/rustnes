//! Rust NES Emulator - Desktop Application using egui

use eframe::egui;
use std::time::Instant;

use rust_nes_emulator::{NES, Rom, BUTTON_A, BUTTON_B, BUTTON_SELECT, BUTTON_START, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT};

/// App state for the egui application
struct NesApp {
    nes: NES,
    rom_loaded: bool,
    button_states: [bool; 8],
    last_frame_time: Instant,
    fps: f64,
}

impl NesApp {
    fn new() -> Self {
        let mut nes = NES::new(44100);
        // nes.debug = true;  // Disable debug output for normal operation
        Self {
            nes: nes,
            rom_loaded: false,
            button_states: [false; 8],
            last_frame_time: Instant::now(),
            fps: 0.0,
        }
    }

    fn load_rom(&mut self, path: &str) {
        match Rom::load_from_file(path) {
            Ok(rom) => {
                if self.nes.load_rom(rom).is_ok() {
                    self.rom_loaded = true;
                    eprintln!("ROM loaded successfully");
                } else {
                    eprintln!("Failed to load ROM into NES");
                }
            }
            Err(e) => {
                eprintln!("Failed to load ROM: {}", e);
            }
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        // Keyboard input - check for new key presses in events
        let mut keys_pressed_this_frame: Vec<egui::Key> = Vec::new();

        ctx.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Key { key, pressed, .. } = event {
                    if *pressed {
                        keys_pressed_this_frame.push(*key);
                    }
                }
            }
        });

        let button_map = [
            (egui::Key::Space, BUTTON_SELECT),
            (egui::Key::Enter, BUTTON_START),
            (egui::Key::ArrowUp, BUTTON_UP),
            (egui::Key::ArrowDown, BUTTON_DOWN),
            (egui::Key::ArrowLeft, BUTTON_LEFT),
            (egui::Key::ArrowRight, BUTTON_RIGHT),
            (egui::Key::A, BUTTON_A),
            (egui::Key::S, BUTTON_B),
        ];

        for (key, button) in button_map.iter() {
            let pressed = keys_pressed_this_frame.iter().any(|k| k == key);

            if pressed && !self.button_states[*button as usize] {
                self.nes.button1_down(*button);
                self.button_states[*button as usize] = true;
            } else if !pressed && self.button_states[*button as usize] {
                self.nes.button1_up(*button);
                self.button_states[*button as usize] = false;
            }
        }
    }
}

impl eframe::App for NesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle input
        self.handle_input(ctx);

        // Calculate FPS
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f64();
        self.fps = 1.0 / dt.max(0.001);
        self.last_frame_time = now;

        // Run NES frame continuously
        if self.rom_loaded {
            self.nes.frame();
        }

        // UI Layout
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Open ROM").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.load_rom(&path.to_string_lossy());
                    }
                }

                ui.label(format!("FPS: {:.1}", self.fps));
                ui.label(format!("Frames: {}", self.nes.frame_count));
                            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust NES Emulator");

            if !self.rom_loaded {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("No ROM loaded. Please select a .nes file.");
                });
            } else {
                // Display NES screen
                let pixels = self.nes.get_frame_buffer();
                let pixels_vec: Vec<u32> = pixels.to_vec();

                // Convert u32 pixels to u8 for ColorImage
                let mut rgba_bytes: Vec<u8> = Vec::with_capacity(pixels_vec.len() * 4);
                for pixel in pixels_vec.iter() {
                    rgba_bytes.push((*pixel >> 24) as u8);
                    rgba_bytes.push((*pixel >> 16) as u8);
                    rgba_bytes.push((*pixel >> 8) as u8);
                    rgba_bytes.push((*pixel) as u8);
                }

                // Create texture using egui 0.28 API
                let texture = egui::ColorImage::from_rgba_unmultiplied([256, 240], &rgba_bytes);
                let texture_handle = ctx.load_texture("nes_frame", texture, egui::TextureOptions::NEAREST);

                let image = egui::Image::from_texture(&texture_handle);

                ui.add(image);

                if ui.button("Reset").clicked() {
                    self.nes.reset();
                }

                // Show controller button states
                ui.horizontal(|ui| {
                    ui.label("Controller 1:");
                    for (i, &pressed) in self.button_states.iter().enumerate() {
                        let label = match i {
                            0 => "A",
                            1 => "B",
                            2 => "Sel",
                            3 => "Start",
                            4 => "Up",
                            5 => "Down",
                            6 => "Left",
                            7 => "Right",
                            _ => "?",
                        };
                        if pressed {
                            ui.label(egui::RichText::new(label).background_color(egui::Color32::GREEN));
                        } else {
                            ui.label(label);
                        }
                    }
                });
            }
        });

        // Update window title
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("Rust NES Emulator - {:.1} FPS", self.fps)));
    }
}

fn main() {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::Vec2::new(768.0, 720.0));

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // If a ROM file is provided as argument, pass it to the app
    let rom_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    eframe::run_native(
        "Rust NES Emulator",
        native_options,
        Box::new(move |_| {
            let mut app = NesApp::new();
            if let Some(path) = rom_path {
                app.load_rom(&path);
            }
            Ok(Box::new(app))
        }),
    ).expect("Failed to run application");
}
