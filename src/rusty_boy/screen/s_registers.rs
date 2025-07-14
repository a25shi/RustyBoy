const TILE_COUNT: usize = 384;

// Palette register
pub struct Palette {
    value: u8,
    lookup: [u8; 4],
    palette: [u8; 4],
}
impl Palette {
    pub fn new(value: u8) -> Self {
        let mut this = Self {
            value: 0,
            lookup: [0; 4],
            palette: [0xFF, 0xAA, 0x55, 0x00],
        };
        this.set(value);
        this
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
    pub value: u8,
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
        self.value = value;
        
        self.lcd_enable = value & (1 << 7) != 0;
        self.windowmap_select = value & (1 << 6) != 0;
        self.window_enable = value & (1 << 5) != 0;
        self.tiledata_select = value & (1 << 4) != 0;
        self.backgroundmap_select = value & (1 << 3) != 0;
        self.sprite_height = value & (1 << 2) != 0;
        self.sprite_enable = value & (1 << 1) != 0;
        self.background_enable = value & (1 << 0) != 0;
        
        //println!("Writing value {} to background enable", value & (1 << 0));
        
        // All VRAM addresses are offset by 0x8000
        // Following addresses are 0x9800 and 0x9C00
        self.backgroundmap_offset = if !self.backgroundmap_select { 0x1800 } else { 0x1C00 };
        self.windowmap_offset = if !self.windowmap_select { 0x1800 } else { 0x1C00 };
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
    pub fn set(&mut self, value: u8) {
        let newvalue = value & 0b0111_1000;
        self.value &= 0b1000_0111;
        self.value |= newvalue;
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
            self.value |= 0b100; // sets flag
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

// Tile cache for bg for now
pub struct TileCache {
    tile_state: [bool; TILE_COUNT], // checks which tiles are cached
    pub tile_cache: [u8; TILE_COUNT * 8 * 8] // tile cache
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            tile_state: [false; TILE_COUNT],
            tile_cache: [0; TILE_COUNT * 8 * 8]
        }
    }
    pub fn update_tile(&mut self, tile_index: usize, vram: &[u8]) {
        if self.tile_state[tile_index] {
            return;
        }
        
        // For each line in the tile
        for y in 0..8 {
            let byte1 = vram[tile_index * 16 + y * 2];
            let byte2 = vram[tile_index * 16 + y * 2 + 1];
            
            // For each pixel in the line
            for i in 0..8 {
                // bit 1
                let mut col_index = ((byte2 >> i) & 1) << 1;
                // bit 0
                col_index |= (byte1 >> i) & 1;
                // Rightmost bit is the leftmost pixel (bit 7 = pixel 0, bit 6 = pixel 1, etc.)
                let x = 7 - i;
                // Each tile has 64 pixels, each line is 8 pixels
                self.tile_cache[tile_index * 64 + y * 8 + x] = col_index;
            }
        }
        
        self.tile_state[tile_index] = true;
    }
    pub fn clear_cache(&mut self) {
        self.tile_state = [false; 384]
    }
    pub fn clear_tile(&mut self, tile_index: usize) {
        self.tile_state[tile_index] = false;
    }
}