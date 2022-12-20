// Remove once stabler.
#![allow(unused)]
mod egui_mq;
mod utils;

use cpu_6502::ppu::NTSC_PALETTE;
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
    pub palettes_file: UserBinaryFile,
    pub texture: Option<Texture2D>,
    pub char_texture: Option<Texture2D>,
    pub char_egui_texture: Option<egui::TextureHandle>,
    pub char_egui_image: Option<egui::ColorImage>,
    pub palettes: [[u8; 4]; 4],
}
// NTSC 720x480 display, but 720x534 display due to pixel aspect ratio.
const TEXTURE_DISPLAY_W: f32 = 720.0;
const TEXTURE_DISPLAY_H: f32 = 534.0;
const SIDE_PANEL_INNER_WIDTH: f32 = 256.0;
const SIDE_PANEL_MARGIN: f32 = 7.0;
const SIDE_PANEL_WIDTH: f32 =
    SIDE_PANEL_INNER_WIDTH + SIDE_PANEL_MARGIN + SIDE_PANEL_MARGIN;
const PALETTE_SWATCH_SIZE: f32 = 22.0;

impl State {
    pub fn new() -> State {
        let CliOptions {
            nametable,
            chartable,
            palette,
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
        let palettes_file = UserBinaryFile::new(
            BinaryFileId::PaletteFile,
            vec!["pal"],
            "NES Palette File",
            palette,
            channel_sender.clone(),
        );

        let mut state = State {
            background: BEIGE,
            nametable,
            chartable,
            channel_sender,
            channel_receiver,
            texture: None,
            char_texture: None,
            char_egui_texture: None,
            char_egui_image: None,
            palettes_file,
            palettes: [
                [0x22, 0x29, 0x1a, 0x0f],
                [0x22, 0x36, 0x17, 0x0f],
                [0x22, 0x30, 0x21, 0x0f],
                [0x22, 0x27, 0x17, 0x0f],
            ],
        };

        // Builds the texture if it's available.
        state.build_view_texture();
        state.build_chartable_texture();

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
                ThreadMessage::NewBinaryFile(BinaryFileId::PaletteFile, path) => {
                    self.palettes_file.load(path);
                    if self.palettes_file.data.len() != 16 {
                        eprintln!(
                            "Invalid palette file. Expected a 16 byte file but a {} byte file was received.",
                            self.palettes_file.data.len()
                        );
                    }
                    for (i, v) in self.palettes_file.data.iter().enumerate() {
                        self.palettes[i / 4][i % 4] = *v;
                    }
                }
            }
            self.build_view_texture();
            self.build_chartable_texture();
        }
    }

    fn build_nametable_texture(&mut self) {
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

    fn build_chartable_texture(&mut self) {
        if self.chartable.data.is_empty() {
            return;
        }

        const TILES_PER_SIDE: usize = 16;
        const TILES_COUNT: usize = TILES_PER_SIDE * TILES_PER_SIDE;
        const TILE_PIXEL_WIDTH: usize = 8;
        const PIXELS_PER_BYTE: usize = 2;
        const RGBA_COMPONENTS: usize = 4;
        const SOURCE_BYTE_LENGTH: usize = TILES_COUNT * 16;
        const TEXTURE_BYTES: usize = TILES_PER_SIDE
            * TILES_PER_SIDE
            * TILE_PIXEL_WIDTH
            * TILE_PIXEL_WIDTH
            * RGBA_COMPONENTS;

        // 4096
        if self.chartable.data.len() != SOURCE_BYTE_LENGTH {
            eprintln!(
                "Char data has size {} bytes, expected {} bytes",
                self.chartable.data.len(),
                SOURCE_BYTE_LENGTH
            );
            return;
        }
        let mut texture_data: [u8; TEXTURE_BYTES] = [0; TEXTURE_BYTES];

        for (tile_index, tile_planes) in self.chartable.data.chunks(16).enumerate() {
            let tile_x = (tile_index % TILES_PER_SIDE);
            let tile_y = (tile_index / TILES_PER_SIDE);
            let x_offset = tile_x * TILE_PIXEL_WIDTH * RGBA_COMPONENTS;
            let y_offset = tile_y
                * TILES_PER_SIDE
                * TILE_PIXEL_WIDTH
                * TILE_PIXEL_WIDTH
                * RGBA_COMPONENTS;

            let tile_plane_1 = &tile_planes[0..8];
            let tile_plane_2 = &tile_planes[8..];
            for ch_y in 0..8 {
                for ch_x in 0..8 {
                    let low_bit = (tile_plane_1[ch_y] >> (7 - ch_x)) & 0b0000_0001;
                    let high_bit = if ch_x == 7 {
                        (tile_plane_2[ch_y] << 1) & 0b0000_0010
                    } else {
                        (tile_plane_2[ch_y] >> (6 - ch_x)) & 0b0000_0010
                    };
                    let value = low_bit + high_bit;

                    let offset = y_offset
                        + x_offset
                        + ch_x * RGBA_COMPONENTS
                        + ch_y * TILES_PER_SIDE * TILE_PIXEL_WIDTH * RGBA_COMPONENTS;

                    let color = match value {
                        0 => 0,
                        1 => 85,
                        2 => 170,
                        _ => 255,
                    };
                    texture_data[offset] = color;
                    texture_data[offset + 1] = color;
                    texture_data[offset + 2] = color;
                    texture_data[offset + 3] = 0xff;
                }
            }
        }

        let texture = Texture2D::from_rgba8(
            (TILES_PER_SIDE * TILE_PIXEL_WIDTH) as u16,
            (TILES_PER_SIDE * TILE_PIXEL_WIDTH) as u16,
            &texture_data,
        );
        texture.set_filter(FilterMode::Nearest);
        self.char_texture = Some(texture);

        self.char_egui_image = Some(egui::ColorImage::from_rgba_unmultiplied(
            [
                TILES_PER_SIDE * TILE_PIXEL_WIDTH,
                TILES_PER_SIDE * TILE_PIXEL_WIDTH,
            ],
            &texture_data,
        ));
        self.char_egui_texture = None;
    }

    fn build_view_texture(&mut self) {
        if self.nametable.data.is_empty() || self.chartable.data.is_empty() {
            return;
        }

        const TILES_PER_SIDE: usize = 16;
        const TILES_COUNT: usize = TILES_PER_SIDE * TILES_PER_SIDE;
        const TILE_PIXEL_WIDTH: usize = 8;
        const TILE_PIXEL_AREA: usize = TILE_PIXEL_WIDTH * TILE_PIXEL_WIDTH;
        const PIXELS_PER_BYTE: usize = 2;
        const RGBA_COMPONENTS: usize = 4;
        const BYTES_PER_BIT_PLANE: usize = 8;
        const BYTES_PER_CH_TILE: usize = BYTES_PER_BIT_PLANE + BYTES_PER_BIT_PLANE; // Two bit planes
        const SOURCE_BYTE_LENGTH: usize = TILES_COUNT * 16;
        const TEXTURE_BYTES: usize = W * H * RGBA_COMPONENTS * TILE_PIXEL_AREA;
        const TEXTURE_ROW_BYTES: usize = W * RGBA_COMPONENTS * TILE_PIXEL_WIDTH;

        let mut texture_data: [u8; TEXTURE_BYTES] = [0; TEXTURE_BYTES];
        for tile_y in 0..H {
            for tile_x in 0..W {
                let tile_index = tile_y as usize * W + tile_x as usize;
                let ch_lookup = self.nametable.data[tile_index];

                // https://www.nesdev.org/wiki/PPU_pattern_tables
                let ch_byte_offset =
                    self.nametable.data[tile_index] as usize * BYTES_PER_CH_TILE;
                let offsets = (
                    ch_byte_offset,
                    ch_byte_offset + BYTES_PER_BIT_PLANE,
                    ch_byte_offset + BYTES_PER_BIT_PLANE + BYTES_PER_BIT_PLANE,
                );
                let ch_plane_1 = &self.chartable.data[offsets.0..offsets.1];
                let ch_plane_2 = &self.chartable.data[offsets.1..offsets.2];

                let x_offset = RGBA_COMPONENTS * tile_x * TILE_PIXEL_WIDTH;
                let y_offset = RGBA_COMPONENTS * tile_y * TILE_PIXEL_AREA * W;

                for ch_y in 0..8 {
                    for ch_x in 0..8 {
                        let low_bit = (ch_plane_1[ch_y] >> (7 - ch_x)) & 0b0000_0001;
                        let high_bit = if ch_x == 7 {
                            (ch_plane_2[ch_y] << 1) & 0b0000_0010
                        } else {
                            (ch_plane_2[ch_y] >> (6 - ch_x)) & 0b0000_0010
                        };

                        let value = low_bit + high_bit;

                        let color = match value {
                            0 => 0,
                            1 => 85,
                            2 => 170,
                            3 => 255,
                            _ => panic!("Logic error in bitshifting."),
                        };

                        let offset = y_offset
                            + x_offset
                            + ch_x * RGBA_COMPONENTS
                            + ch_y * TEXTURE_ROW_BYTES;

                        texture_data[offset] = color;
                        texture_data[offset + 1] = color;
                        texture_data[offset + 2] = color;
                        texture_data[offset + 3] = 0xff;
                    }
                }
            }
        }
        let texture = Texture2D::from_rgba8(
            (W * TILE_PIXEL_WIDTH) as u16,
            (H * TILE_PIXEL_WIDTH) as u16,
            &texture_data,
        );
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
    /// The path to a palette file (.pal)
    #[structopt(short, long)]
    palette: Option<PathBuf>,
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
                    dest_size: Some(vec2(
                        screen_width() - SIDE_PANEL_WIDTH,
                        screen_height(),
                    )),
                    ..Default::default()
                },
            );
        }

        // if let Some(texture) = state.borrow().char_texture {
        //     draw_texture_ex(
        //         texture,
        //         0.0,
        //         0.0,
        //         WHITE,
        //         DrawTextureParams {
        //             dest_size: Some(vec2(
        //                 screen_width() - SIDE_PANEL_WIDTH,
        //                 screen_height(),
        //             )),
        //             ..Default::default()
        //         },
        //     );
        // }

        egui_mq::ui(|egui_ctx| {
            let panel = egui::SidePanel::right("side-panel")
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

                    // Maybe initialize the character texture.
                    if state.borrow().char_egui_texture.is_none() {
                        let image = state.borrow_mut().char_egui_image.take();
                        if let Some(image) = image {
                            state.borrow_mut().char_egui_texture =
                                Some(ui.ctx().load_texture(
                                    "char table",
                                    image,
                                    egui::TextureOptions {
                                        magnification: egui::TextureFilter::Nearest,
                                        minification: egui::TextureFilter::Nearest,
                                    },
                                ));
                        }
                    }

                    if let Some(ref texture) = state.borrow().char_egui_texture {
                        ui.image(
                            texture,
                            [SIDE_PANEL_INNER_WIDTH, SIDE_PANEL_INNER_WIDTH],
                        );
                    }

                    ui.separator();

                    ui.label("Palettes");

                    let open_button = ui.add(egui::widgets::Button::new(
                        match state.borrow().palettes_file.filename {
                            Some(ref filename) => filename,
                            None => "Choose…",
                        },
                    ));
                    if open_button.clicked() {
                        state.borrow_mut().palettes_file.request_new_file();
                    }

                    let palettes = state.borrow().palettes.clone();
                    ui.horizontal_top(|ui| {
                        add_swatch_button(ui, palettes[0][0]);
                        add_swatch_button(ui, palettes[0][1]);
                        add_swatch_button(ui, palettes[0][2]);
                        add_swatch_button(ui, palettes[0][3]);
                        ui.add_space(20.0);
                        add_swatch_button(ui, palettes[1][0]);
                        add_swatch_button(ui, palettes[1][1]);
                        add_swatch_button(ui, palettes[1][2]);
                        add_swatch_button(ui, palettes[1][3]);
                    });
                    ui.horizontal_top(|ui| {
                        add_swatch_button(ui, palettes[2][0]);
                        add_swatch_button(ui, palettes[2][1]);
                        add_swatch_button(ui, palettes[2][2]);
                        add_swatch_button(ui, palettes[2][3]);
                        ui.add_space(20.0);
                        add_swatch_button(ui, palettes[3][0]);
                        add_swatch_button(ui, palettes[3][1]);
                        add_swatch_button(ui, palettes[3][2]);
                        add_swatch_button(ui, palettes[3][3]);
                    });
                });
        });

        // Draw things before egui

        egui_mq::draw();

        // Draw things after egui

        next_frame().await;
    }
}

fn add_swatch_button(ui: &mut egui::Ui, palette_index: u8) {
    let color = NTSC_PALETTE[palette_index as usize];
    ui.add(
        egui::Button::new("")
            .fill(egui::Color32::from_rgb(color[0], color[1], color[2]))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(128, 128, 128),
            ))
            .min_size([PALETTE_SWATCH_SIZE, PALETTE_SWATCH_SIZE].into()),
    );
}
