// Remove once stabler.
#![allow(unused)]
mod egui_mq;
mod utils;

use crossbeam::atomic::AtomicCell;
use macroquad::{self as mq, prelude::*};
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::{cell::RefCell, path::PathBuf};
use utils::{BinaryFileId, ColorConvert, Shortcuts, ThreadMessage, UserBinaryFile};

struct State {
    pub background: Color,
    pub channel_sender: Sender<ThreadMessage>,
    pub channel_receiver: Receiver<ThreadMessage>,
    pub nametable: UserBinaryFile,
    pub chartable: UserBinaryFile,
    pub texture: Option<Texture2D>,
}
// NTSC 720x480 display, but 720x534 display due to pixel aspect ratio.
const TEXTURE_DISPLAY_W: f32 = 720.0;
const TEXTURE_DISPLAY_H: f32 = 534.0;
const SIDE_PANEL_WIDTH: f32 = 200.0;

impl State {
    pub fn new() -> State {
        let CliOptions {
            nametable,
            chartable,
        } = CliOptions::from_args();

        let (channel_sender, channel_receiver) = channel();
        let nametable = UserBinaryFile::new(
            BinaryFileId::NameTable,
            vec!["nam"],
            "NES Nametable",
            nametable,
            channel_sender.clone(),
        );
        let chartable = UserBinaryFile::new(
            BinaryFileId::CharTable,
            vec!["chr"],
            "NES Chartable",
            chartable,
            channel_sender.clone(),
        );

        let mut state = State {
            background: BEIGE,
            nametable,
            chartable,
            channel_sender,
            channel_receiver,
            texture: None,
        };

        // Builds the texture if it's available.
        state.build_texture();

        state
    }

    pub fn update(&mut self) {
        if let Ok(message) = self.channel_receiver.try_recv() {
            match message {
                ThreadMessage::NewBinaryFile(BinaryFileId::NameTable, path) => {
                    self.nametable.load(path)
                }
                ThreadMessage::NewBinaryFile(BinaryFileId::CharTable, path) => {
                    self.chartable.load(path)
                }
            }
            self.build_texture()
        }
    }

    fn build_texture(&mut self) {
        if self.nametable.data.is_empty() || self.chartable.data.is_empty() {
            return;
        }

        let mut texture_data: [u8; W * H * 4] = [0; W * H * 4];
        for y in 0..H {
            for x in 0..W {
                let i = y as usize * W + x as usize;
                let value = self.nametable.data[i];
                texture_data[i * 4] = value;
                texture_data[i * 4 + 1] = value;
                texture_data[i * 4 + 2] = value;
                texture_data[i * 4 + 3] = 0xff;
            }
        }
        let texture = Texture2D::from_rgba8(W as u16, H as u16, &texture_data);
        texture.set_filter(FilterMode::Nearest);
        self.texture = Some(texture);
    }
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "nes-nametables", about = "Visualize NES nametable files.")]
struct CliOptions {
    /// The path to a nametable file (.nam)
    #[structopt(short, long)]
    nametable: Option<PathBuf>,
    /// The path to a character table (.chr)
    #[structopt(short, long)]
    chartable: Option<PathBuf>,
}

const W: usize = 32;
const H: usize = 30;

fn main() {
    mq::Window::from_config(
        Conf {
            sample_count: 4, // msaa
            window_title: "egui with macroquad".to_string(),
            high_dpi: true,
            window_width: (TEXTURE_DISPLAY_W + SIDE_PANEL_WIDTH) as i32,
            window_height: TEXTURE_DISPLAY_H as i32,
            ..Default::default()
        },
        run(),
    );
}

async fn run() {
    let state = RefCell::new(State::new());
    let mut shortcuts = Shortcuts::new();

    loop {
        shortcuts.update();
        if shortcuts.quit {
            return;
        }

        state.borrow_mut().update();

        clear_background(Color::from(state.borrow().background));

        if let Some(texture) = state.borrow().texture {
            draw_texture_ex(
                texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width() - SIDE_PANEL_WIDTH, screen_height())),
                    ..Default::default()
                },
            );
        }

        egui_mq::ui(|egui_ctx| {
            egui::SidePanel::right("side-panel")
                .exact_width(SIDE_PANEL_WIDTH)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    ui.label("Nametable:");

                    let open_button = ui.add(egui::widgets::Button::new(
                        match state.borrow().nametable.filename {
                            Some(ref filename) => filename,
                            None => "Choose…",
                        },
                    ));
                    if open_button.clicked() {
                        state.borrow_mut().nametable.request_new_file();
                    }

                    ui.separator();

                    ui.label("Chartable");
                    let open_button = ui.add(egui::widgets::Button::new(
                        match state.borrow().chartable.filename {
                            Some(ref filename) => filename,
                            None => "Choose…",
                        },
                    ));
                    if open_button.clicked() {
                        state.borrow_mut().chartable.request_new_file();
                    }

                    ui.separator();
                });
        });

        // Draw things before egui

        egui_mq::draw();

        // Draw things after egui

        next_frame().await;
    }
}
