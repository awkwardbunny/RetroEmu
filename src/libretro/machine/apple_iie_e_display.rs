use crate::memory::{Endian, Memory, ROM};

pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHT: usize = 200;
pub const DISPLAY_SCALE: u32 = 3;

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    fn to_slice(&self) -> [u8; 4] {
        [self.r, self.g, self.b, 0xFF]
    }
}

struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn to_index(&self, w: usize) -> usize {
        self.y * w + self.x
    }
}

pub struct Display {
    rom: ROM,
}

impl Display {
    pub fn new() -> Self {
        Self {
            rom: ROM::new(0x1000, Endian::Little, "apple2e_video.bin"),
        }
    }

    fn draw_pixel(&self, frame: &mut [u8], color: &Color, pos: Position) {
        let (pixels, _) = frame.as_chunks_mut::<4>();
        pixels[pos.to_index(DISPLAY_WIDTH)].copy_from_slice(&color.to_slice());
    }

    fn draw_char(&self, frame: &mut [u8], c: char, pos: Position) {
        let color = Color::new(250, 250, 250);
        let black = Color::new(0, 0, 0);

        let offset = c as usize;
        for i in 0..8usize {
            let row = self.rom.read(offset * 8 + i);
            for j in 0..8usize {
                if (row & (1 << j) as u8) != 0 {
                    self.draw_pixel(frame, &color, Position::new(j + pos.x * 8, i + pos.y * 8));
                } else {
                    self.draw_pixel(frame, &black, Position::new(j + pos.x * 8, i + pos.y * 8));
                }
            }
        }
    }

    pub fn write_text(&self, frame: &mut [u8], addr: usize, data: char) {
        let group = addr / 0x80;
        let rem = addr % 0x80;
        if rem > 0x78 {
            return;
        }

        let col = rem % 0x28;
        let group2 = rem / 0x28;
        let row = group2 * 8 + group;
        if row >= 24 {
            return;
        }
        self.draw_char(frame, data, Position::new(col, group2 * 8 + group));
    }
}
