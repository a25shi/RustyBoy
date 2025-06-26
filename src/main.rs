mod cartridge_header;
mod memory;
mod cpu;
mod cartridge;
mod timer;
mod motherboard;
mod screen;

use std::fs;
use std::any::type_name;
use crate::cpu::CPU;

fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

fn main() {
    let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data
    let bytes: Vec<u8> = fs::read("./roms/blargg tests/cpu_instrs/cpu_instrs.gb").unwrap();
    
    let mut cpu = CPU::new(bytes);
    cpu.run();
}
