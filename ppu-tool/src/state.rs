use crate::constants::*;
use macroquad::prelude::*;
use native_dialog::FileDialog;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

pub struct State {
    pub shortcuts: Shortcuts,
    pub background: Color,
    pub channel_sender: Sender<ThreadMessage>,
    pub channel_receiver: Receiver<ThreadMessage>,

    pub nametable: UserBinaryFile,
    pub chartable: UserBinaryFile,
    pub texture: Option<Texture2D>,

    pub char_texture: Option<Texture2D>,
    pub char_egui_texture: Option<egui::TextureHandle>,
    pub char_egui_image: Option<egui::ColorImage>,

    pub palette_change: PaletteChange,
    pub palettes_file: UserBinaryFile,
    pub palettes: [[u8; 4]; 4],
}

impl State {
    pub fn new(
        nametable: Option<PathBuf>,
        chartable: Option<PathBuf>,
        palette: Option<PathBuf>,
    ) -> State {
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
            shortcuts: Shortcuts::new(),
            background: BEIGE,
            nametable,
            chartable,
            channel_sender,
            channel_receiver,
            texture: None,
            char_texture: None,
            char_egui_texture: None,
            char_egui_image: None,

            palette_change: PaletteChange {
                palette_index: 0,
                color_index: 0,
                is_open: false,
            },
            palettes_file,
            palettes: [
                [0x22, 0x29, 0x1a, 0x0f],
                [0x22, 0x36, 0x17, 0x0f],
                [0x22, 0x30, 0x21, 0x0f],
                [0x22, 0x27, 0x17, 0x0f],
            ],
        };

        // Builds the texture if it's available.
        state.build_palettes();
        state.build_view_texture();
        state.build_chartable_texture();

        state
    }

    pub fn update(&mut self) {
        self.shortcuts.update();

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
                    self.build_palettes();
                }
            }
            self.build_view_texture();
            self.build_chartable_texture();
        }
    }

    fn build_palettes(&mut self) {
        if self.palettes_file.data.len() == 0 {
            // No palette data yet.
            return;
        }
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

    // TODO - Hook this up when no character table is available.
    #[allow(dead_code)]
    fn build_nametable_texture(&mut self) {
        if self.nametable.data.is_empty() || self.chartable.data.is_empty() {
            return;
        }

        let mut texture_data: [u8; NAMETABLE_W * NAMETABLE_H * 4] =
            [0; NAMETABLE_W * NAMETABLE_H * 4];
        for y in 0..NAMETABLE_H {
            for x in 0..NAMETABLE_W {
                let i = y as usize * NAMETABLE_W + x as usize;
                let value = self.nametable.data[i];
                texture_data[i * 4] = value;
                texture_data[i * 4 + 1] = value;
                texture_data[i * 4 + 2] = value;
                texture_data[i * 4 + 3] = 0xff;
            }
        }
        let texture =
            Texture2D::from_rgba8(NAMETABLE_W as u16, NAMETABLE_H as u16, &texture_data);
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
            let tile_x = tile_index % TILES_PER_SIDE;
            let tile_y = tile_index / TILES_PER_SIDE;
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

    pub fn build_view_texture(&mut self) {
        if self.nametable.data.is_empty() || self.chartable.data.is_empty() {
            return;
        }

        const TILE_PIXEL_WIDTH: usize = 8;
        const TILE_PIXEL_AREA: usize = TILE_PIXEL_WIDTH * TILE_PIXEL_WIDTH;
        const RGBA_COMPONENTS: usize = 4;
        const BYTES_PER_BIT_PLANE: usize = 8;
        const BYTES_PER_CH_TILE: usize = BYTES_PER_BIT_PLANE + BYTES_PER_BIT_PLANE; // Two bit planes
        const TEXTURE_BYTES: usize =
            NAMETABLE_W * NAMETABLE_H * RGBA_COMPONENTS * TILE_PIXEL_AREA;
        const TEXTURE_ROW_BYTES: usize = NAMETABLE_W * RGBA_COMPONENTS * TILE_PIXEL_WIDTH;

        let mut texture_data: [u8; TEXTURE_BYTES] = [0; TEXTURE_BYTES];
        for tile_y in 0..NAMETABLE_H {
            for tile_x in 0..NAMETABLE_W {
                let tile_index = tile_y as usize * NAMETABLE_W + tile_x as usize;

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
                let y_offset = RGBA_COMPONENTS * tile_y * TILE_PIXEL_AREA * NAMETABLE_W;

                for ch_y in 0..8 {
                    for ch_x in 0..8 {
                        let low_bit = (ch_plane_1[ch_y] >> (7 - ch_x)) & 0b0000_0001;
                        let high_bit = if ch_x == 7 {
                            (ch_plane_2[ch_y] << 1) & 0b0000_0010
                        } else {
                            (ch_plane_2[ch_y] >> (6 - ch_x)) & 0b0000_0010
                        };

                        let value = low_bit + high_bit;

                        if value > 3 {
                            panic!("Logic error in bit shifting.");
                        }

                        let color =
                            NTSC_PALETTE[self.palettes[0][value as usize] as usize];

                        let offset = y_offset
                            + x_offset
                            + ch_x * RGBA_COMPONENTS
                            + ch_y * TEXTURE_ROW_BYTES;

                        texture_data[offset] = color[0];
                        texture_data[offset + 1] = color[1];
                        texture_data[offset + 2] = color[2];
                        texture_data[offset + 3] = 0xff;
                    }
                }
            }
        }
        let texture = Texture2D::from_rgba8(
            (NAMETABLE_W * TILE_PIXEL_WIDTH) as u16,
            (NAMETABLE_H * TILE_PIXEL_WIDTH) as u16,
            &texture_data,
        );
        texture.set_filter(FilterMode::Nearest);
        self.texture = Some(texture);
    }
}

#[derive(Clone, Copy)]
pub struct PaletteChange {
    pub palette_index: u8,
    pub color_index: u8,
    pub is_open: bool,
}

// This works around the limitation that the logo key event is not registered on macOS.
pub struct Shortcuts {
    handler_id: usize,
    pub quit: bool,
}

impl Shortcuts {
    pub fn new() -> Self {
        Self {
            handler_id: macroquad::input::utils::register_input_subscriber(),
            quit: false,
        }
    }

    pub fn update(&mut self) {
        // Reset the state to the default.
        self.quit = false;

        macroquad::input::utils::repeat_all_miniquad_input(self, self.handler_id);
    }
}

impl miniquad::EventHandler for Shortcuts {
    fn update(&mut self, _ctx: &mut miniquad::Context) {}

    fn draw(&mut self, _ctx: &mut miniquad::Context) {}

    fn key_down_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        if keycode == miniquad::KeyCode::Q && (keymods.ctrl || keymods.logo) {
            self.quit = true;
        }
    }
}

