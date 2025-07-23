use std::time::Instant;
#[derive(PartialEq,Clone,Debug,Eq)]
pub struct RTC {
    latched: bool,
    zero_time: Instant,
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_low: u8,
    day_high: u8
}

impl RTC {
    pub fn new() -> Self {
        Self {
            latched: false,
            zero_time: Instant::now(),
            seconds: 0,
            minutes: 0,
            hours: 0,
            day_low: 0,
            day_high: 0
        }
    }
    
    pub fn read(&self, bank: u8) -> u8 {
        match bank {
            0x08 => self.seconds,
            0x09 => self.minutes,
            0x0a => self.hours,
            0x0b => self.day_low,
            0x0c => self.day_high,
            _ => 0xff
        }
    }
    
    pub fn write(&mut self, bank: u8, value: u8) {
        match bank {
            0x08 => self.seconds = value,
            0x09 => self.minutes = value,
            0x0a => self.hours = value,
            0x0b => self.day_low = value,
            0x0c => self.day_high = value,
            _ => {}
        }
    }
    pub fn write_latch_clock(&mut self, value: u8) {
        if value == 0 {
            self.latched = true;
        } else if value == 1 && self.latched {
            self.latched = false;
            self.latch_clock();
        } else {
            self.latched = false;
        }
    }
    
    fn latch_clock(&mut self) {
        let cur_time_secs = self.zero_time.elapsed().as_secs();
        self.seconds = (cur_time_secs % 60) as u8;
        self.minutes = ((cur_time_secs / 60) % 60) as u8;
        self.hours = ((cur_time_secs / 3600) % 24) as u8;
        
        let days = (cur_time_secs / 3600 / 24) as u16; 
        self.day_low = (days & 0xff) as u8;
        self.day_high = ((days >> 8) & 0b1) as u8;
    }
}