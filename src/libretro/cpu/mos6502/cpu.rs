use crate::cpu::mos6502::AddressingMode;
use crate::cpu::mos6502::instruction::{Instr, Instruction};
use crate::memory::Memory;
use bitmatch::bitmatch;
use log::{error, trace};
use std::fmt::{Debug, Formatter};

/// 6502 CPU Emulator
///
/// This struct represents the 6502 microprocessor, including all registers,
/// flags, and the program counter.
pub struct MOS6502 {
    /// Accumulator register
    pub a: u8,
    /// X index register
    pub x: u8,
    /// Y index register
    pub y: u8,
    /// Stack pointer
    pub sp: u8,
    /// Program counter
    pub pc: u16,
    /// Status/Flag register
    pub status: u8,
    /// Cycle counter for timing
    pub cycles: usize,
    /// Instruction count
    pub steps: usize,

    pub current: Option<Instruction>,
}

impl Debug for MOS6502 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MOS6502 CPU")
            .field("A", &format_args!("{:02X}", &self.a))
            .field("X", &format_args!("{:02X}", &self.x))
            .field("Y", &format_args!("{:02X}", &self.y))
            .field("SP", &format_args!("1{:02X}", &self.sp))
            .field("PC", &format_args!("{:04X}", &self.pc))
            .field(
                "STAT",
                &format_args!(
                    "{}{}-{}{}{}{}{}",
                    if self.get_flag(Flag::N) { "N" } else { "-" },
                    if self.get_flag(Flag::V) { "V" } else { "-" },
                    if self.get_flag(Flag::B) { "B" } else { "-" },
                    if self.get_flag(Flag::D) { "D" } else { "-" },
                    if self.get_flag(Flag::I) { "I" } else { "-" },
                    if self.get_flag(Flag::Z) { "Z" } else { "-" },
                    if self.get_flag(Flag::C) { "C" } else { "-" },
                ),
            )
            .field("#C", &format_args!("{:08}", &self.cycles))
            .field("#S", &format_args!("{:08}", &self.steps))
            .field("Current", &format_args!("{:?}", self.current))
            .finish()
    }
}

impl MOS6502 {
    /// Creates a new 6502 CPU instance
    pub fn new() -> Self {
        trace!("new()");
        MOS6502 {
            a: 0,
            x: 0,
            y: 0,
            sp: 0x00, // Stack pointer starts at $01FD
            pc: 0x0000,
            status: 0x00,
            cycles: 0,
            steps: 0,

            current: None,
        }
    }

    /// Resets the CPU to initial state
    pub fn reset(&mut self, mem: &dyn Memory) {
        trace!("reset()");
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.pc = mem.read_word(0xFFFC);
        self.status = 0x00 | 0x04; // Set interrupt disable flag
        self.cycles = 0;
        self.steps = 0;

        self.current = None;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_sp(&self) -> u16 {
        0x100 + (self.sp as u16)
    }

    /// Sets or clears a status flag
    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn update_zn(&mut self, val: u8) {
        self.set_flag(Flag::Z, val == 0);
        self.set_flag(Flag::N, val & 0x80 != 0);
    }

    /// Gets the value of a status flag
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.status & (flag as u8) != 0
    }

