use std::cell::{Cell, RefCell};
use crate::screen::Screen;
use crate::timer::Timer;

pub struct Motherboard {
    pub i_flag: Cell<u8>,
    pub i_master: Cell<bool>,
    pub i_enable: Cell<u8>,
    pub cycles: Cell<u8>,
    pub sync_cycles: Cell<u8>,
    pub timer: RefCell<Timer>,
    pub screen: RefCell<Screen>
}

impl Motherboard {
    pub fn new() -> Self {
        Self {
            i_flag: Cell::new(0),
            i_enable: Cell::new(0),
            cycles: Cell::new(0),
            sync_cycles: Cell::new(0),
            i_master: Cell::new(false),
            timer: RefCell::new(Timer::new()),
            screen: RefCell::new(Screen::new())
        }
    }
    
    // sets interrupt flag
    fn set_interrupt(&self, bit: u8) {
        let flag = 1 << bit;
        self.i_flag
            .set(self.i_flag.get() | flag);
    }
    
    pub fn sync(&self) {
        // timer tick
        if self.timer.borrow_mut().tick(self.cycles.get()) {
            self.set_interrupt(2);
        }
        
        //todo screen tick
        
        // sync
        self.sync_cycles.set(self.sync_cycles.get() + self.cycles.get());
        
        // reset
        self.cycles.set(0);
    }
}