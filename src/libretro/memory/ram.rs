use crate::memory::{Endian, Memory};

pub struct RAM {
    memory: Vec<u8>,
    endian: Endian,
}

impl RAM {
    pub fn new(size: usize, endian: Endian) -> Self {
        RAM {
            memory: vec![0; size],
            endian,
        }
    }
}
impl Memory for RAM {
    fn write(&mut self, addr: usize, data: u8) {
        if self.is_valid(addr) {
            self.memory[addr] = data;
        } else {
            panic!("Address out of bounds: {:?}", addr);
        }
    }

    fn read(&self, addr: usize) -> u8 {
        if self.is_valid(addr) {
            self.memory[addr]
        } else {
            panic!("Address out of bounds: {:?}", addr);
        }
    }

    fn is_valid(&self, addr: usize) -> bool {
        addr < self.memory.len()
    }

    fn read_word(&self, addr: usize) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr + 1) as u16;
        match self.endian {
            Endian::Big => low << 8 | high,
            Endian::Little => high << 8 | low,
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

    fn read_word_zero(&self, addr: u8) -> u16 {
        let low = self.read(addr as usize) as u16;
        let high = self.read(addr.wrapping_add(1) as usize) as u16;
        match self.endian {
            Endian::Big => low << 8 | high,
            Endian::Little => high << 8 | low,
        }
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

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn get_raw(&self) -> &[u8] {
        self.memory.as_slice()
    }
}

impl RAM {
    pub fn load_bytes(&mut self, addr: usize, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.write(addr.wrapping_add(i), byte);
        }
    }

    pub fn get_raw_mut(&mut self) -> &mut [u8] {
        self.memory.as_mut_slice()
    }
}