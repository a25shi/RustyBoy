const WAVE_DUTY_TABLE: [u8; 4] = [0b0000_0001, 0b0000_0011, 0b0000_1111, 0b1111_1100];
const CH3_SHIFT_TABLE: [u8; 4] = [4, 0, 1, 2];

fn get_wave_duty(duty: u8, duty_pos: u8) -> u8 {
    (WAVE_DUTY_TABLE[duty as usize] >> duty_pos) & 0b1
}

fn run_env(timer: &mut u8, cur_vol: &mut u8, nx2: u8) {
    let env_dir = nx2 & 0b1000 != 0;
    let env_period = nx2 & 0b111;
    if env_period != 0 {
        if *timer > 0 {
            *timer -= 1;
        }
        if *timer == 0 {
            *timer = env_period;
            if (*cur_vol < 0xf && env_dir) || (*cur_vol > 0 && !env_dir) {
                if env_dir {
                    *cur_vol += 1;
                } else {
                    *cur_vol -= 1;
                }
            }
        }
    }
}

pub struct CH1 {
    // registers
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

impl CH1 {
    
}


// Sound channel 2
pub struct CH2 {
    // registers
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    // none accessible registers
    enable: bool,
    len_timer: u8,
    freq_timer: u16,
    env_timer: u8,
    wave_duty_pos: u8,
    
    // supposed to be 4 bits
    cur_vol: u8,
}

impl CH2 {
    // length function, disables channel
    pub fn tick_len(&mut self) {
        // check length enabled
        let enabled = self.nr24 & 0b1000000;
        if enabled != 0 && self.len_timer > 0 {
            self.len_timer -= 1;
            if self.len_timer == 0 {
                self.enable = false;
            }
        }
    }

    // envelope function
    pub fn tick_env(&mut self) {
        run_env(&mut self.env_timer, &mut self.cur_vol, self.nr22);
    }

    // runs tick for amount of cycles
    pub fn tick(&mut self, cycles: u8) {
        // freq timer max value should be 8192? About 14 bits so signed should be safe
        let mut freq_timer_signed = self.freq_timer as i16;
        freq_timer_signed -= cycles as i16;
        
        while freq_timer_signed <= 0 {
            // Frequency is lower 3 bits of nr24 and 8 bits of nr23 combined ( max value is 2047 )
            let freq = u16::from_be_bytes([self.nr24, self.nr23]) & 0x7ff;
            // keep overflow 
            freq_timer_signed += ((2048 - freq) * 4) as i16;
            self.wave_duty_pos = (self.wave_duty_pos + 1) % 8;
        }
        
        // Set back to value (shouldn't be signed anymore since we broke out the loop?)
        self.freq_timer = freq_timer_signed as u16;
    }

    // gets the current amp from the channel
    pub fn get_amp(&self) -> u8 {
        if self.enable {
            let duty = (self.nr21 >> 6) & 0b11;
            get_wave_duty(duty, self.wave_duty_pos) * self.cur_vol
        }
        else {
            0
        }
    }
}

pub struct CH3 {
    // wave ram
    wave_ram: [u8; 16],
    
    // registers
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    
    // none accessible registers
    enable: bool,
    len_timer: u8,
    freq_timer: u16,
    vol_shift: u8,
    // selects wave ram index
    wave_duty_pos: u8,
}

impl CH3 {
    pub fn tick_len(&mut self) {
        // check length enabled
        let enabled = self.nr34 & 0b1000000;
        if enabled != 0 && self.len_timer > 0 {
            self.len_timer -= 1;
            if self.len_timer == 0 {
                self.enable = false;
            }
        }
    }
    
    pub fn tick(&mut self, cycles: u8) {
        // freq timer max value should be 4096? About 14 bits so signed should be safe
        let mut freq_timer_signed = self.freq_timer as i16;
        freq_timer_signed -= cycles as i16;

        while freq_timer_signed <= 0 {
            // Frequency is lower 3 bits of nr24 and 8 bits of nr23 combined ( max value is 2047 )
            let freq = u16::from_be_bytes([self.nr34, self.nr33]) & 0x7ff;
            // keep overflow 
            freq_timer_signed += ((2048 - freq) * 2) as i16;
            
            self.wave_duty_pos = (self.wave_duty_pos + 1) % 32;
        }

        // Set back to value (shouldn't be signed anymore since we broke out the loop?)
        self.freq_timer = freq_timer_signed as u16;
    }
    
    pub fn get_amp(&self) -> u8 {
        if self.enable {
            let mut sample = self.wave_ram[self.wave_duty_pos as usize / 2] >> ((self.wave_duty_pos % 2) * 4);
            sample &= 0xf;
            sample >> self.vol_shift
        }
        else {
            0
        }
    }
}

pub struct CH4 {
    // registers
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

impl CH4 {
    
}