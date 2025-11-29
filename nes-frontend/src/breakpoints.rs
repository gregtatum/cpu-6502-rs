use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakpoint {
    AnyChange,
    Value(u8),
}

pub type BreakpointMap = HashMap<u16, Breakpoint>;

/// Returns true if the breakpoint is hit when the previous value differs from the current.
pub fn is_breakpoint_hit(previous: u8, current: u8, breakpoint: &Breakpoint) -> bool {
    if previous == current {
        return false;
    }
    match breakpoint {
        Breakpoint::AnyChange => true,
        Breakpoint::Value(target) => *target == current,
    }
}
