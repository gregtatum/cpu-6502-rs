use egui::epaint::Hsva;
use native_dialog::FileDialog;
use std::{path::PathBuf, sync::mpsc::Sender};

pub trait ColorConvert {
    fn into_egui_hsva(&self) -> egui::ecolor::Hsva;
    fn into_mq_color(&self) -> macroquad::color::Color;
}

impl ColorConvert for egui::ecolor::Hsva {
    fn into_egui_hsva(&self) -> egui::ecolor::Hsva {
        *self
    }
    fn into_mq_color(&self) -> macroquad::color::Color {
        self.to_rgba_unmultiplied().into()
    }
}

impl ColorConvert for macroquad::color::Color {
    fn into_egui_hsva(&self) -> egui::ecolor::Hsva {
        Hsva::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }
    fn into_mq_color(&self) -> macroquad::color::Color {
        *self
    }
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
