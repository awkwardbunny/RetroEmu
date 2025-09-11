#[cfg(test)]
mod tests {
use crate::memory::{RAM, Endian, Memory};
use crate::cpu::mos6502::*;
use rstest::*;

    use crate::cpu::mos6502::instruction::{Instr, Instruction};
    // use super::*;

    #[fixture]
    fn mem() -> RAM {
        RAM::new(0x10000, Endian::Little)
    }

    #[fixture]
    fn cpu() -> MOS6502 {
        MOS6502::new()
    }

    #[rstest]
    fn cpu_reset(mut cpu: MOS6502, mut mem: RAM) {
        mem.write_word(0xFFFC, 0xABCD);
        cpu.reset(&mut mem);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.pc, 0xABCD);
        assert_eq!(cpu.status, 0x04);
        assert_eq!(cpu.cycles, 0);
        assert_eq!(cpu.steps, 0);
        // assert_eq!(cpu.current, None);
    }

    fn exec(cpu: &mut MOS6502, mem: &mut RAM, bytes: &[u8]) -> Instruction {
        let start = cpu.cycles;
        for (i, b) in bytes.iter().enumerate() {
            mem.write(i, *b);
        }
        let mut instr = cpu.step(mem);
        let end = cpu.cycles;
        instr.bytes.clear();
        instr.cycles = end - start;
        instr
    }

    fn instr(i: Instr, am: AddressingMode, c: usize) -> Instruction {
        Instruction::new(0, &vec![], i, am, c)
    }

    #[rstest]
    fn instr_lda_imm(mut cpu: MOS6502, mut mem: RAM) {
        let i = exec(&mut cpu, &mut mem, &[0xA9, 0xFF]);
        assert_eq!(instr(Instr::LDA, AddressingMode::Immediate(0xFF), 2), i); // Decode check
        assert_eq!(cpu.a, 0xFF); // Behavior check
    }

    #[rstest]
    fn instr_lda_abs(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        let i = exec(&mut cpu, &mut mem, &[0xAD, 0xEF, 0xBE]);
        assert_eq!(instr(Instr::LDA, AddressingMode::Absolute(0xBEEF), 4), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_abs_x(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.x = 0xEF;
        let i = exec(&mut cpu, &mut mem, &[0xBD, 0x00, 0xBE]);
        assert_eq!(instr(Instr::LDA, AddressingMode::AbsoluteX(0xBE00), 4), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_abs_x_page_boundary(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.x = 0xF0;
        let i = exec(&mut cpu, &mut mem, &[0xBD, 0xFF, 0xBD]);
        assert_eq!(instr(Instr::LDA, AddressingMode::AbsoluteX(0xBDFF), 5), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_abs_y(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.y = 0xEF;
        let i = exec(&mut cpu, &mut mem, &[0xB9, 0x00, 0xBE]);
        assert_eq!(instr(Instr::LDA, AddressingMode::AbsoluteY(0xBE00), 4), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_abs_y_page_boundary(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.y = 0xF0;
        let i = exec(&mut cpu, &mut mem, &[0xB9, 0xFF, 0xBD]);
        assert_eq!(instr(Instr::LDA, AddressingMode::AbsoluteY(0xBDFF), 5), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_zero(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xEF, 0xFA);
        let i = exec(&mut cpu, &mut mem, &[0xA5, 0xEF]);
        assert_eq!(instr(Instr::LDA, AddressingMode::ZeroPage(0xEF), 3), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_zero_x(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0x10, 0xFA);
        cpu.x = 0x11;
        let i = exec(&mut cpu, &mut mem, &[0xB5, 0xFF]);
        assert_eq!(instr(Instr::LDA, AddressingMode::ZeroPageX(0xFF), 4), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_indirect_x(mut cpu: MOS6502, mut mem: RAM) {
        mem.write_word(0xBEEF, 0xFA);
        mem.write(0x10, 0xEF);
        mem.write(0x11, 0xBE);
        cpu.x = 0x10;
        let i = exec(&mut cpu, &mut mem, &[0xA1, 0x00]);
        assert_eq!(instr(Instr::LDA, AddressingMode::IndirectX(0x00), 6), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_indirect_y(mut cpu: MOS6502, mut mem: RAM) {
        mem.write_word(0xBEEF, 0xFA);
        mem.write(0x10, 0xE0);
        mem.write(0x11, 0xBE);
        cpu.y = 0x0F;
        let i = exec(&mut cpu, &mut mem, &[0xB1, 0x10]);
        assert_eq!(instr(Instr::LDA, AddressingMode::IndirectY(0x10), 5), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_lda_indirect_y_page_boundary(mut cpu: MOS6502, mut mem: RAM) {
        mem.write_word(0xBEEF, 0xFA);
        mem.write(0x10, 0xF0);
        mem.write(0x11, 0xBD);
        cpu.y = 0xFF;
        let i = exec(&mut cpu, &mut mem, &[0xB1, 0x10]);
        assert_eq!(instr(Instr::LDA, AddressingMode::IndirectY(0x10), 6), i); // Decode check
        assert_eq!(cpu.a, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_ldx_imm(mut cpu: MOS6502, mut mem: RAM) {
        let i = exec(&mut cpu, &mut mem, &[0xA2, 0xFF]);
        assert_eq!(instr(Instr::LDX, AddressingMode::Immediate(0xFF), 2), i); // Decode check
        assert_eq!(cpu.x, 0xFF); // Behavior check
    }

    #[rstest]
    fn instr_ldx_abs(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        let i = exec(&mut cpu, &mut mem, &[0xAE, 0xEF, 0xBE]);
        assert_eq!(instr(Instr::LDX, AddressingMode::Absolute(0xBEEF), 4), i); // Decode check
        assert_eq!(cpu.x, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_ldx_abs_y(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.y = 0xEF;
        let i = exec(&mut cpu, &mut mem, &[0xBE, 0x00, 0xBE]);
        assert_eq!(instr(Instr::LDX, AddressingMode::AbsoluteY(0xBE00), 4), i); // Decode check
        assert_eq!(cpu.x, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_ldx_abs_y_page_boundary(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xBEEF, 0xFA);
        cpu.y = 0xF0;
        let i = exec(&mut cpu, &mut mem, &[0xBE, 0xFF, 0xBD]);
        assert_eq!(instr(Instr::LDX, AddressingMode::AbsoluteY(0xBDFF), 5), i); // Decode check
        assert_eq!(cpu.x, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_ldx_zero(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0xEF, 0xFA);
        let i = exec(&mut cpu, &mut mem, &[0xA6, 0xEF]);
        assert_eq!(instr(Instr::LDX, AddressingMode::ZeroPage(0xEF), 3), i); // Decode check
        assert_eq!(cpu.x, 0xFA); // Behavior check
    }

    #[rstest]
    fn instr_ldx_zero_y(mut cpu: MOS6502, mut mem: RAM) {
        mem.write(0x20, 0xFA);
        cpu.y = 0x10;
        let i = exec(&mut cpu, &mut mem, &[0xB6, 0x10]);
        assert_eq!(instr(Instr::LDX, AddressingMode::ZeroPageY(0x10), 4), i); // Decode check
        assert_eq!(cpu.x, 0xFA); // Behavior check
    }
}
