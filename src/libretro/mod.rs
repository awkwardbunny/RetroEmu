/**
 * 6502 Emulator Module
 *
 * This module provides a comprehensive 6502 CPU emulator with support for
 * all official opcodes, memory management, interrupt handling, and debugging features.
 */

pub mod memory;
pub mod cpu;
pub mod machine;
// pub mod debug;
// pub mod tests;

// pub use cpu::MOS6502;
// pub use memory::Memory;
// pub use debug::Debugger;


pub enum DisplayCommand {
    Write(usize, u8),
    Redraw,
    Exit(u8),
}

pub enum EmulatorCommand {
    Cycle,
    Step,
    Run,
    Stop,
    Reset,
}

