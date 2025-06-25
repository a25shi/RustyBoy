use std::cell::{Cell, RefCell};
use crate::timer::Timer;

pub struct Motherboard {
    pub i_flag: Cell<u8>,
    pub i_master: Cell<bool>,
    pub i_enable: Cell<u8>,
    pub timer: RefCell<Timer>
}

impl Motherboard {
    pub fn new() -> Self {
        Self {
            i_flag: Cell::new(0),
            i_enable: Cell::new(0),
            i_master: Cell::new(false),
            timer: RefCell::new(Timer::new())
        }
    }
}