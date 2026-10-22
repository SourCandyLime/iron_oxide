pub type Color = u8;

pub const BLACK: Color   = rgb(0, 0, 0);

pub const GRAY: Color    = rgb(3, 3, 1);
pub const WHITE: Color   = rgb(7, 7, 3);
pub const RED: Color     = rgb(7, 0, 0);
pub const GREEN: Color   = rgb(0, 7, 0);
pub const BLUE: Color    = rgb(0, 0, 3);
pub const YELLOW: Color  = rgb(7, 7, 0);
pub const CYAN: Color    = rgb(0, 7, 3);
pub const MAGENTA: Color = rgb(7, 0, 3);

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    ((r & 0b111) << 5)
        | ((g & 0b111) << 2)
        | (b & 0b11)
}