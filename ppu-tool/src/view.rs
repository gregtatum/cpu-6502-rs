use crate::state::{State, View};
use crate::{constants::*, state::PaletteChange};
use egui::epaint::Hsva;
use std::cell::RefCell;

pub fn side_panel(ctx: &egui::Context, state: &RefCell<State>) {
    egui::SidePanel::right("side-panel")
        .exact_width(SIDE_PANEL_WIDTH)
        .resizable(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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
                    ui.image(texture, [SIDE_PANEL_INNER_WIDTH, SIDE_PANEL_INNER_WIDTH]);
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

                ui.horizontal_top(|ui| {
                    add_swatch_button(&state, ui, 0, 0);
                    add_swatch_button(&state, ui, 0, 1);
                    add_swatch_button(&state, ui, 0, 2);
                    add_swatch_button(&state, ui, 0, 3);
                    ui.add_space(20.0);
                    add_swatch_button(&state, ui, 1, 0);
                    add_swatch_button(&state, ui, 1, 1);
                    add_swatch_button(&state, ui, 1, 2);
                    add_swatch_button(&state, ui, 1, 3);
                });
                ui.horizontal_top(|ui| {
                    add_swatch_button(&state, ui, 2, 0);
                    add_swatch_button(&state, ui, 2, 1);
                    add_swatch_button(&state, ui, 2, 2);
                    add_swatch_button(&state, ui, 2, 3);
                    ui.add_space(20.0);
                    add_swatch_button(&state, ui, 3, 0);
                    add_swatch_button(&state, ui, 3, 1);
                    add_swatch_button(&state, ui, 3, 2);
                    add_swatch_button(&state, ui, 3, 3);
                });
            });
        });
}

pub fn palette_change_color_window(ctx: &egui::Context, state: &RefCell<State>) {
    let mut is_change_palette_open = state.borrow().palette_change.is_open;

    egui::Window::new("Change Color")
        .open(&mut is_change_palette_open)
        .collapsible(false)
        .auto_sized()
        // 485 is the measured window size, 320 just seemed like a nice default.
        .default_pos((TEXTURE_DISPLAY_W - 485.0, 320.0))
        .show(ctx, |ui| {
            for row in 0..4 {
                ui.horizontal_top(|ui| {
                    for column in 0..16 {
                        add_color_button(&state, ui, row * 16 + column);
                    }
                });
            }
        });

    if !is_change_palette_open {
        // The user closed the palette.
        state.borrow_mut().palette_change.is_open = false;
    }
}

pub fn main_art_view(state: &RefCell<State>) {
    use macroquad::prelude::*;
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
}

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

fn color_button(ui: &mut egui::Ui, ntsc_index: u8) -> egui::Response {
    let color = &NTSC_PALETTE[ntsc_index as usize];
    ui.add(
        egui::Button::new("")
            .fill(egui::Color32::from_rgb(color[0], color[1], color[2]))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(128, 128, 128),
            ))
            .min_size([PALETTE_SWATCH_SIZE, PALETTE_SWATCH_SIZE].into()),
    )
}

fn add_swatch_button(
    state: &RefCell<State>,
    ui: &mut egui::Ui,
    palette_index: u8,
    color_index: u8,
) {
    let response = color_button(
        ui,
        state.borrow().palettes[palette_index as usize][color_index as usize],
    );

    if response.clicked() {
        state.borrow_mut().palette_change = PaletteChange {
            palette_index,
            color_index,
            is_open: true,
        };
    }
}

fn add_color_button(state: &RefCell<State>, ui: &mut egui::Ui, ntsc_index: u8) {
    let response = color_button(ui, ntsc_index);

    if response.clicked() {
        let palette_change = state.borrow().palette_change;
        state.borrow_mut().palettes[palette_change.palette_index as usize]
            [palette_change.color_index as usize] = ntsc_index as u8;
        state.borrow_mut().palette_change.is_open = false;
        state.borrow_mut().build_view_texture();
    }
}

pub fn menu(ctx: &egui::Context, state: &RefCell<State>) {
    egui::TopBottomPanel::top("top menu").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            let mut view = state.borrow().view;
            // ui.visuals_mut().button_frame = false;
            let r1 = ui.selectable_value(&mut view, View::FileViewer, "File Viewer");
            let r2 = ui.selectable_value(&mut view, View::RomExplorer, "Rom Explorer");
            if r1.clicked() || r2.clicked() {
                state.borrow_mut().view = view;
            }
        });
    });
}
