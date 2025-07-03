use std::rc::{Rc};
use multi_compare::c;
use crate::rusty_boy::cartridge::Cartridge;
use crate::rusty_boy::motherboard::Motherboard;

pub struct Memory {
    pub cartridge: Cartridge,
    // internal high ram
    pub h_ram: [u8; 128],
    // internal ram
    pub i_ram: [u8; 0x2000],
    // serial bus
    sb1: u8,
    sb2: u8,
    // motherboard pointer
    motherboard: Rc<Motherboard>
}

impl Memory {
    pub fn new(rom: Vec<u8>, mobo: &Rc<Motherboard>) -> Self {
        Self {
            cartridge: Cartridge::new(rom).unwrap(),
            motherboard: mobo.clone(),
            h_ram: [0; 128],
            i_ram: [0; 0x2000],
            sb1: 0,
            sb2: 0,
        }
    }
    pub fn get(&self, address: u16) -> u8 {
        // cartridge rom read
        if address < 0x8000 {
            self.cartridge.read(address)
        }
        // vram
        else if c!(0x8000 <= address < 0xa000) {
            self.motherboard.screen.borrow().get(address)
        }
        // cartridge ram
        else if c!(0xa000 <= address < 0xc000) {
            self.cartridge.read(address)
        }
        // internal ram access
        else if c!(0xc000 <= address < 0xe000) {
            self.i_ram[address as usize - 0xc000]
        }
        // echo ram
        else if c!(0xe000 <= address < 0xfe00) {
            self.get(address - 0x2000)
        }
        // oam
        else if c!(0xfe00 <= address < 0xfea0) {
            self.motherboard.screen.borrow().get(address)
        }
        // prohibited region
        else if c!(0xfea0 <= address < 0xff00) {
            0xff
        }
        // I/O registers
        else if c!(0xff00 <= address < 0xff80) {
            self.io_read(address)
        }
        // hram 
        else if c!(0xff80 <= address <= 0xfffe) {
            self.h_ram[address as usize - 0xff80]
        }
        // interrupt enable flag
        else if address == 0xffff {
            self.motherboard.i_enable.get()
        }
        // should never get here
        else {
            unreachable!()
        }
    }
    pub fn set(&mut self, address: u16, value: u8) {
        // cartridge rom read
        if address < 0x8000 {
            self.cartridge.write(address, value);
        }
        // vram
        else if c!(0x8000 <= address < 0xa000) {
            self.motherboard.screen.borrow_mut().set(address, value);
        }
        // cartridge ram
        else if c!(0xa000 <= address < 0xc000) {
            self.cartridge.write(address, value);
        }
        // internal ram access
        else if c!(0xc000 <= address < 0xe000) {
            self.i_ram[address as usize - 0xc000] = value;
        }
        // echo ram
        else if c!(0xe000 <= address < 0xfe00) {
            self.set(address - 0x2000, value);
        }
        // oam
        else if c!(0xfe00 <= address < 0xfea0) {
            self.motherboard.screen.borrow_mut().set(address, value);
        }
        // prohibited region
        else if c!(0xfea0 <= address < 0xff00) {
            
        }
        // I/O registers
        else if c!(0xff00 <= address < 0xff80) {
            self.io_write(address, value);
        }
        // hram 
        else if c!(0xff80 <= address <= 0xfffe) {
            self.h_ram[address as usize - 0xff80] = value;
        }
        // interrupt enable flag
        else if address == 0xffff {
            self.motherboard.i_enable.set(value);
        }
        // should never get here
        else {
            unreachable!()
        }
    }
    
    fn io_read(&self, address: u16) -> u8 {
        match address {
            // joypad
            0xff00 => self.motherboard.joypad.borrow().get_joypad(),
            // serial bus
            0xff01..=0xff02 => {
                if address == 0xff01 {
                    self.sb1
                } else { 
                    self.sb2
                }
            }
            // timer
            0xff04..=0xff07 => {
                self.motherboard.sync();
                self.motherboard.timer.borrow().get(address)
            }
            // interrupt
            0xff0f => {
                self.motherboard.i_flag.get()
            }
            // TODO: audio
            0xff10..=0xff26 => 0xff,
            // TODO: audio ram
            0xff30..=0xff3f => 0xff,
            // screen
            0xff40..=0xff4b => {
                self.motherboard.sync();
                self.motherboard.screen.borrow().get(address)
            }
            _ => 0xff
        }
    }
    
    fn io_write(&mut self, address: u16, value: u8) {
        match address {
            // joypad
            0xff00 => self.motherboard.joypad.borrow_mut().set_joypad(value),
            // serial bus
            0xff01..=0xff02 => {
                if address == 0xff01 {
                    self.sb1 = value
                } else {
                    self.sb2 = value
                }
            }
            // timer
            0xff04..=0xff07 => {
                self.motherboard.sync();
                self.motherboard.timer.borrow_mut().set(address, value);
            }
            // interrupt
            0xff0f => {
                self.motherboard.i_flag.set(value);
            }
            // TODO: audio
            0xff10..=0xff26 => {},
            // TODO: audio ram
            0xff30..=0xff3f => {},
            // screen
            0xff40..=0xff4b => {
                self.motherboard.sync();
                // LCDC set 
                if address == 0xff40 {
                    let mut screen = self.motherboard.screen.borrow_mut();
                    let prev = screen.lcdc.lcd_enable;
                    screen.lcdc.set(value);

                    // If screen is on then turned off
                    if prev && !screen.lcdc.lcd_enable {
                        screen.scan_counter = 0;
                        let inter = screen.stat.set_mode(0);
                        if inter {
                            self.motherboard.set_interrupt(1);
                        }
                        screen.ly = 0;
                    }
                }
                // DMA
                else if address == 0xff46 {
                    self.dma(value);
                }
                // Otherwise screen
                else {
                    self.motherboard.screen.borrow_mut().set(address, value);
                }
            }
            // Do nothing
            _ => {}
        }
    }
    
    fn dma(&mut self, value: u8) {
        let offset: u16 = value as u16 * 0x100;
        for n in 0..0xa0 {
            self.set(0xfe00 + n, self.get(n + offset));
        }
    }
}