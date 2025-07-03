use crate::rusty_boy::cartridge_header::CartridgeHeader;
use multi_compare::c;


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

// enum for mbc types, mbc0 and mbc1 for now
#[derive(PartialEq, Clone, Debug, Eq)]
enum MBCType {
    MBC0(MBC0),
    MBC1(MBC1)
}

// MBC0 memory controller
#[derive(PartialEq,Clone,Debug,Eq)]
struct MBC0 {}
impl MBC0 {
    pub fn read(&self, address: u16, rom: &[u8], ram: &[u8]) -> u8{
        // Rom read
        if address < 0x8000 {
            rom[address as usize]
        }
        // ram read
        else if c!(0xa000 <= address < 0xc000) {
            ram[address as usize - 0xa000]
        }
        else {
            unreachable!()
        }
    }

    pub fn write(&self, address: u16, value: u8, rom: &[u8], ram: &mut [u8]) {
        // Rom write (shouldnt be possible)
        if address < 0x8000 {}
        // ram write
        else if c!(0xa000 <= address < 0xc000) {
            ram[address as usize - 0xa000] = value
        }
        else {
            unreachable!()
        }
    }
}

// MBC1 memory controller
#[derive(PartialEq,Clone,Debug,Eq)]
struct MBC1 {
    rom_bank: u8,
    total_rom_banks: u16,
    total_ram_banks: u16,
    ram_enabled: bool,
    mode: bool,
}

impl MBC1 {
    fn new(t_rom_banks: u16, t_ram_banks: u16) -> Self {
        Self {
            rom_bank: 1,
            total_rom_banks: t_rom_banks,
            total_ram_banks: t_ram_banks,
            ram_enabled: false,
            mode: false
        }
    }
    // bank_num is the total number of banks in the rom
    fn get_banks(&self) -> (u16, u16) {
        // mod full bank within bank range
        let mod_bank = (self.rom_bank as usize % self.total_rom_banks as usize) as u16;
        // lower bank is upper 2 bits on memory mode 1
        let lower_bank = if self.mode {self.rom_bank & 0b1100000} else {0} as u16;
        (lower_bank, mod_bank)
    }
    pub fn read(&self, address: u16, rom: &[u8], ram: &[u8]) -> u8 {
        // lower cartridge bank read
        if address < 0x4000 {
            let lower_bank = self.get_banks().0 as usize;
            let offset = (lower_bank * 0x4000) % rom.len();
            rom[offset + address as usize]
        }

        // upper cartridge read
        else if c!(0x4000 <= address < 0x8000) {
            let bank = self.get_banks().1;
            let addr = (address - 0x4000) as usize;
            let offset = bank as usize * 0x4000;
            rom[offset + addr]
        }

        // ram bank 00 - 03
        else if c!(0xa000 <= address < 0xc000) {
            if !self.ram_enabled || ram.is_empty() {
                return 0xff
            }
            // upper 2 bits, must shift down to get our ram bank number
            let ram_bank = self.get_banks().0 >> 5;
            let addr = (address - 0xa000) as usize;
            let offset = ram_bank as usize * 0x2000;
            let final_addr = (addr + offset) % ram.len();
            ram[final_addr]
        }

        else {
            unreachable!()
        }
    }
    pub fn write(&mut self, address: u16, value: u8, rom: &[u8], ram: &mut [u8]) {
        // ram enable
        if address < 0x2000 {
            self.ram_enabled = value & 0xf == 0xa;
        }
        if c!(0x2000 <= address < 0x4000) {
            // take lower 5 bits of rom_bank
            let mut temp = value & 0b00011111;
            if temp == 0 {
                temp = 1;
            }

            // Take upper 2 bits and lower 5 bits to combine
            self.rom_bank = (self.rom_bank & 0b1100000) | temp;
        }

        if c!(0x4000 <= address < 0x6000) {
            // clear upper 3 bits and combine with the 2 bit write of value
            self.rom_bank = (self.rom_bank & 0x1f) | ((value & 0x3) << 5)
        }

        // mode change
        if c!(0x6000 <= address < 0x8000) {
            self.mode = value & 0b1 != 0;
        }

        // ram write
        if c!(0xa000 <= address <= 0xbfff) {
            if !self.ram_enabled || ram.is_empty() {
                return;
            }

            let ram_bank = self.get_banks().0 >> 5;
            let addr = (address - 0xa000) as usize;
            let offset = ram_bank as usize * 0x2000;
            let final_addr = (addr + offset) % ram.len();
            ram[final_addr] = value;
        }
    }
}