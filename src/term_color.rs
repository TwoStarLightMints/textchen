struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl From<&str> for RGB {
    fn from(value: &str) -> Self {
        //! Format: red;green;blue

        let color_values: Vec<&str> = value.split(";").collect();

        Self {
            r: color_values[0].parse().unwrap(),
            g: color_values[1].parse().unwrap(),
            b: color_values[2].parse().unwrap(),
        }
    }
}

pub struct Theme {
    body_fonts: RGB,
    font_accents: RGB,
    editor_background: RGB,
    mode_line: RGB,
    title_line: RGB,
}

impl Theme {
    pub fn background_color(&self) -> String {
        format!(
            "\u{001b}[48;2;{};{};{}m",
            self.editor_background.r, self.editor_background.g, self.editor_background.b
        )
    }

    pub fn body_text_color(&self) -> String {
        format!(
            "\u{001b}[38;2;{};{};{}m\u{001b}[48;2;{};{};{}m",
            self.body_fonts.r,
            self.body_fonts.g,
            self.body_fonts.b,
            self.editor_background.r,
            self.editor_background.g,
            self.editor_background.b,
        )
    }

    pub fn title_text_color(&self) -> String {
        format!(
            "\u{001b}[38;2;{};{};{}m\u{001b}[48;2;{};{};{}m",
            self.font_accents.r,
            self.font_accents.g,
            self.font_accents.b,
            self.title_line.r,
            self.title_line.g,
            self.title_line.b,
        )
    }

    pub fn mode_text_color(&self) -> String {
        format!(
            "\u{001b}[38;2;{};{};{}m\u{001b}[48;2;{};{};{}m",
            self.font_accents.r,
            self.font_accents.g,
            self.font_accents.b,
            self.mode_line.r,
            self.mode_line.g,
            self.mode_line.b,
        )
    }

    pub fn command_text_color(&self) -> String {
        format!(
            "\u{001b}[38;2;{};{};{}m\u{001b}[48;2;{};{};{}m",
            self.body_fonts.r,
            self.body_fonts.g,
            self.body_fonts.b,
            self.editor_background.r,
            self.editor_background.g,
            self.editor_background.b,
        )
    }

    pub fn mode_line_color(&self) -> String {
        format!(
            "\u{001b}[48;2;{};{};{}m",
            self.mode_line.r, self.mode_line.g, self.mode_line.b
        )
    }

    pub fn title_line_color(&self) -> String {
        format!(
            "\u{001b}[48;2;{};{};{}m",
            self.title_line.r, self.title_line.g, self.title_line.b
        )
    }
}

pub struct ThemeBuilder {
    body_fonts: Option<RGB>,
    font_accents: Option<RGB>,
    editor_background: Option<RGB>,
    mode_line: Option<RGB>,
    title_line: Option<RGB>,
}

impl ThemeBuilder {
    pub fn new() -> Self {
        Self {
            body_fonts: None,
            font_accents: None,
            editor_background: None,
            mode_line: None,
            title_line: None,
        }
    }

    pub fn font_body(mut self, color: impl AsRef<str>) -> Self {
        self.body_fonts = Some(RGB::from(color.as_ref()));
        self
    }

    pub fn font_accents(mut self, color: impl AsRef<str>) -> Self {
        self.font_accents = Some(RGB::from(color.as_ref()));
        self
    }

    pub fn editor_background(mut self, color: impl AsRef<str>) -> Self {
        self.editor_background = Some(RGB::from(color.as_ref()));
        self
    }

    pub fn mode_line(mut self, color: impl AsRef<str>) -> Self {
        self.mode_line = Some(RGB::from(color.as_ref()));
        self
    }

    pub fn title_line(mut self, color: impl AsRef<str>) -> Self {
        self.title_line = Some(RGB::from(color.as_ref()));
        self
    }

    pub fn build(self) -> Theme {
        let default_font = "0;0;0";
        let default_background = "120;120;120";
        let default_mode = "255;255;255";

        Theme {
            body_fonts: match self.body_fonts {
                Some(color) => color,
                None => RGB::from(default_font),
            },
            font_accents: match self.font_accents {
                Some(color) => color,
                None => RGB::from(default_font),
            },
            editor_background: match self.editor_background {
                Some(color) => color,
                None => RGB::from(default_background),
            },
            mode_line: match self.mode_line {
                Some(color) => color,
                None => RGB::from(default_mode),
            },
            title_line: match self.title_line {
                Some(color) => color,
                None => RGB::from(default_background),
            },
        }
    }
}
