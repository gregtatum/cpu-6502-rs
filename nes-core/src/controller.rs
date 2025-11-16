pub enum BUTTON {
    A = 0b0000_0001,
    B = 0b0000_0010,
    Select = 0b0000_0100,
    Start = 0b0000_1000,
    Up = 0b0001_0000,
    Down = 0b0010_0000,
    Left = 0b0100_0000,
    Right = 0b1000_0000,
}

pub struct Controller {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    // As the controller is read this state gets updated, so it's internal only.
    read_state: u8,

    // The latch is used by the Bus to signal a new controller read. The latch first
    // goes up, which continuously reads the controller state. Once the latch goes
    // down the controller can be read from one bit at a time.
    is_latch_open: bool,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            a: false,
            b: false,
            select: false,
            start: false,
            up: false,
            down: false,
            left: false,
            right: false,
            read_state: 0,
            is_latch_open: false,
        }
    }

    fn encode_state(&self) -> u8 {
        let mut value = 0;
        if self.a {
            value |= BUTTON::A as u8;
        }
        if self.b {
            value |= BUTTON::B as u8;
        }
        if self.select {
            value |= BUTTON::Select as u8;
        }
        if self.start {
            value |= BUTTON::Start as u8;
        }
        if self.up {
            value |= BUTTON::Up as u8;
        }
        if self.down {
            value |= BUTTON::Down as u8;
        }
        if self.left {
            value |= BUTTON::Left as u8;
        }
        if self.right {
            value |= BUTTON::Right as u8;
        }
        value
    }

    /// When the 0b0000_0001 is written to $4016 or $4017 the latch is opened up
    /// to the controller. It will continuously reset its state to read the A button.
    pub fn open_latch(&mut self) {
        eprintln!("open latch");
        self.is_latch_open = true;
    }

    /// When the latch is closed by writing 0b0000_0000 to $4016 or $4017 the controller
    /// will begin reading out the current state one bit at a time.
    pub fn close_latch(&mut self) {
        eprintln!("close latch");
        self.is_latch_open = false;
        // When closing the latch, the next read should be the current controller state.
        self.read_state = self.encode_state();
    }

    /// Reads a bit from the controller's state. After all of the bits have been read
    /// the controller only feeds bit value 1 through. If the latch is open, then
    /// it will just return the cu
    pub fn read_bit(&mut self) -> u8 {
        if self.is_latch_open {
            // When the latch is open, only the current controller state is returned.
            self.read_state = self.encode_state();
        }
        let bit = self.read_state & 0b000_0001;
        self.read_state = (self.read_state >> 1) | 0b1000_0000;
        bit
    }

    /// Simulates unplugging a controller. From here on out the bits will just be 0 if
    /// the controller is read.
    ///
    /// TODO - For more accurate behavior you can simulate open bus reads:
    ///   https://www.nesdev.org/wiki/Controller_reading
    ///   https://www.nesdev.org/wiki/Open_bus_behavior
    pub fn unplug(&mut self) {
        self.a = false;
        self.b = false;
        self.select = false;
        self.start = false;
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_controller_struct() {
        let mut controller = Controller::new();

        controller.a = true;
        controller.select = true;
        controller.up = true;

        assert_eq!(controller.encode_state(), 0b0001_0101);

        // Do this test twice to ensure it can be read from again.
        for _ in 0..2 {
            // All of the reads should match.
            controller.open_latch();
            controller.close_latch();
            for button in vec![
                BUTTON::A,
                BUTTON::B,
                BUTTON::Select,
                BUTTON::Start,
                BUTTON::Up,
                BUTTON::Down,
                BUTTON::Left,
                BUTTON::Right,
            ] {
                let is_pressed = match button {
                    BUTTON::A => controller.a,
                    BUTTON::B => controller.b,
                    BUTTON::Select => controller.select,
                    BUTTON::Start => controller.start,
                    BUTTON::Up => controller.up,
                    BUTTON::Down => controller.down,
                    BUTTON::Left => controller.left,
                    BUTTON::Right => controller.right,
                };
                assert_eq!(controller.read_bit() == 1, is_pressed);
            }

            // And it should continue to read as 1 afterwards.
            for _ in 0..10 {
                assert_eq!(controller.read_bit(), 1);
            }
        }
    }
}
