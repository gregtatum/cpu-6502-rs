pub mod drivers;
pub mod instructions;
pub mod zero_page;

use crate::drivers::controller_sdl2::ControllerManager;
use crate::instructions::{InstructionsAction, InstructionsWindow};
use crate::zero_page::ZeroPageWindow;
use egui::FullOutput;
use glow::HasContext;
use nes_core::{
    asm::{AddressToLabel, AsmLexer, BytesLabels},
    cpu_6502::ExitReason,
    mappers::SimpleProgram,
    nes_core::NesCore,
    opcodes::OpCode,
};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::video::GLProfile;
use sdl2::video::Window;
use sdl2::{event::Event, VideoSubsystem};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, sync::Arc};

/// The front-end for the NES core, powered by SLD2.
struct NesFrontend {
    nes_core: NesCore,
    address_to_label: AddressToLabel,
    event_pump: sdl2::EventPump,
    controller_manager: ControllerManager,
    widgets: Widgets,
    gl: Arc<glow::Context>,
    #[expect(dead_code, reason = "RAII Handle")]
    gl_context: sdl2::video::GLContext,
    window: Window,
    frame_timer: FrameTimer,
}

impl NesFrontend {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;

        let video = sdl.video()?;
        let window = NesFrontend::setup_window(&video)?;
        let (gl, gl_context) = NesFrontend::setup_gl(&video, &window)?;
        let (nes_core, address_to_label) = create_demo_core();

        Ok(Self {
            nes_core,
            address_to_label,
            window,
            event_pump: sdl.event_pump()?,
            controller_manager: ControllerManager::new(&sdl)?,
            widgets: Widgets::new(gl.clone())?,
            gl,
            gl_context,
            frame_timer: FrameTimer::new(),
        })
    }

    /// Create a main window where all of the egui widgets live.
    fn setup_window(video: &VideoSubsystem) -> Result<Window, String> {
        video
            .window("NES Emulator", 1280, 720)
            .opengl()
            .resizable()
            .position_centered()
            .build()
            .map_err(|err| err.to_string())
    }

    ///
    fn setup_gl(
        video: &VideoSubsystem,
        window: &Window,
    ) -> Result<(Arc<glow::Context>, sdl2::video::GLContext), String> {
        {
            // Set up OpenGL attributes
            let gl_attr = video.gl_attr();
            // Don't use deprecated OpenGL functions.
            gl_attr.set_context_profile(GLProfile::Core);
            // The sdl2 examples use version 3.x, but 4.x is also available.
            gl_attr.set_context_version(3, 3);
            // Enable anti-aliasing.
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(4);

            gl_attr.set_double_buffer(true);
        }

        let sdl_gl = window.gl_create_context()?;

        // 0 for immediate updates
        // 1 for updates synchronized with the vertical retrace
        // -1 for adaptive vsync
        video.gl_set_swap_interval(1)?;

        // Convert the SDL2 gl context into a glow (GL on Whatever) so that we
        // can safely use a GL context on "whatever".
        let gl = unsafe {
            glow::Context::from_loader_function(|loader_fn_name: &str| {
                video.gl_get_proc_address(loader_fn_name) as *const _
            })
        };
        Ok((Arc::new(gl), sdl_gl))
    }

    /// Run the frontend by:
    ///
    ///   1. Processing the events
    ///   2. Advancing the CPU by at most 1 frame.
    ///   3. Drawing that frame.
    ///   4. Sleeping to keep an ~60Hz cadence.
    fn run(&mut self) -> Result<(), String> {
        const TARGET_FRAME_TIME: f64 = Duration::from_nanos(16_666_667).as_secs_f64();
        loop {
            self.frame_timer.update();

            if self.process_events()? {
                break;
            }

            match self.nes_core.frame() {
                // This will exit the entire program.
                ExitReason::KIL => break,
                ExitReason::BRK | ExitReason::MaxTicks => {}
            }

            // What other integrations from SDL2 to egui do we want to support?
            // Clipboard, others?

            let zero_page_snapshot = if self.widgets.zero_page_open() {
                let bus = self.nes_core.bus.borrow();
                let mut data = [0u8; 256];
                for (i, byte) in data.iter_mut().enumerate() {
                    *byte = bus.read_u8(i as u16);
                }
                Some(data)
            } else {
                None
            };

            let full_output = self.widgets.update(
                &self.window,
                &self.frame_timer,
                zero_page_snapshot,
                &self.nes_core.cpu,
                Some(&self.address_to_label),
                self.nes_core.is_breakpoint,
            );
            self.widgets.draw(&self.gl, &full_output, &self.window);

            if let Some(action) = self.widgets.take_instruction_action() {
                match action {
                    InstructionsAction::StepInstruction => {
                        self.nes_core.step_instruction();
                    }
                    InstructionsAction::Pause => {
                        self.nes_core.is_breakpoint = true;
                    }
                    InstructionsAction::Resume => {
                        self.nes_core.resume();
                    }
                }
            }

            let elapsed = self.frame_timer.frame_secs();
            if elapsed < TARGET_FRAME_TIME {
                thread::sleep(Duration::from_secs_f64(TARGET_FRAME_TIME - elapsed));
            }
        }

        Ok(())
    }

    /// Process the global event_pump, and return true if the program should exit.
    fn process_events(&mut self) -> Result<bool, String> {
        let events: Vec<_> = self.event_pump.poll_iter().collect();
        for event in events {
            match event {
                // The quit event was observed
                Event::Quit { .. } => return Ok(true),

                // cmd + q
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    keymod,
                    ..
                } if is_command_modifier(keymod) => return Ok(true),

                // cmd + w
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    keymod,
                    ..
                } if is_command_modifier(keymod) => {
                    return Ok(true);
                }

                // Window close button
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Close,
                    ..
                } => {
                    return Ok(true);
                }

                // Pass the events down to the individual components.
                _ => {
                    self.widgets.add_event(&event);
                    self.controller_manager.handle_event(&event, &self.nes_core)
                }
            }
        }

        Ok(false)
    }
}

