use rodio::{Decoder, OutputStream, source::Source};
use rodio::buffer::SamplesBuffer;

const BUFFER_SIZE: usize = 1024;
 const CPU_CLOCK: usize = 4194304;
 const SAMPLE_RATE: usize = 48000;
const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

const CH3_SHIFT_TABLE: [u8; 4] = [4, 0, 1, 2];
const CH4_DIV_TABLE: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

fn get_wave_duty(duty: u8, duty_pos: u8) -> u8 {
    WAVE_DUTY[duty as usize][duty_pos as usize]
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

pub struct Sound {
    // registers
    apu_enabled: bool,
    nr50: u8,
    nr51: u8,

    // channels
    ch1: CH1,
    ch2: CH2,
    ch3: CH3,
    ch4: CH4,

    // counters
    frame_seq: u8,
    frame_counter: u32,

    // sound stream
    stream_handle: OutputStream,
    sink: rodio::Sink,

    // sound buffer
    buffer: Vec<f32>
}

impl Sound {
    pub fn new() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.set_volume(0.10);
        Self {
            apu_enabled: false,
            nr50: 0,
            nr51: 0,
            ch1: CH1::new(),
            ch2: CH2::new(),
            ch3: CH3::new(),
            ch4: CH4::new(),
            frame_seq: 0,
            frame_counter: 0,
            stream_handle,
            sink,
            buffer: vec![0.0; BUFFER_SIZE]
        }
    }

    pub fn play_sound(&mut self, buffer: Vec<f32>) {
        // wait for sink to clear
        while self.sink.len() > 2 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        self.sink.append(SamplesBuffer::new(2, 48000, buffer));
    }

    pub fn tick(&mut self, cycles: u8) {
        if cycles == 0 {
            return;
        }
        let mut c_cycles = 0;

        // step through cycles 2 at a time
        while c_cycles < cycles {
            // tick div
            self.frame_counter += 2;
            let mut frame_seq_tick = false;
            // tick frame sequencer on 512 hz (8192 cycles)
            if self.frame_counter >= 8192 {
                self.frame_counter -= 8192;
                self.frame_seq = (self.frame_seq + 1) % 8;
                frame_seq_tick = true;
            }

            // if power on tick channels
            if self.apu_enabled {
                // tick channels
                self.ch1.tick(2);
                self.ch2.tick(2);
                self.ch3.tick(2);
                self.ch4.tick(2);

                // if frame seq tick, tick other units
                if frame_seq_tick {
                    if self.frame_seq % 2 == 0 {
                        self.ch1.tick_len();
                        self.ch2.tick_len();
                        self.ch3.tick_len();
                        self.ch4.tick_len();
                    }
                    if self.frame_seq % 4 == 2 {
                        self.ch1.tick_sweep();
                    }
                    if self.frame_seq % 8 == 7 {
                        self.ch1.tick_env();
                        self.ch2.tick_env();
                        self.ch4.tick_env();
                    }
                }
            }
            
            if (self.frame_counter % ((CPU_CLOCK / SAMPLE_RATE) as u32)) <= 1 {
                let ch1_amp = self.ch1.get_amp();
                let ch2_amp = self.ch2.get_amp();
                let ch3_amp = self.ch3.get_amp();
                let ch4_amp = self.ch4.get_amp();

                let left_vol = ((self.nr50 >> 4) & 0b111) as f32 / 7.0;
                let right_vol = (self.nr50 & 0b111) as f32 / 7.0;

                let ch1_left = (self.nr51 & 0x10) != 0;
                let ch2_left = (self.nr51 & 0x20) != 0;
                let ch3_left = (self.nr51 & 0x40) != 0;
                let ch4_left = (self.nr51 & 0x80) != 0;

                let ch1_right = (self.nr51 & 0x01) != 0;
                let ch2_right = (self.nr51 & 0x02) != 0;
                let ch3_right = (self.nr51 & 0x04) != 0;
                let ch4_right = (self.nr51 & 0x08) != 0;

                let mut left: f32 = 0.0;
                let mut right: f32 = 0.0;

                if self.ch1.enable {
                    if ch1_left {
                        left += ch1_amp;
                    }
                    if ch1_right {
                        right += ch1_amp;
                    }
                }
                if self.ch2.enable {
                    if ch2_left {
                        left += ch2_amp;
                    }
                    if ch2_right {
                        right += ch2_amp;
                    }
                }
                if self.ch3.enable {
                    if ch3_left {
                        left += ch3_amp;
                    }
                    if ch3_right {
                        right += ch3_amp;
                    }
                }
                if self.ch4.enable {
                    if ch4_left {
                        left += ch4_amp;
                    }
                    if ch4_right {
                        right += ch4_amp;
                    }
                }
                
                left /= 4.0;
                right /= 4.0;
                self.buffer.push(left * left_vol);
                self.buffer.push(right * right_vol);
            }
            
            // If sound buffer is full, send it
            if self.buffer.len() >= BUFFER_SIZE {
                self.play_sound(self.buffer.clone());
                self.buffer.clear();
            }
            
            c_cycles += 2;
        }
    }

    // $ff10 - $ff26 audio registers
    // $ff30 - $ff3f wave ram
    pub fn get(&self, address: u16) -> u8 {
        // println!("reading sound at address {:04x}", address);
        match address {
            0xff10 => self.ch1.nr10 | 0x80,
            0xff11 => self.ch1.nr11 | 0x3F,
            0xff12 => self.ch1.nr12,
            0xff13 => self.ch1.nr13 | 0xFF,
            0xff14 => self.ch1.nr14 | 0xBF,
            0xff16 => self.ch2.nr21 | 0x3F,
            0xff17 => self.ch2.nr22,
            0xff18 => self.ch2.nr23 | 0xFF,
            0xff19 => self.ch2.nr24 | 0xBF,
            0xff1A => self.ch3.nr30 | 0x7F,
            0xff1B => self.ch3.nr31 | 0xFF,
            0xff1C => self.ch3.nr32 | 0x9F,
            0xff1D => self.ch3.nr33 | 0xFF,
            0xff1E => self.ch3.nr34 | 0xBF,
            0xff20 => self.ch4.nr41 | 0xFF,
            0xff21 => self.ch4.nr42,
            0xff22 => self.ch4.nr43,
            0xff23 => self.ch4.nr44 | 0xBF,
            0xff24 => self.nr50,
            0xff25 => self.nr51,
            0xff26 => {
                ((self.apu_enabled as u8) << 7) | 0x70
                    | (self.ch1.enable as u8)
                    | ((self.ch2.enable as u8) << 1)
                    | ((self.ch3.enable as u8) << 2)
                    | ((self.ch4.enable as u8) << 3)
            }
            0xff30..=0xff3f => {
                if self.ch3.enable {
                    self.ch3.wave_ram[self.ch3.wave_duty_pos as usize / 2]
                } else {
                    self.ch3.wave_ram[(address - 0xff30) as usize]
                }
            },
            _ => {
                0xff
            }
        }
    }

    pub fn set(&mut self, address: u16, value: u8) {
        // println!("writing sound at address {:04x}", address);
        // If apu is not enabled and not in the writable addresses when off, return
        if !(self.apu_enabled
            || address == 0xFF26
            || (0xFF30..=0xFF3F).contains(&address)
            || [0xFF11, 0xFF16, 0xFF1B, 0xFF20].contains(&address))
        {
            return;
        }

        let value = if !self.apu_enabled && [0xFF11, 0xFF16, 0xFF20].contains(&address) {
            value & 0b0011_1111
        } else {
            value
        };

        match address {
            // Channel 1
            0xff10 => self.ch1.nr10 = value,
            0xff11 => {
                self.ch1.nr11 = value;
                self.ch1.len_timer = 64 - (value & 0x3f);
            }
            0xff12 => {
                self.ch1.nr12 = value;
                // If top 5 bits are reset, channel will be disabled
                if self.ch1.nr12 & 0xf8 == 0 {
                    self.ch1.enable = false;
                }
            }
            0xff13 => self.ch1.nr13 = value,
            0xff14 => {
                let trigger = value & 0x80 != 0;
                self.ch1.nr14 = value;
                
                // on trigger
                if trigger {
                    // enable channel if dac is on
                    if self.ch1.nr12 & 0xF8 != 0 {
                        self.ch1.enable = true;
                    }

                    // if length timer is expired, reset
                    if self.ch1.len_timer == 0 {
                        self.ch1.len_timer = 64;
                    }

                    // set freq timer
                    let freq = u16::from_be_bytes([self.ch1.nr14, self.ch1.nr13]) & 0x7ff;
                    self.ch1.freq_timer = (2048 - freq) * 4;

                    // set wave duty position
                    self.ch1.wave_duty_pos = 0;

                    // reset env
                    self.ch1.env_timer = self.ch1.nr12 & 0b111;
                    self.ch1.cur_vol = self.ch1.nr12 >> 4;

                    // reset sweep
                    let sweep_period = (self.ch1.nr10 >> 4) & 0b111;
                    let sweep_shift = self.ch1.nr10 & 0b111;
                    self.ch1.shadow_freq = freq;
                    self.ch1.sweep_timer = if sweep_period > 0 {
                        sweep_period
                    } else {
                        8
                    };
                    self.ch1.sweep_enable = sweep_period > 0 || sweep_shift > 0;
                    // overflow check
                    if sweep_shift > 0 {
                        self.ch1.calc_freq(sweep_shift);
                    }
                }
            }
            // Channel 2
            0xff16 => {
                self.ch2.nr21 = value;
                self.ch2.len_timer = 64 - (value & 0x3f);
            }
            0xff17 => {
                self.ch2.nr22 = value;
                // If top 5 bits are reset, channel will be disabled
                if self.ch2.nr22 & 0xf8 == 0 {
                    self.ch2.enable = false;
                }
            }
            0xff18 => self.ch2.nr23 = value,
            0xff19 => {
                let trigger = value & 0x80 != 0;
                self.ch2.nr24 = value;
                
                // on trigger
                if trigger {
                    // enable channel if dac is on
                    if self.ch2.nr22 & 0xF8 != 0 {
                        self.ch2.enable = true;
                    }

                    // if length timer is expired, reset
                    if self.ch2.len_timer == 0 {
                        self.ch2.len_timer = 64;
                    }

                    // set freq timer
                    let freq = u16::from_be_bytes([self.ch2.nr24, self.ch2.nr23]) & 0x7ff;
                    self.ch2.freq_timer = (2048 - freq) * 4;

                    // set wave duty position
                    self.ch2.wave_duty_pos = 0;

                    // reset env
                    self.ch2.env_timer = self.ch2.nr22 & 0b111;
                    self.ch2.cur_vol = self.ch2.nr22 >> 4;
                }
            }
            // Channel 3
            0xff1a => {
                self.ch3.nr30 = value;
                if self.ch3.nr30 & 0x80 == 0 {
                    self.ch3.enable = false;
                }
            }
            0xff1b => {
                self.ch3.nr31 = value;
                self.ch3.len_timer = 256 - value as u16;
            }
            0xff1c => self.ch3.nr32 = value,
            0xff1d => self.ch3.nr33 = value,
            0xff1e => {
                let trigger = value & 0x80 != 0;
                self.ch3.nr34 = value;
                
                // on trigger
                if trigger {
                    // enable channel
                    if self.ch3.nr30 & 0x80 != 0 {
                        self.ch3.enable = true;
                    }

                    // if length timer is expired, reset
                    if self.ch3.len_timer == 0 {
                        self.ch3.len_timer = 256;
                    }

                    // set freq timer
                    let freq = u16::from_be_bytes([self.ch3.nr34, self.ch3.nr33]) & 0x7ff;
                    self.ch3.freq_timer = (2048 - freq) * 2;
                    
                    // set wave duty position
                    self.ch3.wave_duty_pos = 0;
                }
            }
            // Channel 4
            0xff20 => {
                self.ch4.nr41 = value;
                self.ch4.len_timer = 64 - (value & 0x3f);
            }
            0xff21 => {
                self.ch4.nr42 = value;
                // If top 5 bits are reset, channel will be disabled
                if self.ch4.nr42 & 0xf8 == 0 {
                    self.ch4.enable = false;
                }
            }
            0xff22 => self.ch4.nr43 = value,
            0xff23 => {
                let trigger = value & 0x80 != 0;
                self.ch4.nr44 = value;
                
                // on trigger
                if trigger {
                    // enable channel if dac is on
                    if self.ch4.nr42 & 0xF8 != 0 {
                        self.ch4.enable = true;
                    }
                    // set freq timer
                    let divisor = CH4_DIV_TABLE[(self.ch4.nr43 & 0b111) as usize];
                    let shift = self.ch4.nr43 >> 4;
                    self.ch4.freq_timer = (divisor as u16) << shift;
                    
                    // if length timer is expired, reset
                    if self.ch4.len_timer == 0 {
                        // if length is now enabled, sub extra frame seq clock
                        self.ch4.len_timer = 64;
                    }

                    // reset env
                    self.ch4.env_timer = self.ch4.nr42 & 0b111;
                    self.ch4.cur_vol = self.ch4.nr42 >> 4;
                    
                    // reset lfsr
                    self.ch4.lfsr = 0x7fff;
                }
            }
            0xff24 => self.nr50 = value,
            0xff25 => self.nr51 = value,
            0xff26 => {
                let enabled = value >> 7 != 0;
                
                // turning sound on to off
                if self.apu_enabled && !enabled {
                    // reset sound registers
                    for addr in 0xff10..=0xff25 {
                        self.set(addr, 0);
                    }
                    self.apu_enabled = false;
                } 
                    
                // turning sound off to on
                else if !self.apu_enabled && enabled {
                    self.apu_enabled = true;
                }
            }
            // Wave RAM
            0xff30..=0xff3f => self.ch3.wave_ram[(address - 0xff30) as usize] = value,
            _ => {}
        }
    }
}

