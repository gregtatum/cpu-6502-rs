use cpu_6502::bus::Bus;
use cpu_6502::controller::Controller;
use cpu_6502::emulator::Emulator;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::{GameControllerSubsystem, JoystickSubsystem, Sdl};
use std::cell::RefMut;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceIndex(u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct JoystickIndex(u32);

/// Manages the active controllers as exposed through SLD2.
pub struct ControllerManager {
    active_controllers: HashMap<JoystickIndex, GameController>,
    controller_subsystem: GameControllerSubsystem,
    joystick_subsystem: JoystickSubsystem,

    controller_1: Option<JoystickIndex>,
    controller_2: Option<JoystickIndex>,
}

impl ControllerManager {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let mut controller_manager = Self {
            active_controllers: HashMap::new(),
            controller_subsystem: sdl.game_controller()?,
            joystick_subsystem: sdl.joystick()?,
            controller_1: None,
            controller_2: None,
        };

        controller_manager.get_initial_controllers()?;

        Ok(controller_manager)
    }

    /// Gets the initial list of controllers.
    fn get_initial_controllers(&mut self) -> Result<(), String> {
        let num_joysticks = self.joystick_subsystem.num_joysticks()?;

        for device_index in 0..num_joysticks {
            if self.controller_subsystem.is_game_controller(device_index) {
                match self.controller_subsystem.open(device_index) {
                    Ok(controller) => {
                        let joystick_index = JoystickIndex(controller.instance_id());
                        self.active_controllers.insert(joystick_index, controller);
                        self.assign_available_controller(joystick_index);
                    }
                    Err(e) => eprintln!("Failed to open controller: {:?}", e),
                }
            }
        }
        Ok(())
    }

    /// Handles any event related to controllers from the SDL global event pump.
    /// It maps these events to the NES controllers, and handles any controllers
    /// being connected or disconnected.
    pub fn handle_event(&mut self, event: &Event, emulator: &Emulator) {
        match event {
            Event::ControllerButtonDown { button, which, .. } => {
                let index = JoystickIndex(*which);
                self.apply_event(button, index, emulator, true)
            }
            Event::ControllerButtonUp { button, which, .. } => {
                let index = JoystickIndex(*which);
                self.apply_event(button, index, emulator, false)
            }
            Event::ControllerDeviceAdded { which, .. } => {
                self.add_controller(JoystickIndex(*which));
            }
            Event::ControllerDeviceRemoved { which, .. } => {
                self.remove_controller(JoystickIndex(*which), emulator);
            }
            _ => {}
        }
    }

    fn add_controller(&mut self, index: JoystickIndex) {
        match self.controller_subsystem.open(index.0) {
            Ok(sdl_controller) => {
                self.active_controllers.insert(index, sdl_controller);
                self.assign_available_controller(index);
            }
            Err(e) => eprintln!("Failed to open controller: {:?}", e),
        }
    }

    /// If a controller slot is free, auto-assign it.
    fn assign_available_controller(&mut self, index: JoystickIndex) {
        if self.controller_1.is_none() {
            self.controller_1 = Some(index);
        } else if self.controller_2.is_none() {
            self.controller_2 = Some(index);
        }
    }

    /// Once a controller is removed, stop tracking it. If it's being used as
    /// either player 1 or 2, the controller is removed and the Emulator controller
    /// is "unplugged", setting it back to nothing pressed.
    fn remove_controller(&mut self, index: JoystickIndex, emulator: &Emulator) {
        self.active_controllers.remove(&index);

        if let Some(controller_1) = self.controller_1 {
            if controller_1 == index {
                self.controller_1 = None;
                let bus = emulator.bus.borrow();
                bus.controller_1.borrow_mut().unplug();
            }
        }

        if let Some(controller_2) = self.controller_2 {
            if controller_2 == index {
                self.controller_2 = None;
                let bus = emulator.bus.borrow();
                bus.controller_2.borrow_mut().unplug();
            }
        }
    }

    fn get_emulator_controller<'a>(
        &self,
        index: JoystickIndex,
        bus: &'a Bus,
    ) -> Option<RefMut<'a, Controller>> {
        if let Some(controller_1) = self.controller_1 {
            if controller_1 == index {
                return Some(bus.controller_1.borrow_mut());
            }
        }
        if let Some(controller_2) = self.controller_2 {
            if controller_2 == index {
                return Some(bus.controller_2.borrow_mut());
            }
        }
        None
    }

    /// Map the SDL2 controller event to the emulator controller.
    fn apply_event(
        &mut self,
        button: &Button,
        index: JoystickIndex,
        emulator: &Emulator,
        value: bool,
    ) {
        let bus = emulator.bus.borrow();
        if let Some(mut emulator_controller) = self.get_emulator_controller(index, &bus) {
            match button {
                Button::A => emulator_controller.a = value,
                Button::B => emulator_controller.b = value,
                Button::Back => emulator_controller.select = value,
                Button::Start => emulator_controller.start = value,
                Button::DPadUp => emulator_controller.up = value,
                Button::DPadDown => emulator_controller.down = value,
                Button::DPadLeft => emulator_controller.left = value,
                Button::DPadRight => emulator_controller.right = value,
                _ => {}
            }
        };
    }
}
