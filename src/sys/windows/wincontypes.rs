use super::minwindef::*;

#[repr(C)]
#[derive(Default)]
pub struct COORD {
    X: SHORT,
    Y: SHORT,
}

#[repr(C)]
#[derive(Default)]
pub struct SMALL_RECT {
    Left: SHORT,
    Top: SHORT,
    Right: SHORT,
    Bottom: SHORT,
}
