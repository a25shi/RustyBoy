use crate::memory::Memory;
use crate::registers::Registers;
use serde_json::Value;
struct CPU {
    registers: Registers,
    memory: Memory,
    opcode_table: serde_json::Value,
    i_flag: u8,
    i_master: u8,
    i_enable: u8,
    halt: bool
}

impl CPU {
    // u16 reg ptr <- u8 reg
    fn ld_to_ptr(&mut self, ptrreg: &str, reg: &str) {
        let ptr = self.registers.get_u16_reg(ptrreg).unwrap();
        let val = self.registers.get_u8_reg(reg).unwrap();
        self.memory.set(ptr, val);
    }
    // u8 reg <- u16 reg ptr
    fn ld_from_ptr(&mut self, reg: &str, ptrreg: &str) {
        let ptr = self.registers.get_u16_reg(ptrreg).unwrap();
        self.registers.set_u8_reg(reg, self.memory.get(ptr)).unwrap();
    }
    // add 16 bit reg to hl
    fn add_16(&mut self, reg: &str) {
        let val = self.registers.get_u16_reg("hl").unwrap();
        let res = self.registers.get_u16_reg(reg).unwrap();
        let (result, overflow) = val.overflowing_add(res);
        self.registers.set_flag("n", false).unwrap();
        self.registers
            .set_flag("h", (val & 0xFFF) + (res & 0xFFF) > 0xFFF)
            .unwrap();
        self.registers.set_flag("z", overflow).unwrap();
        self.registers.set_u16_reg("hl", result).unwrap();
    }
    fn inc(&mut self, reg: &str) {
        // 8 bit inc
        if reg.len() == 1 {
            let val = self.registers.get_u8_reg(reg).unwrap() + 1;
            self.registers.set_u8_reg(reg, val).unwrap();
            self.registers.set_flag("z", val == 0).unwrap();
            self.registers.set_flag("n", false).unwrap();
            self.registers.set_flag("h", (val & 0xF) == 0x0).unwrap();
        } else { // 16 bit
            let val = self.registers.get_u16_reg(reg).unwrap() + 1;
            self.registers.set_u16_reg(reg, val).unwrap();
        }
    }
    fn dec(&mut self, reg: &str) {
        // 8 bit dec
        if reg.len() == 1 {
            let val = self.registers.get_u8_reg(reg).unwrap() - 1;
            self.registers.set_u8_reg(reg, val).unwrap();
            self.registers.set_flag("z", val == 0).unwrap();
            self.registers.set_flag("n", true).unwrap();
            self.registers.set_flag("h", val & 0xF == 0xF).unwrap();
        } else { // 16 bit
            let val = self.registers.get_u16_reg(reg).unwrap() - 1;
            self.registers.set_u16_reg("bc", val).unwrap();
        }
    }

    fn jr(&mut self, value: i8) {
        self.registers.pc = self.registers.pc.wrapping_add_signed(value as i16);
    }

