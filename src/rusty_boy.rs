use crate::rusty_boy::cpu::CPU;
use std::fs;
use std::path::PathBuf;
use sdl2::keyboard::Keycode;
mod cartridge;
mod cpu;
mod joypad;
mod memory;
mod motherboard;
mod screen;
mod timer;
mod sound;

pub struct RustyBoy {
    cpu: Option<CPU>
}

impl RustyBoy {
    // Init with None as CPU
    pub fn new() -> Self {
        Self {
            cpu: None
        }
    }
    // Inits a new cpu with Rom
    pub fn load_rom(&mut self, rom_file: PathBuf) {
        let bytes: Vec<u8> = fs::read(&rom_file).unwrap();
        self.cpu = Some(CPU::new(bytes));
    }
    
    // handle events
    pub fn handle_events(&mut self, keycode: Option<Keycode>, up: bool) {
        match &mut self.cpu {
            // handle events
            Some(cpu) => {
                let mut joypad = cpu.motherboard.joypad.borrow_mut();
                let mut interrupt = false;
                match keycode {
                    // WASD up left down right
                    Some(Keycode::W) => interrupt = joypad.handle_input(2, up),
                    Some(Keycode::A) => interrupt = joypad.handle_input(1, up),
                    Some(Keycode::S) => interrupt = joypad.handle_input(3, up),
                    Some(Keycode::D) => interrupt = joypad.handle_input(0, up),
                    // A
                    Some(Keycode::K) => interrupt = joypad.handle_input(4, up),
                    // B
                    Some(Keycode::L) => interrupt = joypad.handle_input(5, up),
                    // Select
                    Some(Keycode::I) => interrupt = joypad.handle_input(6, up),
                    // Start
                    Some(Keycode::O) => interrupt = joypad.handle_input(7, up),
                    _ => {}
                }
                if interrupt {
                    cpu.set_interrupt(4);
                }
            }
            None => {}
        }
    }
    // Returns screen buffer vector
    pub fn update_and_render(&mut self) -> Vec<u8> {
        match &mut self.cpu {
            None => {
                [0xff; 160 * 144 * 4].to_vec()
            },
            Some(cpu) => {
                cpu.run_one_frame();
                cpu.motherboard.screen.borrow().screen_buffer.clone()
            }
        }
    }
}
