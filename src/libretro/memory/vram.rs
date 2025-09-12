use crate::DisplayCommand;
use crate::memory::{Endian, Memory};
use std::sync::mpsc;

pub struct VRAM {
    size: usize,
    endian: Endian,
    display: mpsc::Sender<DisplayCommand>,
}

impl VRAM {
    pub fn new(size: usize, endian: Endian, gui_tx: mpsc::Sender<DisplayCommand>) -> Self {
        Self {
            size,
            endian,
            display: gui_tx,
        }
    }
}

impl Memory for VRAM {
    fn write(&mut self, addr: usize, data: u8) {
        if self.is_valid(addr) {
            self.display
                .send(DisplayCommand::Write(addr, data))
                .unwrap();
        } else {
            panic!("Address out of bounds: {:?}", addr);
        }
    }

    fn read(&self, addr: usize) -> u8 {
        if self.is_valid(addr) {
            0u8
        } else {
            panic!("Address out of bounds: {:?}", addr);
        }
    }

    fn is_valid(&self, addr: usize) -> bool {
        addr < self.size
    }

    fn read_word_zero(&self, _: u8) -> u16 {
        0u16
    }

    fn read_word(&self, _: usize) -> u16 {
        0u16
    }

    fn write_word_zero(&mut self, addr: u8, word: u16) {
        let low = (word >> 0) as u8;
        let high = (word >> 8) as u8;
        match self.endian {
            Endian::Little => {
                self.write(addr as usize, low);
                self.write(addr.wrapping_add(1) as usize, high);
            }
            Endian::Big => {
                self.write(addr as usize, high);
                self.write(addr.wrapping_add(1) as usize, low);
            }
        }
    }

    fn write_word(&mut self, addr: usize, word: u16) {
        let low = (word >> 0) as u8;
        let high = (word >> 8) as u8;
        match self.endian {
            Endian::Little => {
                self.write(addr, low);
                self.write(addr + 1, high);
            }
            Endian::Big => {
                self.write(addr, high);
                self.write(addr + 1, low);
            }
        }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get_raw(&self) -> &[u8] {
        todo!()
    }
}
