fn rgb_foreground(rgb: &str) -> String {
    //! Enter in format: "red;green;blue"

    let color_pieces: Vec<_> = rgb.split(";").collect();

    format!(
        "\u{001b}[38;2;{};{};{}m",
        color_pieces[0], color_pieces[1], color_pieces[2]
    )
}

fn rgb_background(rgb: &str) -> String {
    //! Enter in format: "red;green;blue"

    let color_pieces: Vec<_> = rgb.split(";").collect();

    format!(
        "\u{001b}[48;2;{};{};{}m",
        color_pieces[0], color_pieces[1], color_pieces[2]
    )
}

fn create_color(foreground: Option<&str>, background: Option<&str>) -> String {
    let mut new_color = String::new();

    match foreground {
        Some(color) => new_color += rgb_foreground(color).as_str(),
        None => (),
    }

    match background {
        Some(color) => new_color += rgb_background(color).as_str(),
        None => (),
    }

    new_color
}

fn reset_color() {
    print!("\u{001b}[0;0m");
}

fn print_colored(color: (&str, &str), message: &str) {
    print!("{}{}", create_color(Some(color.0), Some(color.1)), message);

    reset_color();
}