// Sound channel 1, length, env, sweep
struct CH1 {
    // registers
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,

    // none accessible registers
    enable: bool,
    len_timer: u8,
    freq_timer: u16,
    env_timer: u8,
    wave_duty_pos: u8,
    cur_vol: u8,

    // sweep
    sweep_enable: bool,
    sweep_timer: u8,
    shadow_freq: u16,
}

impl CH1 {
    pub fn new() -> Self {
        Self {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            enable: false,
            len_timer: 0,
            freq_timer: 0,
            env_timer: 0,
            wave_duty_pos: 0,
            cur_vol: 0,
            sweep_enable: false,
            sweep_timer: 0,
            shadow_freq: 0,
        }
    }
    pub fn tick(&mut self, cycles: u8) {
        // freq timer max value should be 4096? About 14 bits so signed should be safe
        let mut freq_timer_signed = self.freq_timer as i16;
        freq_timer_signed -= cycles as i16;

        while freq_timer_signed <= 0 {
            // Frequency is lower 3 bits of nr24 and 8 bits of nr23 combined ( max value is 2047 )
            let freq = u16::from_be_bytes([self.nr14, self.nr13]) & 0x7ff;
            // keep overflow
            freq_timer_signed += ((2048 - freq) * 4) as i16;

            self.wave_duty_pos = (self.wave_duty_pos + 1) % 8;
        }

        // Set back to value (shouldn't be signed anymore since we broke out the loop?)
        self.freq_timer = freq_timer_signed as u16;
    }

