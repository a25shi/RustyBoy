mod cartridge_header;
mod memory;
mod cpu;
mod registers;
mod cartridge;
mod timer;
mod motherboard;

use std::fs;
use std::any::type_name;
use crate::cpu::CPU;

fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

fn main() {
    let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data
    let bytes: Vec<u8> = fs::read("./roms/blargg tests/cpu_instrs/individual/02-interrupts.gb").unwrap();
    let header = cartridge_header::CartridgeHeader::from_bytes(&bytes).unwrap();

    // Getting opcode data
    let opcodedata = fs::read_to_string("./src/opcodes/Opcodes.json").unwrap();
    let opcodes: serde_json::Value = serde_json::from_str(&opcodedata).unwrap();
    
    let mut cpu = CPU::new(bytes);
    cpu.run();
}
