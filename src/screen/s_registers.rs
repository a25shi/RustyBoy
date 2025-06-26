// Palette register
pub struct Palette {
    value: u8,
    lookup: [u8; 4],
    palette: [u8; 4],
}
impl Palette {
    pub fn new(value: u8) -> Self {
        Self {
            value,
            lookup: [0; 4],
            palette: [0xFF, 0x99, 0x55, 0x00],
        }
    }

    pub fn set(&mut self, value: u8) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        for x in 0..4 {
            self.lookup[x] = self.palette[((value >> x * 2) & 0b11) as usize]
        }
        true
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn get_color(&self, index: u8) -> u8 {
        self.lookup[index as usize]
    }
}
// lcdc register
pub struct LCDC {
    value: u8,
    pub lcd_enable: bool,
    pub windowmap_select: bool,
    pub window_enable: bool,
    pub tiledata_select: bool,
    pub backgroundmap_select: bool,
    pub sprite_height: bool,
    pub sprite_enable: bool,
    pub background_enable: bool,
    pub backgroundmap_offset: u16,
    pub windowmap_offset: u16,
}

impl LCDC {
    pub fn new() -> Self {
        Self {
            value: 0,
            lcd_enable: false,
            windowmap_select: false,
            window_enable: false,
            tiledata_select: false,
            backgroundmap_select: false,
            sprite_height: false,
            sprite_enable: false,
            background_enable: false,
            backgroundmap_offset: 0x1800,
            windowmap_offset: 0x1800
        }
    }
    pub fn set(&mut self, value: u8) {
        self.lcd_enable = value & (1 << 7) != 0;
        self.windowmap_select = value & (1 << 6) != 0;
        self.window_enable = value & (1 << 5) != 0;
        self.tiledata_select = value & (1 << 4) != 0;
        self.backgroundmap_select = value & (1 << 3) != 0;
        self.sprite_height = value & (1 << 2) != 0;
        self.sprite_enable = value & (1 << 1) != 0;
        self.background_enable = value & (1 << 0) != 0;

        // All VRAM addresses are offset by 0x8000
        // Following addresses are 0x9800 and 0x9C00
        self.backgroundmap_offset = if !self.backgroundmap_select { 0x1800 } else { 0x1C00 };
        self.windowmap_offset = if !self.windowmap_select { 0x1800 } else { 0x1C00 }
    }
}

// Stat register
pub struct STAT {
    pub value: u8,
    pub mode: u8
}

impl STAT {
    pub fn new() -> Self {
        Self {
            value: 0b1000_0000,
            mode: 0
        }
    }
    pub fn set_mode(&mut self, mode: u8) -> bool {
        if self.mode == mode {
            return false;
        }
        
        self.mode = mode;
        self.value &= 0b11111100;
        self.value |= mode;

        if mode != 3 && (self.value & (1 << (mode + 3)) != 0) {
            return true;
        }
        
        false
    }
    pub fn update_lyc(&mut self, lyc: u8, ly: u8) -> bool {
        if lyc == ly {
            self.value |= 0b100; // clears flag
            if self.value & 0b0100_0000 != 0 {
                return true;
            } 
        }
        else {
            self.value &= 0b1111_1011;
        }
        false
    }
}