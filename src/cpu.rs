mod registers;
use crate::memory::Memory;
use crate::motherboard::Motherboard;
use registers::Registers;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::rc::Rc;


pub struct CPU {
    pub registers: Registers,
    memory: Memory,
    motherboard: Rc<Motherboard>,
    opcode_table: Value,
    i_queue: bool,
    halt: bool,
    blargg: String,
}

impl CPU {
    pub fn new(rom_file: Vec<u8>) -> Self {
        let opcodedata = fs::read_to_string("./src/opcodes/Opcodes.json").unwrap();
        // let rom: Vec<u8> = fs::read(rom_file).unwrap();
        let mobo = Rc::new(Motherboard::new());
        // Initialize self
        let this = Self {
            registers: Registers::new(),
            memory: Memory::new(rom_file, &mobo),
            opcode_table: serde_json::from_str(&opcodedata).unwrap(),
            motherboard: mobo,
            i_queue: false,
            halt: false,
            blargg: String::new(),
        };
        this
    }

    // u8 reg or hl address read
    fn read_u8_hl(&mut self, reg: &str) -> u8 {
        if reg == "hl" {
            self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
            self.memory.get(self.registers.get_u16_reg("hl").unwrap())
        } else {
            self.registers.get_u8_reg(reg).unwrap()
        }
    }
    // u8 reg or hl address write
    fn write_u8_hl(&mut self, reg: &str, value: u8) {
        if reg == "hl" {
            self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
            self.memory
                .set(self.registers.get_u16_reg("hl").unwrap(), value);
        } else {
            self.registers.set_u8_reg(reg, value).unwrap();
        }
    }
    // bit
    fn bit(&mut self, reg: &str, shift: u8) {
        let val = self.read_u8_hl(reg);
        let ret = val & (1 << shift);

        self.registers.set_flag("z", ret == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", true).unwrap();
    }
    // res
    fn res(&mut self, reg: &str, shift: u8) {
        let val = self.read_u8_hl(reg);
        let ret = val & !(1 << shift);
        self.write_u8_hl(reg, ret);
    }
    // set
    fn set(&mut self, reg: &str, shift: u8) {
        let val = self.read_u8_hl(reg);
        let ret = val | (1 << shift);
        self.write_u8_hl(reg, ret);
    }
    // srl
    fn srl(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        let c = val & 0x01 != 0;
        val >>= 1;
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", c).unwrap();
    }
    // swap
    fn swap(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        val = (val & 0x0f) << 4 | ((val & 0xf0) >> 4);
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", false).unwrap();
    }
    // sra
    fn sra(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        let c = val & 0x01 != 0;
        val = (val & 0x80) | val >> 1;
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", c).unwrap();
    }
    // sla
    fn sla(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        let c = val & 0x80 != 0;
        val <<= 1;
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", c).unwrap();
    }
    // rr
    fn rr(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        let c = self.registers.get_flag("c").unwrap() as u8;
        self.registers.set_flag("c", val & 0x01 != 0).unwrap();

        val = val >> 1 | c << 7;
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
    }
    // rl
    fn rl(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        let c = self.registers.get_flag("c").unwrap() as u8;
        self.registers.set_flag("c", val & 0x80 != 0).unwrap();

        val = val << 1 | c;
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
    }
    // rrc
    fn rrc(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        val = val.rotate_right(1);
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", val & 0x80 != 0).unwrap();
    }
    // rlc
    fn rlc(&mut self, reg: &str) {
        let mut val = self.read_u8_hl(reg);
        val = val.rotate_left(1);
        self.write_u8_hl(reg, val);

        self.registers.set_flag("z", val == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", val & 0x1 != 0).unwrap();
    }
    // u16 reg ptr <- u8 reg
    fn ld_to_ptr(&mut self, ptrreg: &str, reg: &str) {
        let ptr = self.registers.get_u16_reg(ptrreg).unwrap();
        let val = self.registers.get_u8_reg(reg).unwrap();
        self.memory.set(ptr, val);
    }
    // u8 reg <- u16 reg ptr
    fn ld_from_ptr(&mut self, reg: &str, ptrreg: &str) {
        let ptr = self.registers.get_u16_reg(ptrreg).unwrap();
        let mut val = false;
        self.registers
            .set_u8_reg(reg, self.memory.get(ptr))
            .unwrap();
    }
    fn pop_stack(&mut self) -> u16 {
        let a = self.memory.get(self.registers.sp);
        let b = self.memory.get(self.registers.sp + 1);
        self.registers.sp += 2;
        u16::from_be_bytes([b, a])
    }
    fn push_stack(&mut self, value: u16) {
        let [a, b] = value.to_le_bytes();
        self.memory.set(self.registers.sp - 1, b);
        self.memory.set(self.registers.sp - 2, a);
        self.registers.sp -= 2;
    }
    fn jp_to(&mut self, address: u16) {
        self.registers.pc = address;
    }
    fn ret(&mut self) {
        let address = self.pop_stack();
        self.jp_to(address);
    }
    fn call(&mut self, value: u16) {
        self.push_stack(self.registers.pc);
        self.jp_to(value);
    }
    // pops stack to reg u16
    fn pop_reg(&mut self, reg: &str) {
        let value = self.pop_stack();
        self.registers.set_u16_reg(reg, value).unwrap();
    }
    // pushes reg to stack u16
    fn push_reg(&mut self, reg: &str) {
        let value = self.registers.get_u16_reg(reg).unwrap();
        self.push_stack(value);
    }
    // a == reg
    fn cp(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        self.registers
            .set_flag("z", self.registers.a == val)
            .unwrap();
        self.registers.set_flag("n", true).unwrap();
        self.registers
            .set_flag("h", (self.registers.a & 0xf) < val & 0xf)
            .unwrap();
        self.registers
            .set_flag("c", self.registers.a < val)
            .unwrap();
    }
    // a <- a ^ reg
    fn xor(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        self.registers.a ^= val;
        self.registers.set_flag("z", self.registers.a == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", false).unwrap();
    }
    // a <- a | reg
    fn or(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        self.registers.a |= val;
        self.registers.set_flag("z", self.registers.a == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", false).unwrap();
        self.registers.set_flag("c", false).unwrap();
    }
    // a <- a & reg
    fn and(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        self.registers.a &= val;
        self.registers.set_flag("z", self.registers.a == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers.set_flag("h", true).unwrap();
        self.registers.set_flag("c", false).unwrap();
    }
    // a <- a + reg + c
    fn adc_a(&mut self, reg: &str) {
        let val = self.registers.a as u16;
        let carry = self.registers.get_flag("c").unwrap() as u16;
        let res: u16;
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            res = self.memory.get(ptr) as u16;
        } else {
            res = self.registers.get_u8_reg(reg).unwrap() as u16;
        }
        let result = val + res + carry;
        self.registers.set_flag("z", result & 0xff == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers
            .set_flag("h", (val & 0xf) + (res & 0xf) + carry > 0xf)
            .unwrap();
        self.registers.set_flag("c", result > 0xff).unwrap();

        self.registers.a = (result & 0xff) as u8;
    }
    // a <- a + reg
    fn add_a(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        let (result, overflow) = self.registers.a.overflowing_add(val);

        self.registers.set_flag("z", result == 0).unwrap();
        self.registers.set_flag("n", false).unwrap();
        self.registers
            .set_flag("h", (self.registers.a & 0xF) + (val & 0xF) > 0xF)
            .unwrap();
        self.registers.set_flag("c", overflow).unwrap();

        self.registers.a = result;
    }
    // a <- a - reg
    fn sub_a(&mut self, reg: &str) {
        let val: u8;
        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            val = self.memory.get(ptr);
        } else {
            // otherwise get from reg
            val = self.registers.get_u8_reg(reg).unwrap();
        }

        let (result, overflow) = self.registers.a.overflowing_sub(val);

        self.registers.set_flag("z", result == 0).unwrap();
        self.registers.set_flag("n", true).unwrap();
        self.registers
            .set_flag("h", (self.registers.a & 0xF) < (val & 0xF))
            .unwrap();
        self.registers.set_flag("c", overflow).unwrap();

        self.registers.a = result
    }
    // a <- a - reg - carry
    fn sbc_a(&mut self, reg: &str) {
        let val = self.registers.a as i16;
        let carry = self.registers.get_flag("c").unwrap() as i16;
        let res: i16;

        // if its hl, it's a pointer instead
        if reg == "hl" {
            let ptr = self.registers.get_u16_reg("hl").unwrap();
            res = self.memory.get(ptr) as i16;
        } else {
            // otherwise get from reg
            res = self.registers.get_u8_reg(reg).unwrap() as i16;
        }

        let result = val - res - carry;

        self.registers.set_flag("z", result & 0xff == 0).unwrap();
        self.registers.set_flag("n", true).unwrap();
        self.registers
            .set_flag("h", (val & 0xf) < (res & 0xf) + carry)
            .unwrap();
        self.registers.set_flag("c", result < 0x0).unwrap();

        self.registers.a = (result & 0xff) as u8
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
        self.registers.set_flag("c", overflow).unwrap();
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
        } else {
            // 16 bit
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
        } else {
            // 16 bit
            let val = self.registers.get_u16_reg(reg).unwrap() - 1;
            self.registers.set_u16_reg(reg, val).unwrap();
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
        if bytes == 2 && !cb {
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
                    self.registers.a = self.registers.a << 1 | c;
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
                    corr |= if self.registers.get_flag("h")? {
                        0x06
                    } else {
                        0x00
                    };
                    corr |= if self.registers.get_flag("c")? {
                        0x60
                    } else {
                        0x00
                    };

                    if self.registers.get_flag("n")? {
                        t -= corr;
                    } else {
                        corr |= if (t & 0x0f) > 0x09 { 0x06 } else { 0x00 };
                        corr |= if t > 0x99 { 0x60 } else { 0x00 };
                        t += corr;
                    }
                    self.registers.set_flag("z", t == 0)?;
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
                0x2a => { 
                    
                    self.ld_from_ptr("a", "hl");
                    self.inc("hl");
                }
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
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
                    self.memory.set(ptr, val);

                    self.registers.set_flag("z", val == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", val & 0x0f == 0)?;
                }
                0x35 => {
                    let ptr = self.registers.get_u16_reg("hl")?;
                    let val = self.memory.get(ptr) - 1;
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
                    self.memory.set(ptr, val);

                    self.registers.set_flag("z", val == 0)?;
                    self.registers.set_flag("n", true)?;
                    self.registers.set_flag("h", val & 0x0f == 0xf)?;
                }
                0x36 => {
                    let ptr = self.registers.get_u16_reg("hl")?;
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
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
                    self.registers
                        .set_flag("c", !self.registers.get_flag("c")?)?;
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
                0x70 => self.ld_to_ptr("hl", "b"),
                0x71 => self.ld_to_ptr("hl", "c"),
                0x72 => self.ld_to_ptr("hl", "d"),
                0x73 => self.ld_to_ptr("hl", "e"),
                0x74 => self.ld_to_ptr("hl", "h"),
                0x75 => self.ld_to_ptr("hl", "l"),
                0x76 => self.halt = true,
                0x77 => self.ld_to_ptr("hl", "a"),
                0x78 => self.registers.a = self.registers.b,
                0x79 => self.registers.a = self.registers.c,
                0x7a => self.registers.a = self.registers.d,
                0x7b => self.registers.a = self.registers.e,
                0x7c => self.registers.a = self.registers.h,
                0x7d => self.registers.a = self.registers.l,
                0x7e => self.ld_from_ptr("a", "hl"),
                0x7f => self.registers.a = self.registers.a,
                0x80 => self.add_a("b"),
                0x81 => self.add_a("c"),
                0x82 => self.add_a("d"),
                0x83 => self.add_a("e"),
                0x84 => self.add_a("h"),
                0x85 => self.add_a("l"),
                0x86 => self.add_a("hl"),
                0x87 => self.add_a("a"),
                0x88 => self.adc_a("b"),
                0x89 => self.adc_a("c"),
                0x8a => self.adc_a("d"),
                0x8b => self.adc_a("e"),
                0x8c => self.adc_a("h"),
                0x8d => self.adc_a("l"),
                0x8e => self.adc_a("hl"),
                0x8f => self.adc_a("a"),
                0x90 => self.sub_a("b"),
                0x91 => self.sub_a("c"),
                0x92 => self.sub_a("d"),
                0x93 => self.sub_a("e"),
                0x94 => self.sub_a("h"),
                0x95 => self.sub_a("l"),
                0x96 => self.sub_a("hl"),
                0x97 => self.sub_a("a"),
                0x98 => self.sbc_a("b"),
                0x99 => self.sbc_a("c"),
                0x9a => self.sbc_a("d"),
                0x9b => self.sbc_a("e"),
                0x9c => self.sbc_a("h"),
                0x9d => self.sbc_a("l"),
                0x9e => self.sbc_a("hl"),
                0x9f => self.sbc_a("a"),
                0xa0 => self.and("b"),
                0xa1 => self.and("c"),
                0xa2 => self.and("d"),
                0xa3 => self.and("e"),
                0xa4 => self.and("h"),
                0xa5 => self.and("l"),
                0xa6 => self.and("hl"),
                0xa7 => self.and("a"),
                0xa8 => self.xor("b"),
                0xa9 => self.xor("c"),
                0xaa => self.xor("d"),
                0xab => self.xor("e"),
                0xac => self.xor("h"),
                0xad => self.xor("l"),
                0xae => self.xor("hl"),
                0xaf => self.xor("a"),
                0xb0 => self.or("b"),
                0xb1 => self.or("c"),
                0xb2 => self.or("d"),
                0xb3 => self.or("e"),
                0xb4 => self.or("h"),
                0xb5 => self.or("l"),
                0xb6 => self.or("hl"),
                0xb7 => self.or("a"),
                0xb8 => self.cp("b"),
                0xb9 => self.cp("c"),
                0xba => self.cp("d"),
                0xbb => self.cp("e"),
                0xbc => self.cp("h"),
                0xbd => self.cp("l"),
                0xbe => self.cp("hl"),
                0xbf => self.cp("a"),
                0xc0 => {
                    if !self.registers.get_flag("z")? {
                        self.ret();
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xc1 => self.pop_reg("bc"),
                0xc2 => {
                    if !self.registers.get_flag("z")? {
                        self.jp_to(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xc3 => self.jp_to(value),
                0xc4 => {
                    if !self.registers.get_flag("z")? {
                        self.call(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xc5 => self.push_reg("bc"),
                0xc6 => {
                    let (result, overflow) = self.registers.a.overflowing_add(value as u8);

                    self.registers.set_flag("z", result == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers
                        .set_flag("h", (self.registers.a & 0xF) + (value as u8 & 0xF) > 0xF)?;
                    self.registers.set_flag("c", overflow)?;

                    self.registers.a = result;
                }
                0xc7 => self.call(0x00),
                0xc8 => {
                    if self.registers.get_flag("z")? {
                        self.ret();
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xc9 => self.ret(),
                0xca => {
                    if self.registers.get_flag("z")? {
                        self.jp_to(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xcb => unreachable!(),
                0xcc => {
                    if self.registers.get_flag("z")? {
                        self.call(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xcd => self.call(value),
                0xce => {
                    let val = self.registers.a as u16;
                    let carry = self.registers.get_flag("c").unwrap() as u16;
                    let res = value;

                    let result = val + res + carry;
                    self.registers.set_flag("z", result & 0xff == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers
                        .set_flag("h", (val & 0xf) + (res & 0xf) + carry > 0xf)?;
                    self.registers.set_flag("c", result > 0xff)?;

                    self.registers.a = (result & 0xff) as u8;
                }
                0xcf => self.call(0x08),
                0xd0 => {
                    if !self.registers.get_flag("c")? {
                        self.ret();
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xd1 => self.pop_reg("de"),
                0xd2 => {
                    if !self.registers.get_flag("c")? {
                        self.jp_to(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xd3 => unreachable!(),
                0xd4 => {
                    if !self.registers.get_flag("c")? {
                        self.call(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xd5 => self.push_reg("de"),
                0xd6 => {
                    let (result, overflow) = self.registers.a.overflowing_sub(value as u8);

                    self.registers.set_flag("z", result == 0).unwrap();
                    self.registers.set_flag("n", true).unwrap();
                    self.registers
                        .set_flag("h", (self.registers.a & 0xF) < (value & 0xF) as u8)
                        .unwrap();
                    self.registers.set_flag("c", overflow).unwrap();

                    self.registers.a = result
                }
                0xd7 => self.call(0x10),
                0xd8 => {
                    if self.registers.get_flag("c")? {
                        self.ret();
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xd9 => {
                    self.motherboard.i_master.set(true);
                    self.ret();
                }
                0xda => {
                    if self.registers.get_flag("c")? {
                        self.jp_to(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xdb => unreachable!(),
                0xdc => {
                    if self.registers.get_flag("c")? {
                        self.call(value);
                    } else {
                        return Ok(cycles[1].as_u64().unwrap() as u8);
                    }
                }
                0xdd => unreachable!(),
                0xde => {
                    let val = self.registers.a as i16;
                    let carry = self.registers.get_flag("c")? as i16;
                    let res = value as i16;

                    let result = val - res - carry;

                    self.registers.set_flag("z", result & 0xff == 0)?;
                    self.registers.set_flag("n", true)?;
                    self.registers
                        .set_flag("h", (val & 0xf) < (res & 0xf) + carry)?;
                    self.registers.set_flag("c", result < 0x0)?;

                    self.registers.a = (result & 0xff) as u8
                }
                0xdf => self.call(0x18),
                0xe0 => {
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
                    self.memory.set(value + 0xFF00, self.registers.a);
                }
                0xe1 => self.pop_reg("hl"),
                0xe2 => {
                    let ptr = self.registers.c as u16;
                    self.memory.set(ptr + 0xff00, self.registers.a);
                }
                0xe3 => unreachable!(),
                0xe4 => unreachable!(),
                0xe5 => self.push_reg("hl"),
                0xe6 => {
                    self.registers.a &= value as u8;
                    self.registers.set_flag("z", self.registers.a == 0).unwrap();
                    self.registers.set_flag("n", false).unwrap();
                    self.registers.set_flag("h", true).unwrap();
                    self.registers.set_flag("c", false).unwrap();
                }
                0xe7 => self.call(0x20),
                0xe8 => {
                    let (r, c, h);
                    let r8 = value as i8;

                    if r8 >= 0 {
                        c = ((self.registers.sp & 0xFF) + r8 as u16) > 0xFF;
                        h = ((self.registers.sp & 0x0F) as u8 + (r8 as u8 & 0xF)) > 0x0F;
                        r = self.registers.sp.wrapping_add(r8 as u16);
                    } else {
                        r = self.registers.sp.wrapping_add_signed(r8 as i16);
                        c = (r & 0xFF) <= (self.registers.sp & 0xFF);
                        h = (r & 0x0F) <= (self.registers.sp & 0x0F);
                    }
                    self.registers.sp = r;
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", h)?;
                    self.registers.set_flag("c", c)?;
                }
                0xe9 => self.jp_to(self.registers.get_u16_reg("hl")?),
                0xea => {
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 8);
                    self.memory.set(value, self.registers.a)
                },
                0xeb => unreachable!(),
                0xec => unreachable!(),
                0xed => unreachable!(),
                0xee => {
                    self.registers.a ^= value as u8;
                    self.registers.set_flag("z", self.registers.a == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", false)?;
                }
                0xef => self.call(0x28),
                0xf0 => {
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 4);
                    self.registers.a = self.memory.get(value + 0xff00) 
                },
                0xf1 => {
                    self.pop_reg("af");
                    self.registers.f &= 0xf0;
                }
                0xf2 => self.registers.a = self.memory.get(self.registers.c as u16 + 0xff00),
                0xf3 => self.motherboard.i_master.set(false),
                0xf4 => unreachable!(),
                0xf5 => {
                    self.registers.f &= 0xf0;
                    self.push_reg("af");
                }
                0xf6 => {
                    self.registers.a |= value as u8;
                    self.registers.set_flag("z", self.registers.a == 0)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", false)?;
                    self.registers.set_flag("c", false)?;
                }
                0xf7 => self.call(0x30),
                0xf8 => {
                    let (r, c, h);
                    let r8 = value as i8;
                    if r8 >= 0 {
                        c = ((self.registers.sp & 0xFF) + r8 as u16) > 0xFF;
                        h = ((self.registers.sp & 0x0F) as u8 + (r8 as u8 & 0xF)) > 0x0F;
                        r = self.registers.sp.wrapping_add(r8 as u16);
                    } else {
                        r = self.registers.sp.wrapping_add_signed(r8 as i16);
                        c = (r & 0xFF) <= (self.registers.sp & 0xFF);
                        h = (r & 0x0F) <= (self.registers.sp & 0x0F);
                    }
                    self.registers.set_u16_reg("hl", r)?;
                    self.registers.set_flag("z", false)?;
                    self.registers.set_flag("n", false)?;
                    self.registers.set_flag("h", h)?;
                    self.registers.set_flag("c", c)?;
                }
                0xf9 => self.registers.sp = self.registers.get_u16_reg("hl")?,
                0xfa => {
                    self.motherboard.cycles.set(self.motherboard.cycles.get() + 8);
                    self.registers.a = self.memory.get(value) 
                },
                0xfb => self.motherboard.i_master.set(true),
                0xfc => unreachable!(),
                0xfd => unreachable!(),
                0xfe => {
                    self.registers
                        .set_flag("z", self.registers.a == value as u8)?;
                    self.registers.set_flag("n", true)?;
                    self.registers
                        .set_flag("h", (self.registers.a & 0xf) < value as u8 & 0xf)?;
                    self.registers
                        .set_flag("c", self.registers.a < value as u8)?;
                }
                0xff => self.call(value),
                _ => unreachable!(), // makes the compiler stop complaining
            }
        }
        // cb prefixed instructions
        else {
            match opcode {
                0x00 => self.rlc("b"),
                0x01 => self.rlc("c"),
                0x02 => self.rlc("d"),
                0x03 => self.rlc("e"),
                0x04 => self.rlc("h"),
                0x05 => self.rlc("l"),
                0x06 => self.rlc("hl"),
                0x07 => self.rlc("a"),
                0x08 => self.rrc("b"),
                0x09 => self.rrc("c"),
                0x0a => self.rrc("d"),
                0x0b => self.rrc("e"),
                0x0c => self.rrc("h"),
                0x0d => self.rrc("l"),
                0x0e => self.rrc("hl"),
                0x0f => self.rrc("a"),
                0x10 => self.rl("b"),
                0x11 => self.rl("c"),
                0x12 => self.rl("d"),
                0x13 => self.rl("e"),
                0x14 => self.rl("h"),
                0x15 => self.rl("l"),
                0x16 => self.rl("hl"),
                0x17 => self.rl("a"),
                0x18 => self.rr("b"),
                0x19 => self.rr("c"),
                0x1a => self.rr("d"),
                0x1b => self.rr("e"),
                0x1c => self.rr("h"),
                0x1d => self.rr("l"),
                0x1e => self.rr("hl"),
                0x1f => self.rr("a"),
                0x20 => self.sla("b"),
                0x21 => self.sla("c"),
                0x22 => self.sla("d"),
                0x23 => self.sla("e"),
                0x24 => self.sla("h"),
                0x25 => self.sla("l"),
                0x26 => self.sla("hl"),
                0x27 => self.sla("a"),
                0x28 => self.sra("b"),
                0x29 => self.sra("c"),
                0x2a => self.sra("d"),
                0x2b => self.sra("e"),
                0x2c => self.sra("h"),
                0x2d => self.sra("l"),
                0x2e => self.sra("hl"),
                0x2f => self.sra("a"),
                0x30 => self.swap("b"),
                0x31 => self.swap("c"),
                0x32 => self.swap("d"),
                0x33 => self.swap("e"),
                0x34 => self.swap("h"),
                0x35 => self.swap("l"),
                0x36 => self.swap("hl"),
                0x37 => self.swap("a"),
                0x38 => self.srl("b"),
                0x39 => self.srl("c"),
                0x3a => self.srl("d"),
                0x3b => self.srl("e"),
                0x3c => self.srl("h"),
                0x3d => self.srl("l"),
                0x3e => self.srl("hl"),
                0x3f => self.srl("a"),
                0x40 => self.bit("b", 0),
                0x41 => self.bit("c", 0),
                0x42 => self.bit("d", 0),
                0x43 => self.bit("e", 0),
                0x44 => self.bit("h", 0),
                0x45 => self.bit("l", 0),
                0x46 => self.bit("hl", 0),
                0x47 => self.bit("a", 0),
                0x48 => self.bit("b", 1),
                0x49 => self.bit("c", 1),
                0x4a => self.bit("d", 1),
                0x4b => self.bit("e", 1),
                0x4c => self.bit("h", 1),
                0x4d => self.bit("l", 1),
                0x4e => self.bit("hl", 1),
                0x4f => self.bit("a", 1),
                0x50 => self.bit("b", 2),
                0x51 => self.bit("c", 2),
                0x52 => self.bit("d", 2),
                0x53 => self.bit("e", 2),
                0x54 => self.bit("h", 2),
                0x55 => self.bit("l", 2),
                0x56 => self.bit("hl", 2),
                0x57 => self.bit("a", 2),
                0x58 => self.bit("b", 3),
                0x59 => self.bit("c", 3),
                0x5a => self.bit("d", 3),
                0x5b => self.bit("e", 3),
                0x5c => self.bit("h", 3),
                0x5d => self.bit("l", 3),
                0x5e => self.bit("hl", 3),
                0x5f => self.bit("a", 3),
                0x60 => self.bit("b", 4),
                0x61 => self.bit("c", 4),
                0x62 => self.bit("d", 4),
                0x63 => self.bit("e", 4),
                0x64 => self.bit("h", 4),
                0x65 => self.bit("l", 4),
                0x66 => self.bit("hl", 4),
                0x67 => self.bit("a", 4),
                0x68 => self.bit("b", 5),
                0x69 => self.bit("c", 5),
                0x6a => self.bit("d", 5),
                0x6b => self.bit("e", 5),
                0x6c => self.bit("h", 5),
                0x6d => self.bit("l", 5),
                0x6e => self.bit("hl", 5),
                0x6f => self.bit("a", 5),
                0x70 => self.bit("b", 6),
                0x71 => self.bit("c", 6),
                0x72 => self.bit("d", 6),
                0x73 => self.bit("e", 6),
                0x74 => self.bit("h", 6),
                0x75 => self.bit("l", 6),
                0x76 => self.bit("hl", 6),
                0x77 => self.bit("a", 6),
                0x78 => self.bit("b", 7),
                0x79 => self.bit("c", 7),
                0x7a => self.bit("d", 7),
                0x7b => self.bit("e", 7),
                0x7c => self.bit("h", 7),
                0x7d => self.bit("l", 7),
                0x7e => self.bit("hl", 7),
                0x7f => self.bit("a", 7),
                0x80 => self.res("b", 0),
                0x81 => self.res("c", 0),
                0x82 => self.res("d", 0),
                0x83 => self.res("e", 0),
                0x84 => self.res("h", 0),
                0x85 => self.res("l", 0),
                0x86 => self.res("hl", 0),
                0x87 => self.res("a", 0),
                0x88 => self.res("b", 1),
                0x89 => self.res("c", 1),
                0x8a => self.res("d", 1),
                0x8b => self.res("e", 1),
                0x8c => self.res("h", 1),
                0x8d => self.res("l", 1),
                0x8e => self.res("hl", 1),
                0x8f => self.res("a", 1),
                0x90 => self.res("b", 2),
                0x91 => self.res("c", 2),
                0x92 => self.res("d", 2),
                0x93 => self.res("e", 2),
                0x94 => self.res("h", 2),
                0x95 => self.res("l", 2),
                0x96 => self.res("hl", 2),
                0x97 => self.res("a", 2),
                0x98 => self.res("b", 3),
                0x99 => self.res("c", 3),
                0x9a => self.res("d", 3),
                0x9b => self.res("e", 3),
                0x9c => self.res("h", 3),
                0x9d => self.res("l", 3),
                0x9e => self.res("hl", 3),
                0x9f => self.res("a", 3),
                0xa0 => self.res("b", 4),
                0xa1 => self.res("c", 4),
                0xa2 => self.res("d", 4),
                0xa3 => self.res("e", 4),
                0xa4 => self.res("h", 4),
                0xa5 => self.res("l", 4),
                0xa6 => self.res("hl", 4),
                0xa7 => self.res("a", 4),
                0xa8 => self.res("b", 5),
                0xa9 => self.res("c", 5),
                0xaa => self.res("d", 5),
                0xab => self.res("e", 5),
                0xac => self.res("h", 5),
                0xad => self.res("l", 5),
                0xae => self.res("hl", 5),
                0xaf => self.res("a", 5),
                0xb0 => self.res("b", 6),
                0xb1 => self.res("c", 6),
                0xb2 => self.res("d", 6),
                0xb3 => self.res("e", 6),
                0xb4 => self.res("h", 6),
                0xb5 => self.res("l", 6),
                0xb6 => self.res("hl", 6),
                0xb7 => self.res("a", 6),
                0xb8 => self.res("b", 7),
                0xb9 => self.res("c", 7),
                0xba => self.res("d", 7),
                0xbb => self.res("e", 7),
                0xbc => self.res("h", 7),
                0xbd => self.res("l", 7),
                0xbe => self.res("hl", 7),
                0xbf => self.res("a", 7),
                0xc0 => self.set("b", 0),
                0xc1 => self.set("c", 0),
                0xc2 => self.set("d", 0),
                0xc3 => self.set("e", 0),
                0xc4 => self.set("h", 0),
                0xc5 => self.set("l", 0),
                0xc6 => self.set("hl", 0),
                0xc7 => self.set("a", 0),
                0xc8 => self.set("b", 1),
                0xc9 => self.set("c", 1),
                0xca => self.set("d", 1),
                0xcb => self.set("e", 1),
                0xcc => self.set("h", 1),
                0xcd => self.set("l", 1),
                0xce => self.set("hl", 1),
                0xcf => self.set("a", 1),
                0xd0 => self.set("b", 2),
                0xd1 => self.set("c", 2),
                0xd2 => self.set("d", 2),
                0xd3 => self.set("e", 2),
                0xd4 => self.set("h", 2),
                0xd5 => self.set("l", 2),
                0xd6 => self.set("hl", 2),
                0xd7 => self.set("a", 2),
                0xd8 => self.set("b", 3),
                0xd9 => self.set("c", 3),
                0xda => self.set("d", 3),
                0xdb => self.set("e", 3),
                0xdc => self.set("h", 3),
                0xdd => self.set("l", 3),
                0xde => self.set("hl", 3),
                0xdf => self.set("a", 3),
                0xe0 => self.set("b", 4),
                0xe1 => self.set("c", 4),
                0xe2 => self.set("d", 4),
                0xe3 => self.set("e", 4),
                0xe4 => self.set("h", 4),
                0xe5 => self.set("l", 4),
                0xe6 => self.set("hl", 4),
                0xe7 => self.set("a", 4),
                0xe8 => self.set("b", 5),
                0xe9 => self.set("c", 5),
                0xea => self.set("d", 5),
                0xeb => self.set("e", 5),
                0xec => self.set("h", 5),
                0xed => self.set("l", 5),
                0xee => self.set("hl", 5),
                0xef => self.set("a", 5),
                0xf0 => self.set("b", 6),
                0xf1 => self.set("c", 6),
                0xf2 => self.set("d", 6),
                0xf3 => self.set("e", 6),
                0xf4 => self.set("h", 6),
                0xf5 => self.set("l", 6),
                0xf6 => self.set("hl", 6),
                0xf7 => self.set("a", 6),
                0xf8 => self.set("b", 7),
                0xf9 => self.set("c", 7),
                0xfa => self.set("d", 7),
                0xfb => self.set("e", 7),
                0xfc => self.set("h", 7),
                0xfd => self.set("l", 7),
                0xfe => self.set("hl", 7),
                0xff => self.set("a", 7),
                _ => unreachable!(),
            }
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
    pub fn gen_log(&self) -> String {
        let a = self.registers.a;
        let f = self.registers.f;
        let b = self.registers.b;
        let c = self.registers.c;
        let d = self.registers.d;
        let e = self.registers.e;
        let h = self.registers.h;
        let l = self.registers.l;
        let sp = self.registers.sp;
        let pc = self.registers.pc;
        let mem1 = self.memory.get(pc);
        let mem2 = self.memory.get(pc + 1);
        let mem3 = self.memory.get(pc + 2);
        let mem4 = self.memory.get(pc + 3);
        format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            a, f, b, c, d, e, h, l, sp, pc, mem1, mem2, mem3, mem4
        )
    }
    pub fn run(&mut self) {
        let mut file = File::create("log.txt").unwrap();
        while true {
            //writeln!(file, "{}", self.gen_log()).unwrap();
            self.update();
        }
    }

    // runs one full cpu tick
    pub fn update(&mut self) {
        let cycles: u8;

        // blargg debug
        self.handle_blargg();

        if !self.halt {
            cycles = self.execute_next_op();
        } else {
            cycles = 4;
        }
        
        let timer_int = self.motherboard.timer.borrow_mut().tick(cycles - self.motherboard.sync_cycles.get());
        if timer_int {
            self.set_interrupt(2);
        }
        
        // reset cycle counters
        self.motherboard.sync_cycles.set(0);
        self.motherboard.cycles.set(0);
        
        // TODO: add screen

        // Interrupt handling
        if self.check_interrupt() {
            self.halt = false;
        }

        if self.halt && self.i_queue {
            self.halt = false;
            self.registers.pc += 1;
        }

        self.i_queue = false;
    }

    fn set_interrupt(&mut self, bit: u8) {
        let flag = 1 << bit;
        self.motherboard
            .i_flag
            .set(self.motherboard.i_flag.get() | flag);
    }

    fn check_interrupt(&mut self) -> bool {
        let total =
            (self.motherboard.i_enable.get() & 0b11111) & (self.motherboard.i_flag.get() & 0b11111);
        if total != 0 {
            if self.motherboard.i_master.get() {
                // vblank
                if total & 0b1 != 0 {
                    self.handle_interrupt(0b1, 0x40);
                }
                // lcd
                else if total & 0b10 != 0 {
                    self.handle_interrupt(0b10, 0x48);
                }
                // timer
                else if total & 0b100 != 0 {
                    self.handle_interrupt(0b100, 0x50);
                }
                // serial
                else if total & 0b1000 != 0 {
                    self.handle_interrupt(0b1000, 0x58);
                }
                // joypad
                else if total & 0b10000 != 0 {
                    self.handle_interrupt(0b10000, 0x60);
                }
            }
            self.i_queue = true;
            true
        } else {
            self.i_queue = false;
            false
        }
    }

    fn handle_interrupt(&mut self, flag: u8, address: u16) {
        // clear flag
        self.motherboard
            .i_flag
            .set(self.motherboard.i_flag.get() ^ flag);
        self.call(address);
        self.motherboard.i_master.set(false);
    }

    fn handle_blargg(&mut self) {
        let mut temp = false;
        if self.memory.get(0xff02) == 0x81 {
            let val = self.memory.get(0xff01) as char;
            self.blargg.push(val);
            self.memory.set(0xff02, 0);
            temp = true;
        }

        if !self.blargg.is_empty() && temp {
            println!("{}", self.blargg)
        }
    }
}
