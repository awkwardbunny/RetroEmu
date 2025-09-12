mod ram;
mod rom;
mod vram;

pub use ram::RAM;
pub use vram::VRAM;
pub use rom::ROM;
use std::fmt::Debug;

pub enum Endian {
    Little,
    Big,
}

pub trait Memory {
    fn write(&mut self, addr: usize, data: u8);
    fn read(&self, addr: usize) -> u8;
    fn is_valid(&self, addr: usize) -> bool;
    fn read_word_zero(&self, addr: u8) -> u16;
    fn read_word(&self, addr: usize) -> u16;
    fn write_word_zero(&mut self, addr: u8, word: u16);
    fn write_word(&mut self, addr: usize, word: u16);
    fn size(&self) -> usize;
    fn get_raw(&self) -> &[u8];
}

pub type MemoryID = u32;

pub struct MMEntry {
    base: usize,
    id: MemoryID,
    region: Box<dyn Memory>,
}

impl MMEntry {
    pub fn new(base: usize, region: Box<dyn Memory>) -> Self {
        Self {
            base,
            id: 0,
            region,
        }
    }
}

impl Debug for MMEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MMEntry")
            .field("base", &format_args!("{:#X}", &self.base))
            .field("len", &format_args!("{:#X}", &self.region.size()))
            .field("id", &self.id)
            .finish()
    }
}

pub struct MemoryManager {
    regions: Vec<MMEntry>,
    endian: Endian,
    max_addr: usize,
}

impl MemoryManager {
    pub fn new(max_addr: usize) -> Self {
        Self {
            regions: Vec::new(),
            endian: Endian::Little,
            max_addr,
        }
    }

    pub fn find_by_addr_mut(&mut self, addr: usize) -> Option<&mut MMEntry> {
        self.regions
            .iter_mut()
            .find(|x| x.base <= addr && addr < (x.base + x.region.size()))
    }

    pub fn find_by_addr(&self, addr: usize) -> Option<&MMEntry> {
        self.regions
            .iter()
            .find(|x| x.base <= addr && addr < (x.base + x.region.size()))
    }

    pub fn find_by_id(&mut self, id: MemoryID) -> Option<&mut MMEntry> {
        self.regions.iter_mut().find(|x| x.id == id)
    }

    pub fn map(&mut self, addr: usize, region: Box<dyn Memory>) {
        self.regions.insert(0, MMEntry::new(addr, region));
    }
}

impl Debug for MemoryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryManager")
            .field("regions", &self.regions)
            .finish()
    }
}

impl Memory for MemoryManager {
    fn write(&mut self, addr: usize, data: u8) {
        match self.find_by_addr_mut(addr) {
            None => println!("Writing to unmapped memory {:#x}", addr),
            Some(entry) => {
                let offset = addr - entry.base;
                entry.region.write(offset, data)
            }
        }
    }

    fn read(&self, addr: usize) -> u8 {
        match self.find_by_addr(addr) {
            None => {
                println!("Reading from unmapped memory {:#x}", addr);
                return 0;
            }
            Some(entry) => {
                let offset = addr - entry.base;
                entry.region.read(offset)
            }
        }
    }

    fn is_valid(&self, addr: usize) -> bool {
        addr >= self.max_addr
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
        self.max_addr + 1
    }

    fn get_raw(&self) -> &[u8] {
        todo!()
    }
}
