use std::cell::{Cell, RefCell};
use std::rc::Rc;
use crate::screen::Screen;
use crate::timer::Timer;
use crate::joypad::Joypad;

// Note: Since motherboard doesn't have any mutable fields, always borrow as non mut to access fields
pub struct Motherboard {
    pub i_flag: Cell<u8>,
    pub i_master: Cell<bool>,
    pub i_enable: Cell<u8>,
    pub cycles: Cell<u8>,
    pub sync_cycles: Cell<u8>,
    pub timer: RefCell<Timer>,
    pub screen: RefCell<Screen>,
    pub joypad: RefCell<Joypad>
}

impl Motherboard {
    pub fn new() -> Rc<Self> {
        // creates a cyclic reference with memory using weak pointer
        Rc::new_cyclic(|x| {Self {
            i_flag: Cell::new(0),
            i_enable: Cell::new(0),
            cycles: Cell::new(0),
            sync_cycles: Cell::new(0),
            i_master: Cell::new(false),
            timer: RefCell::new(Timer::new()),
            screen: RefCell::new(Screen::new(x.clone())),
            joypad: RefCell::new(Joypad::new())
        }})
    }
    
    // sets interrupt flag
    pub fn set_interrupt(&self, bit: u8) {
        let flag = 1 << bit;
        self.i_flag
            .set(self.i_flag.get() | flag);
    }
    pub fn sync(&self) {
        // timer tick
        if self.timer.borrow_mut().tick(self.cycles.get()) {
            self.set_interrupt(2);
        }
        // screen tick
        self.screen.borrow_mut().update(self.cycles.get());
        
        // sync
        self.sync_cycles.set(self.sync_cycles.get() + self.cycles.get());
        
        // reset
        self.cycles.set(0);
    }
}