    pub fn tick_len(&mut self) {
        // check length enabled
        let enabled = self.nr14 & 0b1000000;
        if enabled != 0 && self.len_timer > 0 {
            self.len_timer -= 1;
            if self.len_timer == 0 {
                self.enable = false;
            }
        }
    }

    pub fn tick_env(&mut self) {
        run_env(&mut self.env_timer, &mut self.cur_vol, self.nr12);
    }

    pub fn tick_sweep(&mut self) {
        let sweep_period = (self.nr10 >> 4) & 0b111;
        let sweep_shift = self.nr10 & 0b111;

        // tick timer
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }

        // timer overflow
        if self.sweep_timer == 0 {
            if sweep_period > 0 {
                self.sweep_timer = sweep_period;
            } else {
                self.sweep_timer = 8;
            }

            if self.sweep_enable && sweep_period > 0 {
                let new_freq = self.calc_freq(sweep_shift);

                if new_freq < 2048 && sweep_shift > 0 {
                    let freq = new_freq & 0x7ff;
                    let [upper, lower] = freq.to_be_bytes();
                    // set frequency
                    // Set lower 3 bits of nr14 and 8 bits of nr13 (forms freq)
                    self.nr14 = (self.nr14 & 0b11111000) | (upper & 0b111);
                    self.nr13 = lower;

                    self.shadow_freq = new_freq;
                    // overflow check
                    self.calc_freq(sweep_shift);
                }
            }
        }
    }

    pub fn calc_freq(&mut self, sweep_shift: u8) -> u16 {
        let sweep_dir = self.nr10 & 0b1000 != 0;
        let mut new_freq = self.shadow_freq >> sweep_shift;

        // decrementing
        if sweep_dir {
            new_freq = self.shadow_freq - new_freq;
        }
        // incrementing
        else {
            new_freq = self.shadow_freq + new_freq;
        }

        // overflow check
        if new_freq > 2047 {
            self.enable = false;
        }
        new_freq
    }

    pub fn get_amp(&self) -> f32 {
        if self.enable {
            let duty = (self.nr11 >> 6) & 0b11;
            let input = (get_wave_duty(duty, self.wave_duty_pos) * self.cur_vol) as f32;
            (input / 7.5) - 1.0
        } else {
            0.0
        }
    }
}

