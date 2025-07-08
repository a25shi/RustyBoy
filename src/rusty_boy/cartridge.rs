use cartridge_header::CartridgeHeader;

mod cartridge_header;
mod mbc;

use mbc::{MBCType, MBC0, MBC1};

const RAM_SIZE: &[usize] = &[
    0,
    0x800, // supposed to be disabled and unused, but pan docs says that it uses 2 KiB?
    0x2000, // 1 bank
    4 * 0x2000, // 4 banks
    16 * 0x2000, // 16 banks (what game needs this much?)
    8 * 0x2000, // 8 banks
];

const RAM_BANKS: &[u16] = &[
    0,
    0,
    1,
    4,
    16,
    8
];


#[derive(Debug, Eq, Clone, PartialEq)]
pub struct Cartridge {
    // Cart header
    pub header: CartridgeHeader,
    // Game cartridge data aka rom
    pub rom: Vec<u8>,
    // cartridge ram
    pub ram: Vec<u8>,
    // MBC
    mbc: MBCType
}

impl Cartridge {
    // constructor
    pub fn new(rom: Vec<u8>) -> Result<Self, String> {
        let header = CartridgeHeader::from_bytes(&rom).unwrap();
        let ram_banks = RAM_BANKS[header.ram_size as usize];
        let rom_banks = 2_u16.pow((header.rom_size + 1) as u32);
        let mbc = match &header.cartridge_type {
            0 | 8 | 9 => MBCType::MBC0(MBC0{}),
            1..=3 => MBCType::MBC1(MBC1::new(rom_banks, ram_banks)),
            _ => unimplemented!()
        };
        // TODO: add mbc2 control and none found ram size
        let ram_size = RAM_SIZE[header.ram_size as usize];
        
        let cartridge = Self {
            header,
            rom,
            ram: vec![0; ram_size],
            mbc
        };
        
        Ok(cartridge)
    }
    
    pub fn read(&self, address: u16) -> u8 {
        match &self.mbc {
            MBCType::MBC0(x) => x.read(address, &self.rom, &self.ram),
            MBCType::MBC1(x) => x.read(address, &self.rom, &self.ram),
        }
    }
    pub fn write(&mut self, address: u16, value: u8) {
        match &mut self.mbc {
            MBCType::MBC0(x) => x.write(address, value, &self.rom, &mut self.ram),
            MBCType::MBC1(x) => x.write(address, value, &self.rom, &mut self.ram),
        }
    }
}