/// Provide some light cross-platform support for key bindings by handling both
/// ctrl and cmd as a modifier.
fn is_command_modifier(keymod: Mod) -> bool {
    keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD)
}

fn create_demo_core() -> (NesCore, AddressToLabel) {
    let mut lexer = AsmLexer::new(
        "
            ; Fill the zero page with incrementing values.
            lda #$22
            root:
                sta $00,x
                adc #3
                inx
                brk
                jmp root
        ",
    );
    match lexer.parse() {
        Ok(()) => {
            let BytesLabels {
                mut bytes,
                address_to_label,
            } = lexer.into_bytes().unwrap();
            bytes.push(OpCode::KIL as u8);
            (
                NesCore::new(Box::new(SimpleProgram::load(&bytes))),
                address_to_label,
            )
        }
        Err(error) => {
            error.panic_nicely();
            panic!("Failed to parse fill zero page script");
        }
    }
}

fn main() {
    if let Err(err) = ensure_assets_workdir() {
        eprintln!("Warning: could not set working dir to assets root: {err}");
    }

    match NesFrontend::new() {
        Ok(mut frontend) => {
            if let Err(message) = frontend.run() {
                eprintln!("Front-end error: {message}");
            } else {
                println!("Exiting gracefully");
            }
        }
        Err(message) => {
            eprintln!("Failed to start the system: {message}");
        }
    }
}

/// Widgets are powered by egui. This struct handles the lifetimes and initialization
/// of anything egui related.
struct Widgets {
    ctx: egui::Context,
    painter: egui_glow::Painter,
    /// Integrate the SDL2 environment to the egui RawInput on every tick.
    input: egui::RawInput,
    zero_page: ZeroPageWindow,
    instructions: InstructionsWindow,
}

impl Widgets {
    fn zero_page_open(&self) -> bool {
        self.zero_page.is_open()
    }

    fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        // Extra preprocessor text injected at the top of both vertex + fragment shaders.
        let shader_prefix = "";

        // Which GLSL / GLSL ES version declaration to use in the shaders. When
        // set to none it's determined automatically for the target.
        let shader_version = None;

