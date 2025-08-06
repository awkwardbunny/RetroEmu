use crate::cpu::mos6502::AddressingMode;

#[derive(Debug, PartialEq)]
pub enum Instr {
    LDA, LDX, LDY, STA, STX, STY,
    TAX, TAY, TSX, TXA, TXS, TYA,
    PHA, PHP, PLA, PLP,
    ASL, LSR, ROL, ROR,
    AND, BIT, EOR, ORA,
    ADC, CMP, CPX, CPY, SBC,
    DEC, DEX, DEY, INC, INX, INY,
    BRK, JMP, JSR, RTI, RTS,
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,
    CLC, CLD, CLI, CLV, SEC, SED, SEI,
    NOP, UNK
}

impl std::fmt::Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(PartialEq)]
pub struct Instruction {
    pub cycles: usize,
    pub pc: u16,
    pub bytes: Vec<u8>,
    pub instr: Instr,
    pub addrmode: AddressingMode,
}

fn to_hex_string(bytes: &Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join(" ")
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04X}: {:12} ; {} {}",
            self.pc,
            to_hex_string(&self.bytes),
            self.instr,
            self.addrmode,
        )
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04X}: {:12} ; {} {}  (#{})",
            self.pc,
            to_hex_string(&self.bytes),
            self.instr,
            self.addrmode,
            self.cycles,
        )
    }
}

impl Instruction {
    pub fn new(pc: u16, bytes: &Vec<u8>, instr: Instr, addrmode: AddressingMode, cycles: usize) -> Self {
        Self {
            cycles,
            pc,
            bytes: bytes.clone(),
            instr,
            addrmode,
        }
    }

    pub fn cycle(&mut self) -> bool {
        self.cycles = self.cycles.saturating_sub(1);
        self.cycles == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::mos6502::AddressingMode;
    use crate::cpu::mos6502::instruction::{Instr, Instruction};

    #[test]
    fn instruction_print() {
        let x = Instruction::new(0xAA, &vec![0xA9 as u8, 0xFF as u8], Instr::LDA, AddressingMode::Immediate(0xFF), 4);
        assert_eq!(x.to_string(), "00AA: A9 FF        ; LDA #$FF");
    }
}