use std::sync::Arc;
use log::{debug, error, trace};
use pixels::{Pixels, SurfaceTexture};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use crate::cpu::mos6502::MOS6502;
use crate::memory::{Endian, MemoryManager, Memory, RAM, ROM};


pub struct AppleIIe {
    cpu: MOS6502,
    memory: MemoryManager,
    is_running: bool,
}

impl AppleIIe {
    pub fn new() -> Self {
        trace!("new()");
        let mut mm = MemoryManager::new(0xFFFF);
        mm.map(0, Box::new(RAM::new(0x10000, Endian::Little)));

        // Text page 0
        mm.map(0x0400, Box::new(RAM::new(0x400, Endian::Little)));

        // Monitor ROM
        mm.map(0xF800, Box::new(ROM::new(0x800, Endian::Little, "apple2e_F8.bin")));

        // 80-Column Card
        mm.map(0xC100, Box::new(ROM::new(0x300, Endian::Little, "apple2e_C1.bin")));
        mm.map(0xC800, Box::new(ROM::new(0x800, Endian::Little, "apple2e_C8.bin")));

        // Integer BASIC
        mm.map(0xE000, Box::new(ROM::new(0x1800, Endian::Little, "apple2e_ibasic_E0.bin")));

        Self {
            cpu: MOS6502::new(),
            memory: mm,
            is_running: false,
        }
    }

    pub fn get_memory(&mut self) -> &mut MemoryManager {
        &mut self.memory
    }

    pub fn reset(&mut self) {
        trace!("reset()");
        self.cpu.reset(&self.memory);

        for i in 0..0x400 {
            self.memory.write(0x400 + i, 0x20);
        }

        let hello = AppleIIeString::from_str(String::from("HEllo world!"));
        for (i, c) in hello.string.chars().enumerate() {
            self.memory.write(0x400 + i, c as u8);
        }
    }

    pub fn run(&mut self) {
        trace!("run()");
        self.is_running = true;

        // for _ in 0..500 {
        //     debug!("{}", self.cpu.step(&mut self.memory));
        //     debug!("{:?} {}", self.cpu, self.get_stack());
        // }

    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn step(&mut self) {
        debug!("{}", self.cpu.step(&mut self.memory));
        debug!("{:?} {}", self.cpu, self.get_stack());
    }

    pub fn is_running(&self) -> bool { self.is_running }

    fn get_stack(&self) -> String {
        let mut s = String::from("Stack:");
        let sp = self.cpu.get_sp();
        for x in (sp+1..0x200).rev() {
            s.push_str(&format!(" {:02X}", self.memory.read(x as usize)));
        }
        s
    }
}

struct AppleIIeString {
    pub string: String,
    pub a2string: String,
}

impl AppleIIeString {
    pub fn from_str(str: String) -> Self {
        let mut a2 = String::with_capacity(str.len());
        for (_, c) in str.char_indices() {
            let newc = {
                if c.is_ascii_uppercase() {
                    let letter = c as u8 - 'A' as u8;
                    (0x1 + letter) as char
                } else if c.is_ascii_lowercase() {
                    let letter = c as u8 - 'a' as u8;
                    (0x61 + letter) as char
                } else if c.is_ascii_digit() {
                    let letter = c as u8 - '0' as u8;
                    (0x30 + letter) as char
                } else {
                    let val = match c {
                        '[' => 0x1B,
                        '\\' => 0x1C,
                        ']' => 0x1D,
                        '^' => 0x1E,
                        '_' => 0x1F,
                        ' ' => 0x20,
                        '!' => 0x21,
                        '"' => 0x22,
                        '#' => 0x23,
                        '$' => 0x24,
                        '%' => 0x25,
                        '&' => 0x26,
                        '\'' => 0x27,
                        '(' => 0x28,
                        ')' => 0x29,
                        '*' => 0x2A,
                        '+' => 0x2B,
                        ',' => 0x2C,
                        '-' => 0x2D,
                        '.' => 0x2E,
                        '/' => 0x2F,
                        ':' => 0x3A,
                        ';' => 0x3B,
                        '<' => 0x3C,
                        '=' => 0x3D,
                        '>' => 0x3E,
                        '?' => 0x3F,
                        _ => 0x56u8
                    };
                    val as char
                }
            };
            a2.push(newc);
        }
        Self {
            string: str,
            a2string: a2,
        }
    }

    pub fn from_a2(a2: String) -> Self {
        let str = String::with_capacity(a2.len());
        Self {
            string: str,
            a2string: a2
        }
    }
}