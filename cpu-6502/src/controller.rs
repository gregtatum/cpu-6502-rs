use std::ops::BitOr;

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

/// Add BitOr support for ergonomic API usage.
impl BitOr for BUTTON {
    type Output = u8;

    fn bitor(self, rhs: BUTTON) -> u8 {
        (self as u8) | (rhs as u8)
    }
}

impl BitOr<u8> for BUTTON {
    type Output = u8;

    fn bitor(self, rhs: u8) -> u8 {
        (self as u8) | rhs
    }
}

impl BitOr<BUTTON> for u8 {
    type Output = u8;

    fn bitor(self, rhs: BUTTON) -> u8 {
        self | (rhs as u8)
    }
}

pub struct Controller {
    // Where the controller state is stored, packed into a u8.
    // 0000_0000
    // RLDU +-BA
    pub state: u8,

    // As the controller is read this state gets updated, so it's internal
    // only.
    read_state: u8,

    // The latch is used by the Bus to signal a new controller read. The latch first
    // goes up, which continuously reads the controller state. Once the latch goes
    // down the controller can be read from one bit at a time.
    is_latch_open: bool,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            state: 0,
            read_state: 0,
            is_latch_open: false,
        }
    }

    pub fn get(&self, mask: BUTTON) -> bool {
        self.state & (mask as u8) != 0
    }

    // Set an individual button.
    pub fn set_button(&mut self, mask: u8, value: bool) {
        if value {
            // set bit(s)
            self.state |= mask;
        } else {
            // clear bit(s)
            self.state &= !mask;
        }
    }

    /// When the 0b0000_0001 is written to $4016 or $4017 the latch is opened up
    /// to the controller. It will continuously reset its state to read the A button.
    pub fn open_latch(&mut self) {
        self.is_latch_open = true;
    }

    /// When the latch is closed by writing 0b0000_0000 to $4016 or $4017 the controller
    /// will begin reading out the current state one bit at a time.
    pub fn close_latch(&mut self) {
        self.is_latch_open = false;
        // When closing the latch, the next read should be the current controller state.
        self.read_state = self.state;
    }

    /// Reads a bit from the controller's state. After all of the bits have been read
    /// the controller only feeds bit value 1 through. If the latch is open, then
    /// it will just return the cu
    pub fn read_bit(&mut self) -> u8 {
        if self.is_latch_open {
            // When the latch is open, only the current controller state is returned.
            self.read_state = self.state;
        }
        let bit = self.read_state & 0b000_0001;
        self.read_state = (self.read_state >> 1) | 0b1000_0000;
        bit
    }

    /// Given a controller state, returns a text representation of it:
    ///
    /// e.g.
    ///
    ///    RLDU +-BA
    ///    1010 0001
    #[cfg(test)]
    pub fn as_string(&self) -> String {
        fn bit(c: char, pressed: bool) -> char {
            if pressed {
                c
            } else {
                '.'
            }
        }

        let s = self.state;

        let r = bit('R', s & BUTTON::Right as u8 != 0);
        let l = bit('L', s & BUTTON::Left as u8 != 0);
        let d = bit('D', s & BUTTON::Down as u8 != 0);
        let u = bit('U', s & BUTTON::Up as u8 != 0);

        let select = bit('-', s & BUTTON::Select as u8 != 0);
        let start = bit('+', s & BUTTON::Start as u8 != 0);
        let b = bit('B', s & BUTTON::B as u8 != 0);
        let a = bit('A', s & BUTTON::A as u8 != 0);

        format!("{}{}{}{} {}{}{}{}", r, l, d, u, start, select, b, a)
    }
}

impl From<u8> for Controller {
    fn from(value: u8) -> Self {
        Controller {
            state: value,
            read_state: 0,
            is_latch_open: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CONTROLLER_READ: &str = "
        JOYPAD1 = $4016
        JOYPAD2 = $4017

        ; At the same time that we strobe bit 0, we initialize the ring counter
        ; so we're hitting two birds with one stone here
        readjoy:
            lda #$01
            ; While the strobe bit is set, buttons will be continuously reloaded.
            ; This means that reading from JOYPAD1 will only return the state of the
            ; first button: button A.
            sta JOYPAD1
            sta buttons
            lsr a        ; now A is 0
            ; By storing 0 into JOYPAD1, the strobe bit is cleared and the reloading stops.
            ; This allows all 8 buttons (newly reloaded) to be read from JOYPAD1.
            sta JOYPAD1
        loop:
            lda JOYPAD1
            lsr a        ; bit 0 -> Carry
            rol buttons  ; Carry -> bit 0; bit 7 -> Carry
            bcc loop
            rts
    ";

    #[test]
    fn test_controller_struct() {
        let mut controller = Controller::from(BUTTON::A | BUTTON::Select | BUTTON::Up);
        assert_eq!(controller.as_string(), "...U .-.A");

        // Pressed:
        assert!(controller.get(BUTTON::A));
        assert!(controller.get(BUTTON::Select));
        assert!(controller.get(BUTTON::Up));

        // Not pressed:
        assert!(!controller.get(BUTTON::B));
        assert!(!controller.get(BUTTON::Start));
        assert!(!controller.get(BUTTON::Down));
        assert!(!controller.get(BUTTON::Left));
        assert!(!controller.get(BUTTON::Right));

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
                assert_eq!(controller.read_bit() == 1, controller.get(button));
            }

            // And it should continue to read as 1 afterwards.
            for _ in 0..10 {
                assert_eq!(controller.read_bit(), 1);
            }
        }
    }
}
