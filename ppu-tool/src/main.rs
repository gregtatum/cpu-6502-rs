// Remove once stabler.
// #![allow(unused)]
mod constants;
mod egui_mq;
mod state;
mod view;

use crate::constants::*;
use macroquad::{self as mq, prelude::*};
use state::State;
use std::{cell::RefCell, path::PathBuf};

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
    let state = {
        let CliOptions {
            nametable,
            chartable,
            palette,
        } = CliOptions::from_args();

        RefCell::new(State::new(nametable, chartable, palette))
    };

    loop {
        state.borrow_mut().update();
        if state.borrow().shortcuts.quit {
            return;
        }

        clear_background(Color::from(state.borrow().background));
        view::main_art_view(&state);
        egui_mq::ui(|ctx| {
            view::palette_change_color_window(&ctx, &state);
            view::side_panel(&ctx, &state);
        });

        egui_mq::draw();
        next_frame().await;
    }
}