// Sound channel 2, length, env
struct CH2 {
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
    pub fn new() -> Self {
        Self {
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            enable: false,
            len_timer: 0,
            freq_timer: 0,
            env_timer: 0,
            wave_duty_pos: 0,
            cur_vol: 0,
        }
    }
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
    pub fn get_amp(&self) -> f32 {
        if self.enable {
            let duty = (self.nr21 >> 6) & 0b11;
            let input = (get_wave_duty(duty, self.wave_duty_pos) * self.cur_vol) as f32;
            (input / 7.5) - 1.0
        } else {
            0.0
        }
    }
}

// Sound channel 3, length
struct CH3 {
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
    len_timer: u16,
    freq_timer: u16,

    // selects wave ram index
    wave_duty_pos: u8,
}

impl CH3 {
    pub fn new() -> Self {
        Self {
            wave_ram: [0xff; 16],
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            enable: false,
            len_timer: 0,
            freq_timer: 0,
            wave_duty_pos: 0,
        }
    }
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

    pub fn get_amp(&self) -> f32 {
        if self.enable {
            let vol_shift = CH3_SHIFT_TABLE[((self.nr32 >> 5) & 0b11) as usize];
            let mut sample =
                self.wave_ram[self.wave_duty_pos as usize / 2] >> ((self.wave_duty_pos % 2) * 4);
            sample &= 0xf;
            ((sample >> vol_shift) as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }
}

// Sound channel 4, length, env
struct CH4 {
    // registers
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    // none accessible registers
    enable: bool,
    len_timer: u8,
    freq_timer: u16,
    cur_vol: u8,
    env_timer: u8,
    lfsr: u16,
}

impl CH4 {
    pub fn new() -> Self {
        Self {
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            enable: false,
            len_timer: 0,
            freq_timer: 0,
            cur_vol: 0,
            env_timer: 0,
            lfsr: 0,
        }
    }
    pub fn tick_len(&mut self) {
        // check length enabled
        let enabled = self.nr44 & 0b1000000;
        if enabled != 0 && self.len_timer > 0 {
            self.len_timer -= 1;
            if self.len_timer == 0 {
                self.enable = false;
            }
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        let mut freq_timer_signed = self.freq_timer as i32;
        let divisor = CH4_DIV_TABLE[(self.nr43 & 0b111) as usize];
        let shift = self.nr43 >> 4;
        // tick timer
        freq_timer_signed -= cycles as i32;

        while freq_timer_signed <= 0 {
            freq_timer_signed += (divisor as i32) << shift as i32;
            let xor_res = (self.lfsr & 0b1) ^ ((self.lfsr & 0b10) >> 1);
            self.lfsr = (self.lfsr >> 1) | (xor_res << 14);

            if ((self.nr43 >> 3) & 0b1) != 0 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_res << 6;
            }
        }

        // Set back to value (shouldn't be signed anymore since we broke out the loop?)
        self.freq_timer = freq_timer_signed as u16;
    }

    pub fn tick_env(&mut self) {
        run_env(&mut self.env_timer, &mut self.cur_vol, self.nr42);
    }

    pub fn get_amp(&self) -> f32 {
        if self.enable {
            let input = (!self.lfsr & 0b1) as f32 * self.cur_vol as f32;
            (input / 7.5) - 1.0
        } else {
            0.0
        }
    }
}
