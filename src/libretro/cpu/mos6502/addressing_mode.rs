use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Addressing modes for the 6502 CPU
#[derive(Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Implied(),
    Accumulator(),
    Immediate(u8),
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    Indirect(u16),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    IndirectX(u8),
    IndirectY(u8),
    Relative(u8),
}

impl Display for AddressingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AddressingMode::Implied() => write!(f, ""),
            AddressingMode::Accumulator() => write!(f, "A"),
            AddressingMode::Immediate(d) => write!(f, "#${:02X}", d),
            AddressingMode::Absolute(a) => write!(f, "${:04X}", a),
            AddressingMode::AbsoluteX(a) => write!(f, "${:04X},X", a),
            AddressingMode::AbsoluteY(a) => write!(f, "${:04X},Y", a),
            AddressingMode::Indirect(a) => write!(f, "(${:04X})", a),
            AddressingMode::ZeroPage(a) => write!(f, "${:02X}", a),
            AddressingMode::ZeroPageX(a) => write!(f, "${:02X},X", a),
            AddressingMode::ZeroPageY(a) => write!(f, "${:02X},Y", a),
            AddressingMode::IndirectX(a) => write!(f, "(${:02X},X)", a),
            AddressingMode::IndirectY(a) => write!(f, "(${:02X}),Y", a),
            AddressingMode::Relative(offset) => write!(f, "*{:+}", (*offset).cast_signed()+2),
            // AddressingMode::Relative(offset) => {
            //     let neg = offset & 0x80 != 0;
            //     write!(f, "*{}{:X}", if neg { "-" } else { "+" }, (*offset).cast_signed().abs())
            // },
        }
    }
}

impl Debug for AddressingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl AddressingMode {
    pub fn add_to_vec(self, v: &mut Vec<u8>) {
        match self {
            AddressingMode::Immediate(d) => v.push(d),
            AddressingMode::Absolute(a) => { v.push((a & 15) as u8); v.push((a >> 8) as u8) },
            AddressingMode::AbsoluteX(a) => { v.push((a & 15) as u8); v.push((a >> 8) as u8) },
            AddressingMode::AbsoluteY(a) => { v.push((a & 15) as u8); v.push((a >> 8) as u8) },
            AddressingMode::Indirect(a) => { v.push((a & 15) as u8); v.push((a >> 8) as u8) },
            AddressingMode::ZeroPage(a) => v.push(a),
            AddressingMode::ZeroPageX(a) => v.push(a),
            AddressingMode::ZeroPageY(a) => v.push(a),
            AddressingMode::IndirectX(a) => v.push(a),
            AddressingMode::IndirectY(a) => v.push(a),
            AddressingMode::Relative(r) => v.push(r),
            _ => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::mos6502::AddressingMode;

    #[test]
    fn addressing_mode_print() {
        assert_eq!(AddressingMode::Implied().to_string(), "");
        assert_eq!(AddressingMode::Accumulator().to_string(), "A");
        assert_eq!(AddressingMode::Immediate(0xFF).to_string(), "#$FF");
        assert_eq!(AddressingMode::Absolute(0x1234).to_string(), "$1234");
        assert_eq!(AddressingMode::AbsoluteX(0x1234).to_string(), "$1234,X");
        assert_eq!(AddressingMode::AbsoluteY(0x1234).to_string(), "$1234,Y");
        assert_eq!(AddressingMode::ZeroPage(0xAA).to_string(), "$AA");
        assert_eq!(AddressingMode::ZeroPageX(0xAA).to_string(), "$AA,X");
        assert_eq!(AddressingMode::ZeroPageY(0xAA).to_string(), "$AA,Y");
        assert_eq!(AddressingMode::Indirect(0x1234).to_string(), "($1234)");
        assert_eq!(AddressingMode::IndirectX(0xCC).to_string(), "($CC,X)");
        assert_eq!(AddressingMode::IndirectY(0xEE).to_string(), "($EE),Y");
        assert_eq!(AddressingMode::Relative(2).to_string(), "*+4");
        assert_eq!(AddressingMode::Relative(0xFE).to_string(), "*+0");
        assert_eq!(AddressingMode::Relative(0xF0).to_string(), "*-14");
    }
}
