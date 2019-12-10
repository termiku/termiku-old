use super::*;

impl Screen {
    pub fn reset_graphics(&mut self) {
        self.cursor.properties = CellProperties::new();
    }

    // TODO: be able to configurable
    fn get_simple_color(&mut self, color: u8) -> Color {
        SimpleColor::from_u8(color).to_color()
    }
        
    pub fn simple_color_foreground(&mut self, color: u8) {        
        self.cursor.properties.fg = self.get_simple_color(color);
    }
    
    pub fn simple_color_background(&mut self, color: u8) {
        self.cursor.properties.bg = Some(self.get_simple_color(color));
    }
    
    pub fn default_color_foreground(&mut self) {
        self.cursor.properties.fg = CellProperties::new().fg;
    }
    
    pub fn default_color_background(&mut self) {
        self.cursor.properties.bg = CellProperties::new().bg;
    }
    
    // Color is encoded as r * 36 + g * 6 + b
    fn get_color_cube(&mut self, color: u8) -> Color {
        let blue = get_hex_color_from_cube_encoding(color % 6);
        let green = get_hex_color_from_cube_encoding((color / 6) % 6);
        let red = get_hex_color_from_cube_encoding(color / 36);
        
        Color(red, green, blue, 255)
    }
    
    pub fn cube_color_foreground(&mut self, color: u8) {
        self.cursor.properties.fg = self.get_color_cube(color);
    }
    
    pub fn cube_color_background(&mut self, color: u8) {
        self.cursor.properties.bg = Some(self.get_color_cube(color));
    }
    
    // no idea if its correct, taken from https://jonasjacek.github.io/colors/
    fn get_grayscale_color(&mut self, color: u8) -> Color {
        let color = 8 + color * 10;
        
        Color(color, color, color, 255)
    }
    
    pub fn grayscale_color_foreground(&mut self, color: u8) {
        self.cursor.properties.fg = self.get_grayscale_color(color);
        
    }
    
    pub fn grayscale_color_background(&mut self, color: u8) {
        self.cursor.properties.bg = Some(self.get_grayscale_color(color));
    }
    
    pub fn true_color_foreground(&mut self, r: u8, g: u8, b: u8) {
        self.cursor.properties.fg = Color::from_rgb(r, g, b);
    }
    
    pub fn true_color_background(&mut self, r: u8, g: u8, b: u8) {
        self.cursor.properties.bg = Some(Color::from_rgb(r, g, b));
    }
}

/// Simple terminal colors, as defined by xterm.
/// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
/// 4th parameter is alpha channel, which will always be 255 for simple colors

#[repr(u8)]
enum SimpleColor {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl SimpleColor {
    pub fn from_u8(byte: u8) -> Self {
        use SimpleColor::*;
        
        match byte {
            0 => Black,
            1 => Red,
            2 => Green,
            3 => Yellow,
            4 => Blue,
            5 => Magenta,
            6 => Cyan,
            7 => White,
            8 => BrightBlack,
            9 => BrightRed,
            10 => BrightGreen,
            11 => BrightYellow,
            12 => BrightBlue,
            13 => BrightMagenta,
            14 => BrightCyan,
            15 => BrightWhite,
            _ => unreachable!()
        }
    }
    
    pub fn to_color(&self) -> Color {
        use SimpleColor::*;
        
        match self {
            Black => Color(0, 0, 0, 255),
            Red => Color(205, 0, 0, 255),
            Green => Color(0, 205, 0, 255),
            Yellow => Color(205, 205, 0, 255),
            Blue => Color(0, 0, 238, 255),
            Magenta => Color(205, 0, 205, 255),
            Cyan => Color(0, 205, 205, 255),
            White => Color(229, 229, 229, 255),
            BrightBlack => Color(127, 127, 127, 255),
            BrightRed => Color(255, 0, 0, 255),
            BrightGreen => Color(0, 255, 0, 255),
            BrightYellow => Color(255, 255, 0, 255),
            BrightBlue => Color(0, 0, 252, 255),
            BrightMagenta => Color(255, 0, 255, 255),
            BrightCyan => Color(0, 255, 255, 255),
            BrightWhite => Color(255, 255, 255, 255),
        }
    }
}

// no idea if its correct, taken from https://jonasjacek.github.io/colors/
fn get_hex_color_from_cube_encoding(data: u8) -> u8 {
    match data {
        0 => 0,
        1 => 95,
        2 => 135,
        3 => 175,
        4 => 215,
        5 => 255,
        _ => unreachable!()
    }
}
