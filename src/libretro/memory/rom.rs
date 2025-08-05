use crate::memory::{Endian, Memory};
use log::{debug, warn};
use memmap2::Mmap;
use std::env;
use std::fs::File;
use std::path::PathBuf;

pub struct ROM {
    memory: Vec<u8>,
    endian: Endian,
}

impl ROM {
    fn get_rom_path(name: &str) -> PathBuf {
        // Try to get RETRO_PATH from environment, default to "~/.retro" if not set
        let retro_path =
            env::var("RETRO_PATH").unwrap_or_else(|_| env::var("HOME").unwrap() + "/.retro");
        let mut path = PathBuf::from(retro_path);
        path.push("rom");
        path.push(name);
        path
    }

    pub fn new(_size: usize, endian: Endian, path: &str) -> Self {
        let rom_path = Self::get_rom_path(path);
        debug!("Loading ROM from {}", rom_path.display());

        let Ok(file) = File::open(&rom_path) else {
            panic!("Cannot find file {}", rom_path.display());
        };

        let mmap = unsafe { Mmap::map(&file).unwrap() };
        ROM {
            memory: mmap.to_vec(),
            endian,
        }
    }
}
impl Memory for ROM {
    fn write(&mut self, addr: usize, data: u8) {
        warn!("Writing to ROM ignored: {:04X}: {:02X}", addr, data);
    }

    fn read(&self, addr: usize) -> u8 {
        if self.is_valid(addr) {
            self.memory[addr]
        } else {
            warn!("Reading out of bounds: {:?}", addr);
            0
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
        warn!("Writing to ROM ignored: {:04X}: {:04X}", addr, word);
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
        warn!("Writing to ROM ignored: {:04X}: {:04X}", addr, word);
    }

    fn size(&self) -> usize {
        self.memory.len()
    }

    fn get_raw(&self) -> &[u8] {
        self.memory.as_slice()
    }
}
