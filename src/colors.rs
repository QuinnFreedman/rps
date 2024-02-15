#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub fg: u8,
    pub bg: u8,
    pub fgb: u8,
    pub bgb: u8,
    pub name: &'static str,
}

#[allow(unused)]
pub const BLACK: Color = Color {
    fg: 30,
    bg: 40,
    fgb: 90,
    bgb: 100,
    name: "black",
};
#[allow(unused)]
pub const RED: Color = Color {
    fg: 31,
    bg: 41,
    fgb: 91,
    bgb: 101,
    name: "red",
};
#[allow(unused)]
pub const GREEN: Color = Color {
    fg: 32,
    bg: 42,
    fgb: 92,
    bgb: 102,
    name: "green",
};
#[allow(unused)]
pub const YELLOW: Color = Color {
    fg: 33,
    bg: 43,
    fgb: 93,
    bgb: 103,
    name: "yellow",
};
#[allow(unused)]
pub const BLUE: Color = Color {
    fg: 34,
    bg: 44,
    fgb: 94,
    bgb: 104,
    name: "blue",
};
#[allow(unused)]
pub const MAGENTA: Color = Color {
    fg: 35,
    bg: 45,
    fgb: 95,
    bgb: 105,
    name: "magenta",
};
#[allow(unused)]
pub const CYAN: Color = Color {
    fg: 36,
    bg: 46,
    fgb: 96,
    bgb: 106,
    name: "cyan",
};
#[allow(unused)]
pub const WHITE: Color = Color {
    fg: 37,
    bg: 47,
    fgb: 97,
    bgb: 107,
    name: "white",
};
#[allow(unused)]
pub const DEFAULT: Color = Color {
    fg: 39,
    bg: 49,
    fgb: 99,
    bgb: 109,
    name: "",
};
