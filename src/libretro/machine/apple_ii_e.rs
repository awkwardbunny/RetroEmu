use crate::cpu::mos6502::MOS6502;
use crate::machine::Machine;
use crate::memory::{Endian, Memory, MemoryManager, RAM, ROM, VRAM};
use crate::DisplayCommand;
use log::{debug, trace};
use std::fs::File;
use std::sync::mpsc;

pub struct AppleIIe {
    cpu: MOS6502,
    memory: MemoryManager,
    disk1: Option<File>,
    disk2: Option<File>,
}

impl Machine for AppleIIe {
    fn reset(&mut self) {
        trace!("reset()");
        self.cpu.reset(&self.memory);

        // for i in 0..0x400 {
        //     self.memory.write(0x400 + i, 0x20);
        // }
        //
        // let hello = AppleIIeString::from_str(String::from("HEllo world!"));
        // for (i, c) in hello.string.chars().enumerate() {
        //     self.memory.write(0x400 + i, c as u8);
        // }
    }

    fn cycle(&mut self) {
        trace!("cycle()");
        if let Some(i) = self.cpu.cycle(&mut self.memory) {
            debug!("{}", i);
        }
    }

    fn step(&mut self) {
        debug!("{}", self.cpu.step(&mut self.memory));
        debug!("{:?} {}", self.cpu, self.get_stack());
    }

    fn read(&self, addr: usize) -> u8 {
        self.memory.read(addr)
    }

    fn write(&mut self, addr: usize, data: u8) {
        self.memory.write(addr, data);
    }
}

impl AppleIIe {
    pub fn new(gui_tx: mpsc::Sender<DisplayCommand>) -> Self {
        trace!("new()");
        let mut mm = MemoryManager::new(0xFFFF);
        mm.map(0, Box::new(RAM::new(0x10000, Endian::Little)));

        // Text page 0
        mm.map(0x0400, Box::new(VRAM::new(0x400, Endian::Little, gui_tx)));

        // Monitor ROM
        mm.map(
            0xF800,
            // Box::new(ROM::new(0x800, Endian::Little, "apple2e_F8.bin")),
            Box::new(ROM::new(0x800, Endian::Little, "apple2e_vtest.bin")),
        );

        // 80-Column Card
        mm.map(
            0xC100,
            Box::new(ROM::new(0x300, Endian::Little, "apple2e_C1.bin")),
        );
        mm.map(
            0xC800,
            Box::new(ROM::new(0x800, Endian::Little, "apple2e_C8.bin")),
        );

        // Integer BASIC
        mm.map(
            0xE000,
            Box::new(ROM::new(0x1800, Endian::Little, "apple2e_ibasic_E0.bin")),
        );

        let mut mach = Self {
            cpu: MOS6502::new(),
            memory: mm,
            disk1: None,
            disk2: None,
        };
        mach.reset();
        mach
    }

    pub fn load_disk1(&mut self, disk: File) {
        self.disk1 = Some(disk);
    }

    pub fn unload_disk1(&mut self) {
        self.disk1 = None;
    }

    pub fn load_disk2(&mut self, disk: File) {
        self.disk2 = Some(disk);
    }

    pub fn unload_disk2(&mut self) {
        self.disk2 = None;
    }

    pub fn get_memory(&mut self) -> &mut MemoryManager {
        &mut self.memory
    }

    fn get_stack(&self) -> String {
        let mut s = String::from("Stack:");
        let sp = self.cpu.get_sp();
        for x in (sp + 1..0x200).rev() {
            s.push_str(&format!(" {:02X}", self.memory.read(x as usize)));
        }
        s
    }
}
