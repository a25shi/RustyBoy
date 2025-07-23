use multi_compare::c;
use crate::rusty_boy::cartridge::rtc::RTC;

// enum for mbc types, mbc0 and mbc1 for now
#[derive(PartialEq, Clone, Debug, Eq)]
pub enum MBCType {
    MBC0(MBC0),
    MBC1(MBC1),
    MBC3(MBC3)
}

// MBC0 memory controller
#[derive(PartialEq,Clone,Debug,Eq)]
pub struct MBC0 {}
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
pub struct MBC1 {
    rom_bank: u8,
    total_rom_banks: u16,
    total_ram_banks: u16,
    ram_enabled: bool,
    mode: bool,
}

impl MBC1 {
    pub fn new(t_rom_banks: u16, t_ram_banks: u16) -> Self {
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

#[derive(PartialEq,Clone,Debug,Eq)]
pub struct MBC3 {
    rtc: RTC,
    rom_bank: u8,
    ram_bank: u8,
    total_rom_banks: u16,
    total_ram_banks: u16,
    ram_enabled: bool,
}

impl MBC3 {
    pub fn new(t_rom_banks: u16, t_ram_banks: u16) -> Self {
        Self {
            rtc: RTC::new(),
            rom_bank: 1,
            ram_bank: 0,
            total_rom_banks: t_rom_banks,
            total_ram_banks: t_ram_banks,
            ram_enabled: false,
        }
    }
    pub fn read(&self, address: u16, rom: &[u8], ram: &[u8]) -> u8 {
        if address < 0x4000 {
            rom[address as usize]
        }
            
        else if c!(0x4000 <= address < 0x8000) {
            let cur_rom_bank = self.rom_bank as usize % self.total_rom_banks as usize;
            let offset = cur_rom_bank * 0x4000;
            rom[offset + (address as usize - 0x4000)]
        }
        // ram bank - or - rtc register read
        else if c!(0xa000 <= address < 0xc000) {
            match self.ram_bank {
                // Ram read
                0x00..=0x07 => {
                    if !self.ram_enabled || ram.is_empty() {
                        return 0xff
                    }
                    let cur_ram_bank = self.ram_bank as usize % self.total_ram_banks as usize;
                    let final_addr = (cur_ram_bank * 0x2000 + (address as usize - 0xa000)) % ram.len();
                    ram[final_addr]
                }
                // rtc read
                0x08..=0x0c => {
                    self.rtc.read(self.ram_bank)
                }
                // unknown
                _ => 0xff
            }
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
        // rom bank
        else if c!(0x2000 <= address < 0x4000) {
            let mut temp = value & 0b01111111;
            if temp == 0 {
                temp = 1;
            }
            self.rom_bank = temp;
        }

        // ram bank - or - rtc register select
        else if c!(0x4000 <= address < 0x6000) {
            self.ram_bank = value;
        }

        // latch clock data
        else if c!(0x6000 <= address < 0x8000) {
            self.rtc.write_latch_clock(value);
        }
            
        // ram bank - or - rtc register write
        else if c!(0xa000 <= address <= 0xbfff) {
            match self.ram_bank {
                // Ram write
                0x00..=0x07 => {
                    if !self.ram_enabled || ram.is_empty() {
                        return;
                    }
                    let cur_ram_bank = self.ram_bank as usize % self.total_ram_banks as usize;
                    let final_addr = (cur_ram_bank * 0x2000 + (address as usize - 0xa000)) % ram.len();
                    ram[final_addr] = value;
                }
                // rtc write
                0x08..=0x0c => {
                    self.rtc.write(self.ram_bank, value);
                }
                // unknown
                _ => {}
            }
        }
    }
}