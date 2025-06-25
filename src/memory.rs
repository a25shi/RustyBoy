use std::cell::RefCell;
use std::ptr::addr_eq;
use std::rc::{Rc, Weak};
use multi_compare::c;
use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::motherboard::Motherboard;

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
            h_ram: [0xff; 128],
            i_ram: [0xff; 0x2000],
            sb1: 0,
            sb2: 0,
        }
    }
    pub fn get(&self, address: u16) -> u8 {
        // cartridge rom read
        if address < 0x8000 {
            self.cartridge.read(address)
        }
        // TODO: vram
        else if c!(0x8000 <= address < 0xa000) {
            0
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
        // TODO: oam
        else if c!(0xfe00 <= address < 0xfea0) {
            0
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
        // TODO: vram
        else if c!(0x8000 <= address < 0xa000) {
            
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
        // TODO: oam
        else if c!(0xfe00 <= address < 0xfea0) {
            
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
    
    // TODO: implement io reads
    fn io_read(&self, address: u16) -> u8 {
        match address {
            // joypad
            0xff00 => {0}
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
            // TODO: screen
            0xff40..=0xff4b => {
                if address == 0xff44 {
                    0x90
                } else {
                    0xff
                }
            }
            _ => 0xff
        }
    }
    // TODO: implement io writes
    fn io_write(&mut self, address: u16, value: u8) {
        match address {
            // joypad
            0xff00 => {}
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
            // TODO: screen
            0xff40..=0xff4b => {
                // DMA
                if address == 0xff46 {
                    self.dma(value);
                }
                // Otherwise screen
                else {
                    
                }
            },
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