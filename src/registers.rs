pub struct Registers {
    pub f: u8,
    pub a: u8,
    pub c: u8,
    pub b: u8,
    pub e: u8,
    pub d: u8,
    pub l: u8,
    pub h: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    // Initial register values
    pub fn new() -> Self {
        Self {
            a: 0x01,
            f: 0xb0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,
        }
    }
    // Specifically for the u16 register access (AF, BC, DE, HL) of the u8 registers
    pub fn get_u16_reg(&self, reg: &str) -> Result<u16, String> {
        match reg {
            "af" => Ok(u16::from_be_bytes([self.a, self.f])),
            "bc" => Ok(u16::from_be_bytes([self.b, self.c])),
            "de" => Ok(u16::from_be_bytes([self.d, self.e])),
            "hl" => Ok(u16::from_be_bytes([self.h, self.l])),
            "sp" => Ok(self.sp),
            "pc" => Ok(self.pc),
            _ => Err(String::from("Unexpected Register")),
        }
    }
    // u8 register access
    pub fn get_u8_reg(&self, reg: &str) -> Result<u8, String> {
        match reg {
            "a" => Ok(self.a),
            "f" => Ok(self.f),
            "b" => Ok(self.b),
            "c" => Ok(self.c),
            "d" => Ok(self.d),
            "e" => Ok(self.e),
            "h" => Ok(self.h),
            "l" => Ok(self.l),
            _ => Err(String::from("Unexpected Register")),
        }
    }
    // Specifically for the u16 register modification (AF, BC, DE, HL) of the u8 registers
    pub fn set_u16_reg(&mut self, reg: &str, value: u16) -> Result<(), String> {
        match reg {
            "af" => {
                let [a, f] = value.to_be_bytes();
                self.a = a;
                // flags only need top 4 bits
                self.f = f & 0xf0;
                Ok(())
            }
            "bc" => {
                let [b, c] = value.to_be_bytes();
                self.b = b;
                self.c = c;
                Ok(())
            }
            "de" => {
                let [d, e] = value.to_be_bytes();
                self.d = d;
                self.e = e;
                Ok(())
            }
            "hl" => {
                let [h, l] = value.to_be_bytes();
                self.h = h;
                self.l = l;
                Ok(())
            }
            "sp" => {
                self.sp = value;
                Ok(())
            }
            "pc" => {
                self.pc = value;
                Ok(())
            }
            _ => Err(String::from("Unexpected Register")),
        }
    }
    // u8 register modification
    pub fn set_u8_reg(&mut self, reg: &str, value: u8) -> Result<(), String> {
        match reg {
            "a" => {
                self.a = value;
                Ok(())
            }
            "f" => {
                self.f = value;
                Ok(())
            }
            "b" => {
                self.b = value;
                Ok(())
            }
            "c" => {
                self.c = value;
                Ok(())
            }
            "d" => {
                self.d = value;
                Ok(())
            }
            "e" => {
                self.e = value;
                Ok(())
            }
            "h" => {
                self.h = value;
                Ok(())
            }
            "l" => {
                self.l = value;
                Ok(())
            }
            _ => Err(String::from("Unexpected Register")),
        }
    }
    // Get flag
    pub fn get_flag(&self, flag: &str) -> Result<bool, String> {
        match flag {
            "z" => Ok((self.f >> 7) & 1 != 0),
            "n" => Ok((self.f >> 6) & 1 != 0),
            "h" => Ok((self.f >> 5) & 1 != 0),
            "c" => Ok((self.f >> 4) & 1 != 0),
            _ => Err(String::from("Unexpected Flag")),
        }
    }
    // Set flag
    pub fn set_flag(&mut self, flag: &str, value: bool) -> Result<(), String> {
        match flag {
            "z" => {
                self.f = (self.f & !(1 << 7)) | (value as u8) << 7;
                Ok(())
            },
            "n" => {
                self.f = (self.f & !(1 << 6)) | (value as u8) << 6;
                Ok(())
            },
            "h" => {
                self.f = (self.f & !(1 << 5)) | (value as u8) << 5;
                Ok(())
            },
            "c" => {
                self.f = (self.f & !(1 << 4)) | (value as u8) << 4;
                Ok(())
            },
            _ => Err(String::from("Unexpected Flag")),
        }
    }
}
