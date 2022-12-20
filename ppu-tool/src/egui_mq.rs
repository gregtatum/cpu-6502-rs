// Based off of the MIT licensed
// https://github.com/optozorax/egui-macroquad/
// I chose to inline to have more control over it.

use egui_miniquad::EguiMq;
use macroquad::prelude::*;
use miniquad as mq;

pub use egui;
pub use macroquad;

struct Egui(EguiMq, usize);

// Global variable and global functions because it's more like macroquad way
static mut EGUI: Option<Egui> = None;

fn get_egui() -> &'static mut Egui {
    unsafe {
        if let Some(egui) = &mut EGUI {
            egui
        } else {
            EGUI = Some(Egui::new());
            EGUI.as_mut().unwrap()
        }
    }
}

impl Egui {
    fn new() -> Self {
        Self(
            EguiMq::new(unsafe { get_internal_gl() }.quad_context),
            macroquad::input::utils::register_input_subscriber(),
        )
    }

    fn ui<F: FnOnce(&mut mq::Context, &egui::Context)>(&mut self, f: F) {
        let gl = unsafe { get_internal_gl() };
        macroquad::input::utils::repeat_all_miniquad_input(self, self.1);

        self.0.run(gl.quad_context, f);
    }

    fn draw(&mut self) {
        let mut gl = unsafe { get_internal_gl() };
        // Ensure that macroquad's shapes are not goint to be lost, and draw them now
        gl.flush();
        self.0.draw(&mut gl.quad_context);
    }
}

/// Calculates egui ui. Must be called once per frame.
pub fn ui<F: FnOnce(&egui::Context)>(f: F) {
    get_egui().ui(|_, ctx| f(ctx))
}

/// Configure egui without beginning or ending a frame.
#[allow(dead_code)]
pub fn cfg<F: FnOnce(&egui::Context)>(f: F) {
    f(get_egui().0.egui_ctx());
}

/// Draw egui ui. Must be called after `ui` and once per frame.
pub fn draw() {
    get_egui().draw()
}

impl mq::EventHandler for Egui {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, _ctx: &mut mq::Context) {}

    fn mouse_motion_event(&mut self, _ctx: &mut mq::Context, x: f32, y: f32) {
        self.0.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.0.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.0.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.0.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.0.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.0.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
    ) {
        self.0.key_up_event(keycode, keymods);
    }
}