        // Enables a compile-time `#define DITHERING 1` in the fragment shader.
        // You must write the dithering code yourself.
        let dithering = false;

        Ok(Widgets {
            ctx: egui::Context::default(),
            painter: egui_glow::Painter::new(
                gl,
                shader_prefix,
                shader_version,
                dithering,
            )
            .map_err(|err| err.to_string())?,
            input: Default::default(),
            zero_page: ZeroPageWindow::new(),
            instructions: InstructionsWindow::new(),
        })
    }

    fn add_event(&mut self, event: &sdl2::event::Event) {
        use sdl2::keyboard::Keycode;

        // Route arrow keys directly to the zero page grid when it is focused so
        // they do not participate in egui focus navigation.
        if let sdl2::event::Event::KeyDown {
            keycode: Some(keycode),
            ..
        } = event
        {
            let mapped = match keycode {
                Keycode::Up => Some(egui::Key::ArrowUp),
                Keycode::Down => Some(egui::Key::ArrowDown),
                Keycode::Left => Some(egui::Key::ArrowLeft),
                Keycode::Right => Some(egui::Key::ArrowRight),
                _ => None,
            };
            if let Some(key) = mapped {
                if self.zero_page.grid_focused() {
                    self.zero_page.enqueue_key(key);
                }
                return;
            }
        }

        if let Some(egui_event) = Self::convert_event(event) {
            self.input.events.push(egui_event);
        }
    }

    /// SDL2 is our primary app interface, but egui is used for widgets. Convert SDL2
    /// events into egui events.
    fn convert_event(event: &sdl2::event::Event) -> Option<egui::Event> {
        use egui::PointerButton;
        use sdl2::event::Event;
        use sdl2::keyboard::{Keycode, Mod};
        use sdl2::mouse::MouseButton;

        let convert_mouse_event = |mouse_button| match mouse_button {
            MouseButton::Left => Some(PointerButton::Primary),
            MouseButton::Right => Some(PointerButton::Secondary),
            MouseButton::Middle => Some(PointerButton::Middle),
            MouseButton::X1 => Some(PointerButton::Extra1),
            MouseButton::X2 => Some(PointerButton::Extra2),
            MouseButton::Unknown => None,
        };

        let convert_key = |key| match key {
            Keycode::Tab => Some(egui::Key::Tab),
            Keycode::Up => Some(egui::Key::ArrowUp),
            Keycode::Down => Some(egui::Key::ArrowDown),
            Keycode::Left => Some(egui::Key::ArrowLeft),
            Keycode::Right => Some(egui::Key::ArrowRight),
            Keycode::Return | Keycode::Return2 | Keycode::KpEnter => {
                Some(egui::Key::Enter)
            }
            Keycode::Escape => Some(egui::Key::Escape),
            _ => None,
        };

        let convert_modifiers = |keymod: Mod| egui::Modifiers {
            alt: keymod.intersects(Mod::LALTMOD | Mod::RALTMOD),
            ctrl: keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD),
            shift: keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
            mac_cmd: keymod.intersects(Mod::LGUIMOD | Mod::RGUIMOD),
            command: keymod
                .intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD),
        };

        match *event {
            Event::MouseMotion { x, y, .. } => {
                Some(egui::Event::PointerMoved(egui::pos2(x as f32, y as f32)))
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                convert_mouse_event(mouse_btn).map(|button| egui::Event::PointerButton {
                    pos: egui::pos2(x as f32, y as f32),
                    button,
                    pressed: true,
                    // TODO - The modifiers aren't on this event and need to be tracked
                    // separately
                    modifiers: egui::Modifiers::default(),
                })
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                convert_mouse_event(mouse_btn).map(|button| egui::Event::PointerButton {
                    pos: egui::pos2(x as f32, y as f32),
                    button,
                    pressed: false,
                    // TODO - The modifiers aren't on this event and need to be tracked
                    // separately
                    modifiers: egui::Modifiers::default(),
                })
            }
            Event::MouseWheel { x, y, .. } => Some(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(x as f32, y as f32),
                // TODO - The modifiers aren't on this event and need to be tracked
                // separately
                modifiers: Default::default(),
            }),
            Event::TextInput { ref text, .. } => Some(egui::Event::Text(text.clone())),
            Event::KeyDown {
                keycode: Some(keycode),
                repeat,
                keymod,
                ..
            } => convert_key(keycode).map(|key| egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat,
                modifiers: convert_modifiers(keymod),
            }),
            Event::KeyUp {
                keycode: Some(keycode),
                repeat,
                keymod,
                ..
            } => convert_key(keycode).map(|key| egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat,
                modifiers: convert_modifiers(keymod),
            }),
            _ => None,
        }
    }

    fn update(
        &mut self,
        window: &Window,
        frame_timer: &FrameTimer,
        zero_page_snapshot: Option<[u8; 256]>,
        cpu: &nes_core::cpu_6502::Cpu6502,
        address_to_label: Option<&AddressToLabel>,
        is_breakpoint: bool,
    ) -> FullOutput {
        let (draw_width, _draw_height) = window.drawable_size();
        let (logical_width, logical_height) = window.size();
        let pixels_per_point = draw_width as f32 / logical_width.max(1) as f32;

        self.input.time = Some(frame_timer.secs_from_start());
        self.input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(logical_width as f32, logical_height as f32),
        ));

        // Sync the pixel_per_point.
        if let Some(ref mut root_viewport) =
            self.input.viewports.get_mut(&egui::ViewportId::ROOT)
        {
            root_viewport.native_pixels_per_point = Some(pixels_per_point);
        }

        // Take the input, which resets the raw input back to its default for
        // the next frame.
        let input = std::mem::take(&mut self.input);

        let zero_page_new = &mut self.zero_page;
        let instructions = &mut self.instructions;
        let zero_page_snapshot = zero_page_snapshot.map(Box::new);
        self.ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                zero_page_new.widget(ui, zero_page_snapshot.as_deref());
            });
            instructions.widget(ctx, cpu, address_to_label, is_breakpoint);
        })
    }

    fn take_instruction_action(&mut self) -> Option<InstructionsAction> {
        self.instructions.take_action()
    }

    fn draw(&mut self, gl: &glow::Context, full_output: &FullOutput, window: &Window) {
        unsafe {
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let clipped_primitives = self
            .ctx
            .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);
        let textures_delta = full_output.textures_delta.clone();
        let (width, height) = window.drawable_size();

        self.painter.paint_and_update_textures(
            [width as u32, height as u32],
            self.ctx.pixels_per_point(),
            clipped_primitives.as_slice(),
            &textures_delta,
        );

        window.gl_swap_window();
    }
}