#[derive(Copy, Clone)]
pub enum BinaryFileId {
    NameTable,
    CharTable,
    PaletteFile,
}

pub enum ThreadMessage {
    NewBinaryFile(BinaryFileId, PathBuf),
}

pub struct UserBinaryFile {
    pub id: BinaryFileId,
    pub filename: Option<String>,
    pub data: Vec<u8>,
    pub extensions: Vec<&'static str>,
    pub extension_description: &'static str,
    pub channel_sender: Sender<ThreadMessage>,
}

impl UserBinaryFile {
    /// Creates the new UserBinaryFile, and optionally initializes the file with PathBuf.
    /// The update method still needs to be called after this.
    pub fn new(
        id: BinaryFileId,
        extensions: Vec<&'static str>,
        extension_description: &'static str,
        path: Option<PathBuf>,
        channel_sender: Sender<ThreadMessage>,
    ) -> UserBinaryFile {
        let mut binary_file = UserBinaryFile {
            id,
            filename: None,
            data: Vec::new(),
            extensions,
            extension_description,
            channel_sender,
        };

        if let Some(path) = path {
            binary_file.load(path);
        }

        binary_file
    }

    /// Loads the binary file if a user has requested a new one.
    pub fn load(&mut self, path: PathBuf) {
        let data = std::fs::read(path.clone());
        if let Err(err) = data {
            eprintln!("Failed to read the nametable file: {}", err);
            return;
        }

        self.data = data.unwrap();

        let filename = path.file_name();
        if filename.is_none() {
            eprintln!("Could not get the filename from the path.");
            self.filename = None;
            return;
        }

        let filename = filename.unwrap().to_str();
        if filename.is_none() {
            eprintln!("Filename couldn't be turned into a string for the nametable.");
            self.filename = None;
            return;
        }

        self.filename = Some(filename.unwrap().to_string());
    }

    pub fn request_new_file(&mut self) {
        let channel_sender = self.channel_sender.clone();
        let description = self.extension_description;
        let extensions = self.extensions.clone();
        let id = self.id;
        when_dialog_ready(move || {
            match FileDialog::new()
                .set_location("~/Desktop")
                .add_filter(&description, &extensions)
                .show_open_single_file()
            {
                Ok(Some(p)) => {
                    if let Err(err) =
                        channel_sender.send(ThreadMessage::NewBinaryFile(id, p))
                    {
                        eprintln!("Problem sending message {:?}", err);
                    };
                }
                Err(err) => {
                    eprintln!("Unable to open nametable file. {:?}", err);
                }
                _ => {}
            }
        });
    }
}

/// On macOS the dialog can't be in the draw call. Defer it here to run as a task on the
/// main thread.
fn when_dialog_ready<F>(callback: F)
where
    F: 'static + Send + FnOnce(),
{
    #[cfg(target_os = "macos")]
    {
        let main = dispatch::Queue::main();
        main.exec_async(callback);
    }
    #[cfg(not(target_os = "macos"))]
    {
        callback();
    }
}
