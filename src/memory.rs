use multi_compare::c;
pub struct Memory {
    // Game cartridge data
    pub cartridge: Vec<u8>,
    // cartridge ram
    pub c_ram: [u8; 0x8000],
    // internal high ram
    pub h_ram: [u8; 128],
    // internal ram
    pub i_ram: [u8; 0x2000],
    // rom bank 
    pub rom_bank: u16,
    // ram bank
    pub ram_bank: u16,
    // memory mode
    pub mode: bool,
    // ram enable
    pub ram_enabled: bool,
    // total ram banks
    pub total_ram_banks: u16,
    // total rom banks
    pub total_rom_banks: u16,
    // bit mask for registers
    pub bank_bits: u16,
    // mbc type
    pub mbc: u8
}

impl Memory {
    pub fn get(&self, address: u16) -> u8 {
        // if address < 0x4000 {
        //    
        // }
        // else if c!(0x4000 <= address < 0x8000) {
        //     
        // }
        // else {
        //     
        // }
        0
    }
    pub fn set(&self, address: u16, value: u8) {
        
    }
}