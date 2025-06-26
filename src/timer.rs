
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Timer {
    div: u16,
    div_counter: u16,
    tac: u8,
    tma: u8,
    tima: u8,
    pub counter: isize
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xad,
            div_counter: 0,
            tac: 0,
            tma: 0,
            tima: 0,
            counter: 1024
        }
    }
    pub fn get(&self, address: u16) -> u8 {
        match address {  
            0xff04 => self.div as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => unreachable!()
        }
    }

    pub fn set(&mut self, address: u16, value: u8) {
        match address {
            0xff04 => self.reset(),
            0xff05 => self.tima = value,
            0xff06 => { 
                self.tma = value;
            },
            0xff07 => { 
                let temp = self.tac;
                self.tac = value & 0b111;
                // reset counter on new tac
                if temp != self.tac {
                    self.reset_counter();
                }
            }
            _ => unreachable!()
        }
    }
    
    pub fn tick(&mut self, cycles: u8) -> bool {
        // dont bother on empty cycles
        if cycles == 0 {
            return false
        }
        // iterate timer div (div and div_counter should be u8, but are u16 to keep track of overflow)
        self.div_counter += cycles as u16;
        self.div += self.div_counter >> 8;
        self.div_counter &= 0xff;
        self.div &= 0xff;

        // check timer enabled
        if self.tac & 0b100 == 0 {
            return false;
        }
        
        self.counter -= cycles as isize;
        if self.counter <= 0 {
            // reset timer on overflow
            self.counter += self.get_freq();
            let (res, ov) = self.tima.overflowing_add(1);
            self.tima = res;
            // On tima overflow, trigger interrupt
            if ov {
                self.tima = self.tma;
                return true;
            }
        }
        false
    }
    
    // gets the clock freq
    fn get_freq(&self) -> isize {
        let c_select = self.tac & 0b11;
        match c_select {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!()
        }
    }
    
    // resets counter
    fn reset_counter(&mut self) {
        self.counter = self.get_freq();
    }
    
    // reset timer
    fn reset(&mut self) {
        self.div = 0;
        self.div_counter = 0;
        self.reset_counter();
    }
}