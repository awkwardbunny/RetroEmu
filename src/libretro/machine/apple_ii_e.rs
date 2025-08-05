use log::{debug, trace};
use crate::cpu::mos6502::MOS6502;
use crate::memory::{Endian, MemoryManager, Memory, RAM, ROM};

pub struct AppleIIe {
    cpu: MOS6502,
    memory: MemoryManager
}

impl AppleIIe {
    pub fn new() -> Self {
        trace!("new()");
        let mut mm = MemoryManager::new(0xFFFF);
        mm.map(0, Box::new(RAM::new(0x10000, Endian::Little)));

        // Monitor ROM
        mm.map(0xF800, Box::new(ROM::new(0x800, Endian::Little, "apple2e_F8.bin")));

        // 80-Column Card
        mm.map(0xC100, Box::new(ROM::new(0x300, Endian::Little, "apple2e_C1.bin")));
        mm.map(0xC800, Box::new(ROM::new(0x800, Endian::Little, "apple2e_C8.bin")));

        // Integer BASIC
        mm.map(0xE000, Box::new(ROM::new(0x1800, Endian::Little, "apple2e_ibasic_E0.bin")));

        Self {
            cpu: MOS6502::new(),
            memory: mm
        }
    }

    pub fn get_memory(&mut self) -> &mut MemoryManager {
        &mut self.memory
    }

    pub fn reset(&mut self) {
        trace!("reset()");
        self.cpu.reset(&self.memory);
    }

    pub fn run(&mut self) {
        trace!("run()");

        for _ in 0..500 {
            debug!("{}", self.cpu.step(&mut self.memory));
            debug!("{:?} {}", self.cpu, self.get_stack());
        }
    }

    fn get_stack(&self) -> String {
        let mut s = String::from("Stack:");
        let sp = self.cpu.get_sp();
        for x in (sp+1..0x200).rev() {
            s.push_str(&format!(" {:02X}", self.memory.read(x as usize)));
        }
        s
    }
}