    /// Fetches the next byte from memory
    pub fn fetch(&mut self, mem: &dyn Memory) -> u8 {
        trace!("fetch()");
        let byte = mem.read(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Fetches the next word (2 bytes) from memory
    pub fn fetch_word(&mut self, mem: &dyn Memory) -> u16 {
        trace!("fetch_word()");
        let word = mem.read_word(self.pc as usize);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    pub fn push(&mut self, mem: &mut dyn Memory, data: u8) {
        trace!("push()");
        mem.write(0x100 + self.sp as usize, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop(&mut self, mem: &mut dyn Memory) -> u8 {
        trace!("pop()");
        self.sp = self.sp.wrapping_add(1);
        let data = mem.read(0x100 + self.sp as usize);
        data
    }

    pub fn push_word(&mut self, mem: &mut dyn Memory, data: u16) {
        trace!("push_word()");
        let bytes = data.to_le_bytes();
        self.push(mem, bytes[1]);
        self.push(mem, bytes[0]);
        // memory.write_word(0x100 + self.sp as usize, data);
        // self.sp = self.sp.wrapping_sub(2);
    }

    pub fn pop_word(&mut self, mem: &mut dyn Memory) -> u16 {
        trace!("pop_word()");
        let data_l = self.pop(mem);
        let data_h = self.pop(mem);
        ((data_h as u16) << 8) + (data_l as u16)
        // let data = mem.read_word(0x100 + self.sp as usize);
        // self.sp = self.sp.wrapping_add(2);
        // data
    }

    /// Emulate a single clock cycle
    pub fn cycle(&mut self, mem: &mut dyn Memory) -> Option<Instruction> {
        trace!("cycle()");

        if self.current.is_none() {
            self.current = Some(self.execute(mem));
        }

        self.cycles += 1;
        if self.current.as_mut().unwrap().cycle() {
            self.current.take()
        } else {
            None
        }
    }

    /// Executes a single instruction
    pub fn step(&mut self, mem: &mut dyn Memory) -> Instruction {
        trace!("step()");

        loop {
            if let Some(i) = self.cycle(mem) {
                self.steps += 1;
                return i;
            }
        }
    }

    #[bitmatch]
    fn execute(&mut self, mem: &mut dyn Memory) -> Instruction {
        trace!("execute()");

        let pc = self.pc;
        let opcode = self.fetch(mem);
        let mut bytes = vec![opcode];

        #[bitmatch]
        match opcode {
            "xxxy_yy01" => {
                let i = match x {
                    0 => Instr::ORA,
                    1 => Instr::AND,
                    2 => Instr::EOR,
                    3 => Instr::ADC,
                    4 => Instr::STA,
                    5 => Instr::LDA,
                    6 => Instr::CMP,
                    7 => Instr::SBC,
                    _ => unreachable!(),
                };
                let am = match y {
                    0 => AddressingMode::IndirectX(self.fetch(mem)),
                    1 => AddressingMode::ZeroPage(self.fetch(mem)),
                    2 => AddressingMode::Immediate(self.fetch(mem)),
                    3 => AddressingMode::Absolute(self.fetch_word(mem)),
                    4 => AddressingMode::IndirectY(self.fetch(mem)),
                    5 => AddressingMode::ZeroPageX(self.fetch(mem)),
                    6 => AddressingMode::AbsoluteY(self.fetch_word(mem)),
                    7 => AddressingMode::AbsoluteX(self.fetch_word(mem)),
                    _ => unreachable!(),
                };

                am.add_to_vec(&mut bytes);
                let c = self.instr_arithmetic(mem, &i, &am);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "0xxy_yy10" => {
                let i = match x {
                    0 => Instr::ASL,
                    1 => Instr::ROL,
                    2 => Instr::LSR,
                    3 => Instr::ROR,
                    _ => unreachable!(),
                };
                let am = match y {
                    1 => AddressingMode::ZeroPage(self.fetch(mem)),
                    2 => AddressingMode::Accumulator(),
                    3 => AddressingMode::Absolute(self.fetch_word(mem)),
                    5 => AddressingMode::ZeroPageX(self.fetch(mem)),
                    7 => AddressingMode::AbsoluteX(self.fetch_word(mem)),
                    0 | 4 | 6 => {
                        println!("{:04X}", pc);
                        unreachable!()
                    }
                    _ => unreachable!(),
                };
                am.add_to_vec(&mut bytes);
                let c = self.instr_shift(mem, &i, &am);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "11xy_y110" => {
                let (am, addr, c) = match y {
                    0 => {
                        let x = self.fetch(mem);
                        (AddressingMode::ZeroPage(x), x as usize, 5)
                    }
                    1 => {
                        let x = self.fetch_word(mem);
                        (AddressingMode::Absolute(x), x as usize, 6)
                    }
                    2 => {
                        let x = self.fetch(mem);
                        (AddressingMode::ZeroPageX(x), x as usize, 6)
                    }
                    3 => {
                        let x = self.fetch_word(mem);
                        (AddressingMode::AbsoluteX(x), x as usize, 7)
                    }
                    _ => unreachable!(),
                };
                let (i, result) = match x {
                    0 => (Instr::DEC, mem.read(addr).wrapping_sub(1)),
                    1 => (Instr::INC, mem.read(addr).wrapping_add(1)),
                    _ => unreachable!(),
                };
                mem.write(addr, result);
                self.update_zn(result);

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "10xy_y1x0" => {
                let (am, addr, c) = match y {
                    0 => {
                        let x = self.fetch(mem);
                        (AddressingMode::ZeroPage(x), x as usize, 3)
                    }
                    1 => {
                        let x = self.fetch_word(mem);
                        (AddressingMode::Absolute(x), x as usize, 4)
                    }
                    2 => {
                        let x = self.fetch(mem);
                        if x & 1 == 1 {
                            (
                                AddressingMode::ZeroPageY(x),
                                x.wrapping_add(self.y) as usize,
                                4,
                            )
                        } else {
                            (
                                AddressingMode::ZeroPageX(x),
                                x.wrapping_add(self.x) as usize,
                                4,
                            )
                        }
                    }
                    3 => (AddressingMode::Implied(), 0, 1),
                    _ => unreachable!(),
                };

                if let AddressingMode::Implied() = am {
                    return Instruction::new(pc, &bytes, Instr::UNK, AddressingMode::Implied(), 0);
                }

                let i = match x {
                    0 => {
                        mem.write(addr, self.y);
                        Instr::STY
                    }
                    1 => {
                        mem.write(addr, self.x);
                        Instr::STX
                    }
                    2 => {
                        self.y = mem.read(addr);
                        self.update_zn(self.y);
                        Instr::LDY
                    }
                    3 => {
                        self.x = mem.read(addr);
                        self.update_zn(self.x);
                        Instr::LDX
                    }
                    _ => unreachable!(),
                };

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "xxy1_1000" => {
                let is_y = y == 1;
                let i = match x {
                    0 => {
                        self.set_flag(Flag::C, is_y);
                        if is_y { Instr::SEC } else { Instr::CLC }
                    }
                    1 => {
                        self.set_flag(Flag::I, is_y);
                        if is_y { Instr::SEI } else { Instr::CLI }
                    }
                    2 => {
                        if is_y {
                            self.a = self.y;
                            self.update_zn(self.y);
                            Instr::TYA
                        } else {
                            self.set_flag(Flag::V, false);
                            Instr::CLV
                        }
                    }
                    3 => {
                        self.set_flag(Flag::D, is_y);
                        if is_y { Instr::SED } else { Instr::CLD }
                    }
                    _ => unreachable!(),
                };
                Instruction::new(pc, &bytes, i, AddressingMode::Implied(), 2)
            }
            "0xx0_1000" => {
                let (i, c) = match x {
                    0 => {
                        self.push(mem, self.status);
                        (Instr::PHP, 3)
                    }
                    1 => {
                        self.status = self.pop(mem);
                        (Instr::PLP, 4)
                    }
                    2 => {
                        self.push(mem, self.a);
                        (Instr::PHA, 3)
                    }
                    3 => {
                        self.a = self.pop(mem);
                        self.update_zn(self.a);
                        (Instr::PLA, 4)
                    }
                    _ => unreachable!(),
                };
                Instruction::new(pc, &bytes, i, AddressingMode::Implied(), c)
            }
            "1xx0_1000" => {
                let i = match x {
                    0 => {
                        self.y = self.y.wrapping_sub(1);
                        self.update_zn(self.y);
                        Instr::DEY
                    }
                    1 => {
                        self.y = self.a;
                        self.update_zn(self.y);
                        Instr::TAY
                    }
                    2 => {
                        self.y = self.y.wrapping_add(1);
                        self.update_zn(self.y);
                        Instr::INY
                    }
                    3 => {
                        self.x = self.x.wrapping_add(1);
                        self.update_zn(self.x);
                        Instr::INX
                    }
                    _ => unreachable!(),
                };
                Instruction::new(pc, &bytes, i, AddressingMode::Implied(), 2)
            }
            "10xx_1010" => {
                let i = match x {
                    0 => {
                        self.a = self.x;
                        self.update_zn(self.a);
                        Instr::TXA
                    }
                    1 => {
                        self.sp = self.x;
                        Instr::TXS
                    }
                    2 => {
                        self.x = self.a;
                        self.update_zn(self.x);
                        Instr::TAX
                    }
                    3 => {
                        self.x = self.sp;
                        self.update_zn(self.x);
                        Instr::TSX
                    }
                    _ => unreachable!(),
                };
                Instruction::new(pc, &bytes, i, AddressingMode::Implied(), 2)
            }
            "11x0_y100" => {
                let (data, c, am) = if y == 0 {
                    let a = self.fetch(mem);
                    (mem.read(a as usize), 3, AddressingMode::ZeroPage(a))
                } else {
                    let a = self.fetch_word(mem);
                    (mem.read(a as usize), 4, AddressingMode::Absolute(a))
                };

                let (i, data2) = if x == 0 {
                    (Instr::CPY, self.y)
                } else {
                    (Instr::CPX, self.x)
                };

                let result = data2.wrapping_sub(data);
                self.update_zn(result);
                self.set_flag(Flag::C, data2 >= data);

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "01x0_1100" => {
                let (addr, am, c) = if x == 0 {
                    let a = self.fetch_word(mem);
                    (a, AddressingMode::Absolute(a), 3)
                } else {
                    let a = self.fetch_word(mem);
                    (mem.read_word(a as usize), AddressingMode::Indirect(a), 5)
                };
                self.pc = addr;
                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, Instr::JMP, am, c)
            }
            "0010_x100" => {
                let (data, am, c) = if x == 0 {
                    let a = self.fetch(mem);
                    (mem.read(a as usize), AddressingMode::ZeroPage(a), 3)
                } else {
                    let a = self.fetch_word(mem);
                    (mem.read(a as usize), AddressingMode::Absolute(a), 4)
                };

                let result = self.a & data;
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::V, result & 0x40 != 0);
                self.set_flag(Flag::N, result & 0x80 != 0);

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, Instr::BIT, am, c)
            }
            "1010_00x0" => {
                let data = self.fetch(mem);
                let am = AddressingMode::Immediate(data);

                let i = if x == 0 {
                    self.y = data;
                    self.update_zn(self.y);
                    Instr::LDY
                } else {
                    self.x = data;
                    self.update_zn(self.x);
                    Instr::LDX
                };

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, 2)
            }
            "xxx1_0000" => {
                let offset = self.fetch(mem);
                let am = AddressingMode::Relative(offset);

                let (branch, i) = match x {
                    0 => (!self.get_flag(Flag::N), Instr::BPL),
                    1 => (self.get_flag(Flag::N), Instr::BMI),
                    2 => (!self.get_flag(Flag::V), Instr::BVC),
                    3 => (self.get_flag(Flag::V), Instr::BVS),
                    4 => (!self.get_flag(Flag::C), Instr::BCC),
                    5 => (self.get_flag(Flag::C), Instr::BCS),
                    6 => (!self.get_flag(Flag::Z), Instr::BNE),
                    7 => (self.get_flag(Flag::Z), Instr::BEQ),
                    _ => unreachable!(),
                };

                let mut c = 2;
                let target = self.pc.wrapping_add(offset as u16);
                if target >> 8 != self.pc >> 8 {
                    c += 1;
                }

                if branch {
                    self.pc = target;
                    c += 1;
                }

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, c)
            }
            "0010_0000" => {
                let addr = self.fetch_word(mem);
                let am = AddressingMode::Absolute(addr);

                self.push_word(mem, self.pc - 1);
                self.pc = addr;

                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, Instr::JSR, AddressingMode::Implied(), 6)
            }
            "0110_0000" => {
                self.pc = self.pop_word(mem) + 1;
                Instruction::new(pc, &bytes, Instr::RTS, AddressingMode::Implied(), 6)
            }
            "0100_0000" => {
                self.status = self.pop(mem);
                self.pc = self.pop_word(mem);
                Instruction::new(pc, &bytes, Instr::RTI, AddressingMode::Implied(), 6)
            }
            "0000_0000" => {
                self.push_word(mem, self.pc + 1);
                self.push(mem, self.status);
                self.pc = mem.read_word(0xFE);
                Instruction::new(pc, &bytes, Instr::BRK, AddressingMode::Implied(), 7)
            }
            "11x0_0000" => {
                let (i, data) = if x == 0 {
                    (Instr::CPY, self.y)
                } else {
                    (Instr::CPX, self.x)
                };

                let data2 = self.fetch(mem);

                let result = data2.wrapping_sub(data);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::V, result & 0x40 != 0);
                self.set_flag(Flag::N, result & 0x80 != 0);

                let am = AddressingMode::Immediate(data2);
                am.add_to_vec(&mut bytes);
                Instruction::new(pc, &bytes, i, am, 2)
            }
            // "000x_x110" => {
            //     let (addr, am, c) = match x {
            //         0 => {
            //             let a = self.fetch(mem);
            //             (a as usize, AddressingMode::ZeroPage(a), 5)
            //         }
            //         1 => {
            //             let a = self.fetch_word(mem);
            //             (a as usize, AddressingMode::Absolute(a), 6)
            //         }
            //         2 => {
            //             let a = self.fetch(mem);
            //             (a.wrapping_add(self.x) as usize, AddressingMode::ZeroPageX(a), 6)
            //         }
            //         3 => {
            //             let a = self.fetch_word(mem);
            //             ((a + self.x as u16) as usize, AddressingMode::AbsoluteX(a), 7)
            //         }
            //         _ => unreachable!(),
            //     };
            //
            //     let mut data = mem.read(addr);
            //     self.set_flag(Flag::C, data & 0x80 != 0);
            //     data <<= 1;
            //     mem.write(addr, data);
            //
            //     self.update_zn(data);
            //
            //     am.add_to_vec(&mut bytes);
            //     Instruction::new(pc, &bytes, Instr::ASL, am, c)
            // }
            // "0000_1010" => {
            //     self.set_flag(Flag::C, self.a & 0x80 != 0);
            //     self.a <<= 1;
            //     self.update_zn(self.a);
            //     Instruction::new(pc, &bytes, Instr::ASL, AddressingMode::Accumulator(), 2)
            // }
            _ => {
                error!("Unknown opcode {:04X}: {:02X}", pc, opcode);
                Instruction::new(pc, &bytes, Instr::UNK, AddressingMode::Implied(), 0)
            }
        }
    }

    fn instr_arithmetic(&mut self, mem: &mut dyn Memory, i: &Instr, am: &AddressingMode) -> usize {
        let (data, mut c) = match i {
            Instr::STA => (self.a, 0),
            _ => match am {
                AddressingMode::Immediate(d) => (*d, 2),
                AddressingMode::ZeroPage(a) => (mem.read(*a as usize), 3),
                AddressingMode::ZeroPageX(a) => (mem.read(a.wrapping_add(self.x) as usize), 4),
                AddressingMode::Absolute(a) => (mem.read(*a as usize), 4),
                AddressingMode::AbsoluteX(a) => {
                    mem.read(*a as usize); // Side-effect
                    let addr = a.wrapping_add(self.x as u16);
                    (
                        mem.read(addr as usize),
                        if addr >> 8 != a >> 8 { 5 } else { 4 },
                    )
                }
                AddressingMode::AbsoluteY(a) => {
                    mem.read(*a as usize); // Side-effect
                    let addr = a.wrapping_add(self.y as u16);
                    (
                        mem.read(addr as usize),
                        if addr >> 8 != a >> 8 { 5 } else { 4 },
                    )
                }
                AddressingMode::IndirectX(a) => (
                    mem.read(mem.read_word_zero(a.wrapping_add(self.x)) as usize),
                    6,
                ),
                AddressingMode::IndirectY(a) => {
                    let addr1 = mem.read_word_zero(*a);
                    let addr2 = addr1 + self.y as u16;
                    (
                        mem.read(addr2 as usize),
                        if addr2 >> 8 != addr1 >> 8 { 6 } else { 5 },
                    )
                }
                _ => unreachable!(),
            },
        };
        match i {
            Instr::ORA => {
                self.a |= data;
                self.update_zn(self.a);
            }
            Instr::AND => {
                self.a &= data;
                self.update_zn(self.a);
            }
            Instr::EOR => {
                self.a ^= data;
                self.update_zn(self.a);
            }
            Instr::ADC => {
                let (sum, carry) = self.a.overflowing_add(data);
                let (_, overflow) = (self.a as i8).overflowing_add(data as i8);
                self.a = sum;
                self.set_flag(Flag::C, carry);
                self.set_flag(Flag::V, overflow);
                self.update_zn(self.a);
            }
            Instr::STA => match am {
                AddressingMode::Immediate(_) => {}
                AddressingMode::ZeroPage(a) => {
                    c = 3;
                    mem.write(*a as usize, data)
                }
                AddressingMode::ZeroPageX(a) => {
                    c = 4;
                    mem.write(a.wrapping_add(self.x) as usize, data)
                }
                AddressingMode::Absolute(a) => {
                    c = 4;
                    mem.write(*a as usize, data)
                }
                AddressingMode::AbsoluteX(a) => {
                    c = 5;
                    mem.write(a.wrapping_add(self.x as u16) as usize, data)
                }
                AddressingMode::AbsoluteY(a) => {
                    c = 5;
                    mem.write(a.wrapping_add(self.y as u16) as usize, data)
                }
                AddressingMode::IndirectX(a) => {
                    c = 6;
                    mem.write(mem.read_word_zero(a.wrapping_add(self.x)) as usize, data)
                }
                AddressingMode::IndirectY(a) => {
                    c = 6;
                    mem.write((mem.read_word_zero(*a) + self.y as u16) as usize, data)
                }
                _ => unreachable!(),
            },
            Instr::LDA => {
                self.a = data;
                self.update_zn(self.a);
            }
            Instr::CMP => {
                let data2 = self.fetch(mem);

                let result = data2.wrapping_sub(data);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::V, result & 0x40 != 0);
                self.set_flag(Flag::N, result & 0x80 != 0);
            }
            Instr::SBC => {
                let x = if self.get_flag(Flag::C) {
                    data
                } else {
                    data + 1
                };
                let (diff, carry) = self.a.overflowing_sub(x);
                let (_, overflow) = (self.a as i8).overflowing_add(x as i8);
                self.a = diff;
                self.set_flag(Flag::C, carry);
                self.set_flag(Flag::V, overflow);
                self.update_zn(self.a);
            }
            _ => unreachable!(),
        }
        c
    }

    fn instr_shift(&mut self, mem: &mut dyn Memory, i: &Instr, am: &AddressingMode) -> usize {
        if let AddressingMode::Accumulator() = am {
            match i {
                Instr::ASL => {
                    self.set_flag(Flag::C, self.a & 0x80 != 0);
                    self.a <<= 1;
                    self.update_zn(self.a);
                    return 2;
                }
                Instr::ROL => {
                    let is_carry = self.a & 0x80 != 0;
                    self.a <<= 1;
                    if self.get_flag(Flag::C) {
                        self.a += 1;
                    }
                    self.set_flag(Flag::C, is_carry);
                    self.update_zn(self.a);
                    return 2;
                }
                Instr::LSR => {
                    let is_carry = self.a & 0x80 != 0;
                    self.a >>= 1;
                    self.set_flag(Flag::C, is_carry);
                    self.update_zn(self.a);
                    return 2;
                }
                Instr::ROR => {
                    let is_carry = self.a & 0x01 != 0;
                    self.a >>= 1;
                    if self.get_flag(Flag::C) {
                        self.a += 0x80;
                    }
                    self.set_flag(Flag::C, is_carry);
                    self.update_zn(self.a);
                    return 2;
                }
                _ => unreachable!(),
            }
        }

        let (addr, c) = match am {
            AddressingMode::ZeroPage(a) => (*a as usize, 5),
            AddressingMode::Absolute(a) => (*a as usize, 6),
            AddressingMode::ZeroPageX(a) => (a.wrapping_add(self.x) as usize, 6),
            AddressingMode::AbsoluteX(a) => (a.wrapping_add(self.x as u16) as usize, 7),
            _ => unreachable!(),
        };

        let mut data = mem.read(addr);

        match i {
            Instr::ASL => {
                self.set_flag(Flag::C, data & 0x80 != 0);
                data <<= 1;
                self.update_zn(data);
            }
            Instr::ROL => {
                let is_carry = data & 0x80 != 0;
                data <<= 1;
                if self.get_flag(Flag::C) {
                    data += 1;
                }
                self.set_flag(Flag::C, is_carry);
                self.update_zn(data);
            }
            Instr::LSR => {
                let is_carry = data & 0x80 != 0;
                data >>= 1;
                self.set_flag(Flag::C, is_carry);
                self.update_zn(data);
            }
            Instr::ROR => {
                let is_carry = data & 0x01 != 0;
                data >>= 1;
                if self.get_flag(Flag::C) {
                    data += 0x80;
                }
                self.set_flag(Flag::C, is_carry);
                self.update_zn(data);
            }
            _ => unreachable!(),
        }

        mem.write(addr, data);
        c
    }
}

/// Status flags for the 6502 CPU
#[derive(Clone, Copy)]
pub enum Flag {
    /// Negative flag (bit 7)
    N = 0x80,
    /// Overflow flag (bit 6)
    V = 0x40,
    /// Unused (bit 5)
    _UNUSED = 0x20,
    /// Break flag (bit 4)
    B = 0x10,
    /// Decimal mode flag (bit 3)
    D = 0x08,
    /// Interrupt disable flag (bit 2)
    I = 0x04,
    /// Zero flag (bit 1)
    Z = 0x02,
    /// Carry flag (bit 0)
    C = 0x01,
}
