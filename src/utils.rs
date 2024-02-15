fn set_stdout_color(fg: &colors::Color, bg: &colors::Color) {
    print!("\x1b[{}m\x1b[{}m", fg.fg, bg.bg);
}
