pub struct Joypad {
    value: u8,
    // joypad is top 4 bits, directional is bottom 4
    joypad: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            value: 0b11001111,
            joypad: 0xff
        }
    }
    
    // resets bit, returns interrupt true or false
    fn reset_joypad_bit(&mut self, bit: u8) -> bool {
        let prev = (self.joypad >> bit) & 1;
        self.joypad &= !(1 << bit);

        // Going from 1 to 0
        if prev != 0 {
            // Test joypad bit 
            if bit > 3 && !(((self.value >> 5) & 1) != 0) {
                return true;
            }
            // Test directional
            else if bit <= 3 && !(((self.value >> 4) & 1) != 0) {
                return true;
            }
        }

        false
    }
    
    // sets joypad bit
    fn set_joypad_bit(&mut self, bit: u8) -> bool {
        self.joypad |= 1 << bit;
        false
    }
    
    // gets the joypad value
    pub fn get_joypad(&self) -> u8 {
        self.value
    }

    // sets joypad, depending on which button or dpad selected
    pub fn set_joypad(&mut self, mut value: u8) {
        let buttons = ((value >> 5) & 1) != 0;
        let dpad = ((value >> 4) & 1) != 0;
        
        // set top 2 and botton 4 bits to 1, keep buttons and dpad select
        value |= 0b11001111;
        
        // add our joypad values
        if !buttons {
            value &= self.joypad >> 4;
        }
        else if !dpad {
            value &= self.joypad & 0xf;
        }
        
        // store
        self.value = value;
    }

    // Bit corresponds to the appropriate bit key in self.joypad
    pub fn handle_input(&mut self, bit: u8, up: bool) -> bool {
        // on keyup
        if up {
            self.set_joypad_bit(bit)
        } 
        // on keydown
        else {
            self.reset_joypad_bit(bit)
        }
    }
}
    
    