impl Drop for Widgets {
    fn drop(&mut self) {
        self.painter.destroy();
    }
}

fn ensure_assets_workdir() -> Result<(), String> {
    // When launched via .app, the cwd is inside Contents/MacOS; walk ancestors to find the repo root.
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    for ancestor in exe.ancestors() {
        let bases = [ancestor.join("assets"), ancestor.join("Resources/assets")];
        for base in bases {
            let candidate = base.join("liberation_mono/LiberationMono-Regular.ttf");
            if candidate.exists() {
                let dir = base
                    .parent()
                    .ok_or("failed to find parent for assets directory")?;
                env::set_current_dir(dir).map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }
    Err("assets directory not found in executable ancestors".into())
}

struct FrameTimer {
    last: Option<Instant>,
    now: Option<Instant>,
    start: Instant,
}

impl FrameTimer {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            last: None,
            now: None,
        }
    }

    fn update(&mut self) {
        self.last = self.now;
        self.now = Some(Instant::now());
    }

    fn secs_from_start(&self) -> f64 {
        if let Some(now) = self.now {
            return (now - self.start).as_secs_f64();
        }
        return 0.0;
    }

    fn frame_secs(&self) -> f64 {
        (if let (Some(now), Some(last)) = (self.now, self.last) {
            now - last
        } else {
            // Assume 1 frame at 60hz.
            Duration::from_micros(16_667)
        })
        .as_secs_f64()
    }
}
