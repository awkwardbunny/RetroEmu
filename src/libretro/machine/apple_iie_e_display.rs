use log::debug;
use crate::memory::{Endian, Memory, MemoryManager, ROM};

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
    fn toSlice(&self) -> [u8; 4] {
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

    fn toIndex(&self, w: usize) -> usize {
        self.y * w + self.x
    }
}

pub struct Display {
    rom: ROM,
    bytes: [u8; 0x400],
}

impl Display {
    pub fn new() -> Self {
        Self { rom: ROM::new(0x1000, Endian::Little, "apple2e_video.bin"), bytes: [0; 0x400] }
    }

    // pub fn update(&mut self) {}

    fn draw_pixel(&self, frame: &mut [u8], color: &Color, pos: Position) {
        let (pixels, _) = frame.as_chunks_mut::<4>();
        pixels[pos.toIndex(DISPLAY_WIDTH)].copy_from_slice(&color.toSlice());
    }

    fn draw_char(&self, frame: &mut [u8], c: char, pos: Position) {
        let (pixels, _) = frame.as_chunks_mut::<4>();
        let color = Color::new(250, 250, 250);

        let offset = c as usize;
        for i in 0..8usize {
            let row = self.rom.read(offset*8 + i);
            for j in 0..8usize {
                if (row & (1 << j) as u8) != 0 {
                    self.draw_pixel(frame, &color, Position::new(j+pos.x*8, i+pos.y*8));
                }
            }
        }
        // match c {
        //     'A'..='Z' => {
        //         let offset = c as usize - 'A' as usize + 1;
        //         for i in 0..8usize {
        //             let row = self.rom.read(offset*8 + i);
        //             for j in 0..8usize {
        //                 if (row & (1 << j) as u8) != 0 {
        //                     self.draw_pixel(frame, &color, Position::new(j+pos.x*8, i+pos.y*DISPLAY_HEIGHT));
        //                 }
        //             }
        //         }
        //     }
        //     'a'..='z' => {
        //         let offset = c as usize - 'a' as usize + 1 + 0x60;
        //         for i in 0..8usize {
        //             let row = self.rom.read(offset*8 + i);
        //             for j in 0..8usize {
        //                 if (row & (1 << j) as u8) != 0 {
        //                     self.draw_pixel(frame, &color, Position::new(j+pos.x*8, i+pos.y*DISPLAY_HEIGHT));
        //                 }
        //             }
        //         }
        //     }
        //     '0'..='9' => {
        //         let offset = c as usize - '0' as usize + 0xb0;
        //         for i in 0..8usize {
        //             let row = self.rom.read(offset*8 + i);
        //             for j in 0..8usize {
        //                 if (row & (1 << j) as u8) == 0 {
        //                     self.draw_pixel(frame, &color, Position::new(j+pos.x*8, i+pos.y*DISPLAY_HEIGHT));
        //                 }
        //             }
        //         }
        //     }
        //     special => {
        //
        //     }
        // }
    }

    fn draw_row(&self, frame: &mut [u8], mm: &MemoryManager, row: usize) {
    }

    pub fn draw(&self, frame: &mut [u8], mm: &MemoryManager) {
        let base = 0x400;
        for row in 0..24 {
            let m = row % 8;
            let r = row / 8;
            for col in 0..40 {
                self.draw_char(frame, mm.read(base + m * 0x80 + r * 0x28 + col) as char, Position::new(col, row));
            }
        }
        // for i in 0..40u8 {
        //     self.draw_char(frame, i as char, Position::new(i as usize, 0));
        // }
        // for i in 0..40u8 {
        //     self.draw_char(frame, (i+40) as char, Position::new(i as usize, 1));
        // }
        // for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        //     let x = i % DISPLAY_WIDTH as usize;
        //     let y = i / DISPLAY_HEIGHT as usize;
        //
        //     let rgba = if x < 100 {
        //         Color::new(255, 0, 0)
        //     } else {
        //         Color::new(0, 255, 0)
        //     };
        //     pixel.copy_from_slice(&rgba.toSlice());
        // }
    }
}

impl Memory for Display {
    fn write(&mut self, addr: usize, data: u8) {
        self.bytes[addr] = data;
    }

    fn read(&self, addr: usize) -> u8 {
        self.bytes[addr]
    }

    fn is_valid(&self, addr: usize) -> bool {
        addr < self.bytes.len()
    }

    fn read_word_zero(&self, addr: u8) -> u16 {
        todo!()
    }

    fn read_word(&self, addr: usize) -> u16 {
        todo!()
    }

    fn write_word_zero(&mut self, addr: u8, word: u16) {
        todo!()
    }

    fn write_word(&mut self, addr: usize, word: u16) {
        todo!()
    }

    fn size(&self) -> usize {
        0x400
    }

    fn get_raw(&self) -> &[u8] {
        &self.bytes
    }
}
