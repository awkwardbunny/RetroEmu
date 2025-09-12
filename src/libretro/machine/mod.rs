mod apple_ii_e;
pub mod apple_iie_e_display;
mod apple_ii_e_string;

pub use apple_ii_e::AppleIIe;

pub trait Machine {
    fn reset(&mut self);
    fn cycle(&mut self);
    fn step(&mut self);

    fn read(&self, addr: usize) -> u8;
    fn write(&mut self, addr: usize, data: u8);
}