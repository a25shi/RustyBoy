mod cartridge_header;
mod memory;
mod cpu;
mod registers;

use std::fs;
use std::any::type_name;

fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

fn main() {
    let file: &str = "./roms/super mario.gb";
    // Getting cartridge header data
    let bytes: Vec<u8> = fs::read(file).unwrap();
    let header = cartridge_header::CartridgeHeader::from_bytes(&bytes).unwrap();

    // Getting opcode data
    let opcodedata = fs::read_to_string("./src/opcodes/Opcodes.json").unwrap();
    let opcodes: serde_json::Value = serde_json::from_str(&opcodedata).unwrap();

    println!("{:?}", header);
    println!("{:?}", opcodes["unprefixed"]["0x01"]["cycles"].as_array().unwrap());
    println!("{:?}", format!("{:#04X}", 1));
}