    // return is number of cycles executed
    fn execute_opcode(
        &mut self,
        opcode: u8,
        bytes: u8,
        cycles: Vec<Value>,
        cb: bool,
    ) -> Result<u8, String> {
        let mut value: u16 = 0;
        // immediate 8 bits
        if bytes == 2 {
            value = self.memory.get(self.registers.pc) as u16;
            self.registers.pc += 1;
        }
        // immediate 16 bits
        else if bytes == 3 {
            let a = self.memory.get(self.registers.pc + 1);
            let b = self.memory.get(self.registers.pc);
            value = u16::from_be_bytes([a, b]);
            self.registers.pc += 2;
        }

        // Unprefixed instructions
        if !cb {
            match opcode {
                // NOP
                0x00 => {}
                0x01 => self.registers.set_u16_reg("bc", value)?,
                0x02 => self.ld_to_ptr("bc", "a"),
                0x03 => self.inc("bc"),
                0x04 => self.inc("b"),
                0x05 => self.dec("b"),
                0x06 => self.registers.b = value as u8,
                0x07 => {
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", self.registers.a & 0x80 != 0)?;
                    self.registers.a = self.registers.a.rotate_left(1);
                }
                0x08 => {
                    self.memory.set(value, (self.registers.sp & 0xFF) as u8);
                    self.memory.set(value + 1, (self.registers.sp >> 8) as u8);
                }
                0x09 => self.add_16("bc"),
                0x0a => self.ld_from_ptr("a", "bc"),
                0x0b => self.dec("bc"),
                0x0c => self.inc("c"),
                0x0d => self.dec("c"),
                0x0e => self.registers.c = value as u8,
                0x0f => {
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", self.registers.a & 0x01 != 0)?;
                    self.registers.a = self.registers.a.rotate_right(1);
                }
                // TODO: ADD STOP
                0x10 => {}
                0x11 => self.registers.set_u16_reg("de", value)?,
                0x12 => self.ld_to_ptr("de", "a"),
                0x13 => self.inc("de"),
                0x14 => self.inc("d"),
                0x15 => self.dec("d"),
                0x16 => self.registers.d = value as u8,
                0x17 => {
                    let c = self.registers.get_flag("c")? as u8;
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", self.registers.a & 0x80 != 0)?;
                    self.registers.a = self.registers.a << 1 + c;
                }
                0x18 => self.jr(value as i8),
                0x19 => self.add_16("de"),
                0x1a => self.ld_from_ptr("a", "de"),
                0x1b => self.dec("de"),
                0x1c => self.inc("e"),
                0x1d => self.dec("e"),
                0x1e => self.registers.e = value as u8,
                0x1f => {
                    let c = self.registers.get_flag("c")? as u8;
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", self.registers.a & 0x01 != 0)?;
                    self.registers.a = self.registers.a >> 1 | c << 7;
                }
                0x20 => {
                    if !self.registers.get_flag("z")? {
                        self.jr(value as i8);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0x21 => self.registers.set_u16_reg("hl", value)?,
                0x22 => {
                    self.ld_to_ptr("hl", "a");
                    self.inc("hl");
                }
                0x23 => self.inc("hl"),
                0x24 => self.inc("h"),
                0x25 => self.dec("h"),
                0x26 => self.registers.h = value as u8,
                0x27 => {
                    let mut t = self.registers.a;
                    let mut corr: u8 = 0;
                    corr |= if self.registers.get_flag("h")? { 0x06 } else { 0x00 };
                    corr |= if self.registers.get_flag("c")? { 0x60 } else { 0x00 };

                    if self.registers.get_flag("n")? {
                        t -= corr;
                    } else {
                        corr |= if (t & 0x0f) > 0x09 { 0x06 } else { 0x00 };
                        corr |= if t > 0x99 { 0x60 } else { 0x00 };
                        t += corr;
                    }
                    self.registers.set_flag("z",t == 0)?;
                    self.registers.set_flag("c", corr & 0x60 != 0)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.a = t;
                }
                0x28 => {
                    if self.registers.get_flag("z")? {
                        self.jr(value as i8);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0x29 => self.add_16("hl"),
                0x2a => self.ld_from_ptr("a", "hl"),
                0x2b => self.dec("hl"),
                0x2c => self.inc("l"),
                0x2d => self.dec("l"),
                0x2e => self.registers.l = value as u8,
                0x2f => {
                    self.registers.a = !self.registers.a;
                    self.registers.set_flag("n", true)?;
                    self.registers.set_flag("h", true)?;
                }
                0x30 => {
                    if !self.registers.get_flag("c")? {
                        self.jr(value as i8);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0x31 => self.registers.sp = value,
                0x32 => {
                    self.ld_to_ptr("hl", "a");
                    self.dec("hl");
                }
                0x33 => self.inc("sp"),
                0x34 => {
                    let ptr = self.registers.get_u16_reg("hl")?;
                    let val = self.memory.get(ptr) + 1;
                    self.memory.set(ptr, val);

                    self.registers.set_flag("z", val == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", val & 0x0f == 0)?;
                }
                0x35 => {
                    let ptr = self.registers.get_u16_reg("hl")?;
                    let val = self.memory.get(ptr) - 1;
                    self.memory.set(ptr, val);

                    self.registers.set_flag("z", val == 0)?;
                    self.registers.set_flag("n", true)?;
                    self.registers.set_flag("h", val & 0x0f == 0xf)?;
                }
                0x36 => {
                    let ptr = self.registers.get_u16_reg("hl")?;
                    self.memory.set(ptr, value as u8);
                }
                0x37 => {
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", true)?;
                }
                0x38 => {
                    if self.registers.get_flag("c")? {
                        self.jr(value as i8);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0x39 => self.add_16("sp"),
                0x3a => {
                    self.ld_from_ptr("a", "hl");
                    self.dec("hl");
                }
                0x3b => self.dec("sp"),
                0x3c => self.inc("a"),
                0x3d => self.dec("a"),
                0x3e => self.registers.a = value as u8,
                0x3f => {
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", !self.registers.get_flag("c")?)?;
                }
                0x40 => self.registers.b = self.registers.b,
                0x41 => self.registers.b = self.registers.c,
                0x42 => self.registers.b = self.registers.d,
                0x43 => self.registers.b = self.registers.e,
                0x44 => self.registers.b = self.registers.h,
                0x45 => self.registers.b = self.registers.l,
                0x46 => self.ld_from_ptr("b", "hl"),
                0x47 => self.registers.b = self.registers.a,
                0x48 => self.registers.c = self.registers.b,
                0x49 => self.registers.c = self.registers.c,
                0x4a => self.registers.c = self.registers.d,
                0x4b => self.registers.c = self.registers.e,
                0x4c => self.registers.c = self.registers.h,
                0x4d => self.registers.c = self.registers.l,
                0x4e => self.ld_from_ptr("c", "hl"),
                0x4f => self.registers.c = self.registers.a,
                0x50 => self.registers.d = self.registers.b,
                0x51 => self.registers.d = self.registers.c,
                0x52 => self.registers.d = self.registers.d,
                0x53 => self.registers.d = self.registers.e,
                0x54 => self.registers.d = self.registers.h,
                0x55 => self.registers.d = self.registers.l,
                0x56 => self.ld_from_ptr("d", "hl"),
                0x57 => self.registers.d = self.registers.a,
                0x58 => self.registers.e = self.registers.b,
                0x59 => self.registers.e = self.registers.c,
                0x5a => self.registers.e = self.registers.d,
                0x5b => self.registers.e = self.registers.e,
                0x5c => self.registers.e = self.registers.h,
                0x5d => self.registers.e = self.registers.l,
                0x5e => self.ld_from_ptr("e", "hl"),
                0x5f => self.registers.e = self.registers.a,
                0x60 => self.registers.h = self.registers.b,
                0x61 => self.registers.h = self.registers.c,
                0x62 => self.registers.h = self.registers.d,
                0x63 => self.registers.h = self.registers.e,
                0x64 => self.registers.h = self.registers.h,
                0x65 => self.registers.h = self.registers.l,
                0x66 => self.ld_from_ptr("h", "hl"),
                0x67 => self.registers.h = self.registers.a,
                0x68 => self.registers.l = self.registers.b,
                0x69 => self.registers.l = self.registers.c,
                0x6a => self.registers.l = self.registers.d,
                0x6b => self.registers.l = self.registers.e,
                0x6c => self.registers.l = self.registers.h,
                0x6d => self.registers.l = self.registers.l,
                0x6e => self.ld_from_ptr("l", "hl"),
                0x6f => self.registers.l = self.registers.a,
                _ => unreachable!(),
            }
        }
        // cb prefixed instructions
        else {
        }
        Ok(cycles[0].as_u64().unwrap() as u8)
    }

    fn execute_next_op(&mut self) -> u8 {
        let address: u16 = self.registers.pc;
        let (new_address, opcode, bytes, cycles, cb) = self.decode(address);
        self.registers.pc = new_address;
        self.execute_opcode(opcode, bytes, cycles, cb).unwrap()
    }

    // Decodes the instruction at an address
    // Return format: opcode, bytes, cycles, cb
    fn decode(&self, mut address: u16) -> (u16, u8, u8, Vec<Value>, bool) {
        let mut opcode: u8 = self.memory.get(address);
        let mut cb: bool = false;
        address += 1;

        // If we are getting from CB table, get next opcode
        if opcode == 0xCB {
            opcode = self.memory.get(address);
            address += 1;
            cb = true;
        }

        let hexstring = format!("{:#04X}", opcode);

        // If from cb table
        if cb {
            let bytes = self.opcode_table["cbprefixed"][&hexstring]["bytes"]
                .as_u64()
                .unwrap();
            let cycles = self.opcode_table["cbprefixed"][&hexstring]["cycles"]
                .as_array()
                .unwrap();
            (address, opcode, bytes as u8, cycles.clone(), cb)
        }
        // Otherwise get from other table
        else {
            let bytes = self.opcode_table["unprefixed"][&hexstring]["bytes"]
                .as_u64()
                .unwrap();
            let cycles = self.opcode_table["unprefixed"][&hexstring]["cycles"]
                .as_array()
                .unwrap();
            (address, opcode, bytes as u8, cycles.clone(), cb)
        }
    }
    
    // runs one full cpu tick
    fn update(&self) {
        
    }
}
