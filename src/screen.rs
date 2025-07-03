use crate::screen::s_registers::{Palette, TileCache, LCDC, STAT};
use std::collections::BTreeMap;
use std::rc::Weak;
use crate::motherboard::Motherboard;

mod s_registers;

pub struct Screen {
    pub VRAM: [u8; 0x2000],
    pub OAM: [u8; 0xa0],
    pub LCDC: LCDC,
    pub STAT: STAT,
    pub SCY: u8,
    pub SCX: u8,
    pub WY: u8,
    pub WY_counter: u8,
    pub WX: u8,
    pub LY: u8,
    pub LYC: u8,
    pub scan_counter: isize,
    pub next_mode: u8,
    pub BGP: Palette,
    pub OBP0: Palette,
    pub OBP1: Palette,
    // Screen buffer RGBA
    pub screen_buffer: Vec<u8>,
    // Screen buffer color index
    pub screen_buffer_color: [u8; 160 * 144],

    // tile cache
    pub tile_cache: TileCache,
    pub frame_done: bool,

    // motherboard pointer
    motherboard: Weak<Motherboard>
}

impl Screen {
    pub fn new(mb: Weak<Motherboard>) -> Self {
        Self {
            VRAM: [0; 0x2000],
            OAM: [0; 0xa0],
            LCDC: LCDC::new(),
            STAT: STAT::new(),
            SCY: 0,
            SCX: 0,
            WY: 0,
            WY_counter: 255,
            WX: 0,
            LY: 0,
            LYC: 0,
            scan_counter: 456,
            next_mode: 2,
            BGP: Palette::new(0xfc),
            OBP0: Palette::new(0xff),
            OBP1: Palette::new(0xff),
            // RGBA
            screen_buffer: [0; 160 * 144 * 4].to_vec(),
            screen_buffer_color: [0; 160 * 144],
            tile_cache: TileCache::new(),
            frame_done: false,
            motherboard: mb
        }
    }
    // Ticks on cycles
    pub fn update(&mut self, cycles: u8) {
        if cycles == 0 {
            return;
        }

        if self.LCDC.lcd_enable {
            self.scan_counter -= cycles as isize;
        } else {
            return;
        }

        // next scanline
        if self.scan_counter <= 0 {
            if self.LY == 153 {
                self.LY = 0;
                // oam logic without inc
                self.set_mode(2);
                self.scan_counter += 80;
                self.next_mode = 3;
                self.check_lyc();
            }
            else {
                self.set_mode(self.next_mode);
                // oam mode 2
                if self.STAT.mode == 2 {
                    self.LY += 1;
                    self.scan_counter += 80;
                    self.next_mode = 3;
                    self.check_lyc();
                }

                // pixel draw mode 3
                else if self.STAT.mode == 3 {
                    self.scan_counter += 172;
                    self.next_mode = 0;
                }

                // hblank mode 0
                else if self.STAT.mode == 0 {
                    self.scan_counter += 204;
                    self.draw_scanline();
                    if self.LY < 143 {
                        self.next_mode = 2;
                    } else {
                        self.next_mode = 1;
                    }
                }

                // vblank mode 1
                else if self.STAT.mode == 1 {
                    self.scan_counter += 456;
                    self.next_mode = 1;
                    self.LY += 1;
                    self.check_lyc();

                    if self.LY == 144 {
                        // V-blank interrupt
                        //println!("Vblank interrupt");
                            
                        if let Some(motherboard_strong) = self.motherboard.upgrade() {
                            motherboard_strong.set_interrupt(0);
                        } else {
                            println!("Could not upgrade, motherboard does not exist??");
                        }
                        // Frame finished 
                        self.frame_done = true;
                    }
                }
            }
        }
    }
    pub fn set(&mut self, address: u16, value: u8) {
        match address { 
            // vram
            0x8000..0xa000 => {
                self.VRAM[address as usize - 0x8000] = value;
                if address < 0x9800 {
                    self.tile_cache.clear_tile((((address & 0xfff0) - 0x8000) / 16) as usize)
                }
            }
            //oam
            0xfe00..0xfea0 => self.OAM[address as usize - 0xfe00] = value,
            // moved to memory.rs to set interrupt
            0xff40 => {}
            0xff41 => self.STAT.set(value),
            0xff42 => self.SCY = value,
            0xff43 => self.SCX = value,
            0xff44 => {} // read only,
            0xff45 => self.LYC = value,
            0xff46 => {} // dma
            0xff47 => {
                if self.BGP.set(value) {
                    self.tile_cache.clear_cache();
                }
            }
            0xff48 => { 
                self.OBP0.set(value); 
            }
            0xff49 => {
                self.OBP1.set(value);
            }
            0xff4a => self.WY = value,
            0xff4b => self.WX = value,
            _ => unreachable!()
        }
    }
    pub fn get(&self, address: u16) -> u8 {
        match address {
            0x8000..0xa000 => self.VRAM[address as usize - 0x8000],
            0xfe00..0xfea0 => self.OAM[address as usize - 0xfe00],
            0xff40 => self.LCDC.value,
            0xff41 => self.STAT.value,
            0xff42 => self.SCY,
            0xff43 => self.SCX,
            0xff44 => self.LY,
            0xff45 => self.LYC,
            0xff46 => 0,
            0xff47 => self.BGP.get(),
            0xff48 => self.OBP0.get(),
            0xff49 => self.OBP1.get(),
            0xff4a => self.WY,
            0xff4b => self.WX,
            _ => unreachable!()
        }
    }
    fn set_pixel_color(&mut self, x: u8, y: u8, color: u8, color_index: Option<u8>) {
        let offset = (y as usize * 160 + x as usize) * 4;
        // Sets screen buffer pixel color
        // RGB
        self.screen_buffer[offset] = color;
        self.screen_buffer[offset + 1] = color;
        self.screen_buffer[offset + 2] = color;
        // A
        self.screen_buffer[offset + 3] = 255;
        
        // Sets screen buffer color index, skips if set to none ( For sprites )
        match color_index {
            Some(index) => {
                self.screen_buffer_color[y as usize * 160 + x as usize] = index;
            },
            None => {}
        }
    }
    fn draw_blank_scanline(&mut self) {
        for x in 0..160 {
            let color = self.BGP.get_color(0);
            self.set_pixel_color(x, self.LY, color, Some(0));
        }
    }
    fn draw_scanline(&mut self) {
        if self.LCDC.window_enable && self.WY <= self.LY && self.WX - 7 < 160 {
            self.WY_counter += 1
        }
        if self.LCDC.background_enable {
            self.draw_background_scanline();
        } else {
            self.draw_blank_scanline();
        }

        if self.LCDC.sprite_enable  {
            self.draw_sprite_scanline();
        }

        if self.LY == 143 {
            // set to max for overflow to 0 on next
            self.WY_counter = 255;
        }
    }
    fn draw_background_scanline(&mut self) {
        let wx = self.WX - 7;
        let mut x_pos: usize;
        let mut y_pos: usize;
        let mut offset: u16;
        let mut xmask: usize;
        let mut xmaskeq: u8;
        let mut tile_index: usize = 0;
        for x in 0..160 {
            if self.LCDC.window_enable && self.WY <= self.LY && x >= wx {
                x_pos = (x - wx) as usize;
                y_pos = self.WY_counter as usize;
                offset = self.LCDC.windowmap_offset;
                xmask = x_pos % 8;
                xmaskeq = wx;
            }
            else {
                x_pos = (x + self.SCX) as usize;
                y_pos = (self.SCY + self.LY) as usize;
                offset = self.LCDC.backgroundmap_offset;
                xmask = ((x + (self.SCX & 0b111)) % 8) as usize;
                xmaskeq = 0;
            }

            if xmask == 0 || x == xmaskeq {
                tile_index = self.get_tile(x_pos, y_pos, offset);
                //println!("{tile_index}");
                self.tile_cache.update_tile(tile_index, &self.VRAM)
            }
            
            // This is with caching
            let color_index = self.tile_cache.tile_cache[tile_index * 64 + (y_pos % 8) * 8 + (x_pos % 8)];
            let color = self.BGP.get_color(color_index);
            // This is without caching
            // tile_index = self.get_tile(x_pos, y_pos, offset);
            // let color = self.get_tile_bgp(tile_index, x_pos as usize, y_pos as usize);
            
            self.set_pixel_color(x, self.LY, color, Some(color_index));
        }
    }
    
    fn get_tile_bgp(&self, tile_index: usize, x: usize, y: usize) -> u8 {
        let line = 2 * (y % 8);
        let pixel_index = 7 - (x % 8);
        
        let byte1 = self.VRAM[tile_index * 16 + line];
        let byte2 = self.VRAM[tile_index * 16 + line + 1];

        let mut col_index = ((byte2 >> pixel_index) & 1) << 1;
        col_index |= (byte1 >> pixel_index) & 1;
        self.BGP.get_color(col_index)
    }
    
    fn draw_sprite_scanline(&mut self) {
        let spriteheight = if self.LCDC.sprite_height { 16 } else { 8 };
        let mut spritecount = 0;
        let mut spritemap: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
        for n in (0..0xa0).step_by(4) {
            // Init as signed to avoid overflow comparisons
            let y = self.OAM[n] as i32 - 16;
            let ly = self.LY as i32;
            let x = self.OAM[n + 1] - 8;
            // within sprite range
            
            if ly < y + spriteheight && ly >= y {
                // insert sprite into bst to sort
                match spritemap.get_mut(&x) {
                    // If the vec exists, append at the end
                    Some(arr) => {
                        arr.push(n)
                    }
                    None => {
                        // Otherwise insert vec with sprite
                        spritemap.insert(x, vec![n]);
                    }
                }
                spritecount += 1;
            }

            // Break from for loop if sprite count hits 10
            if spritecount == 10 {
                break;
            }
        }

        // render the sorted sprites
        // for each oam in the spritemap (sprites with smaller x are given prio)
        for (sprite_x, sprite_n_vec) in spritemap.iter().rev() {
            // for each sprite in the related x (sprites inserted earlier have prio)
            for sprite_n in sprite_n_vec.iter().rev() {
                let y = self.OAM[*sprite_n] - 16;
                let mut tile_index = self.OAM[*sprite_n + 2] as usize;
                let attr = self.OAM[*sprite_n + 3];
                // If spriteheight is 16, ignore bit 0
                if spriteheight == 16 {
                    tile_index &= 0b11111110
                }

                // x and y flip bool
                let yflip: bool = (attr >> 6) & 1 != 0;
                let xflip: bool = (attr >> 5) & 1 != 0;

                // Object priority bool
                let prio: bool = (attr >> 7) & 1 != 0;

                let mut line = self.LY - y;
                
                if yflip {
                    line = spriteheight as u8 - line - 1;
                }
                line *= 2;

                // get sprite byte data
                let byte1 = self.VRAM[tile_index * 16 + line as usize];
                let byte2 = self.VRAM[tile_index * 16 + line as usize + 1];

                // for each 8 sprite pixels in the line
                for i in 0..8 {
                    let mut index = i;
                    // X pixel location
                    let xpixel = 7 - i + sprite_x;

                    if xflip {
                        index = 7 - index;
                    }

                    let mut color_index = ((byte2 >> index) & 1) << 1;
                    color_index |= (byte1 >> index) & 1;
                    
                    let mut color: u8 = 0;
                    // color index 0 is transparent on sprites
                    if color_index == 0 {
                        continue;
                    }

                    // Get from obp1
                    if attr & 0b10000 != 0 {
                        color = self.OBP1.get_color(color_index);
                    }
                    // Get from obp0
                    else {
                        color = self.OBP0.get_color(color_index);
                    }
                    
                    // If bg prio
                    if prio {
                        if self.screen_buffer_color[self.LY as usize * 160 + xpixel as usize] == 0 {
                            // set pixel color
                            self.set_pixel_color(xpixel, self.LY, color, None);
                        }
                    }
                    //sprite shows regardless
                    else {
                        // set pixel color
                        self.set_pixel_color(xpixel, self.LY, color, None);
                    }
                }
            }
        }
    }
    fn get_tile(&self, x: usize, y: usize, offset: u16) -> usize {
        let tile_addr = offset as usize + (y / 8 * 32 % 0x400) + (x / 8 % 32) ;// tilemap offset + tileRow + tileCol
        let mut tile_index = self.VRAM[tile_addr as usize] as usize;
        // if 8800 method
        if !self.LCDC.tiledata_select {
            tile_index += 0x100;
            if tile_index >= 0x180 {
                tile_index -= 0x100;
            }
        }
        tile_index
    }
    // Checks lyc == ly and triggers interrupt if needed
    fn check_lyc(&mut self) {
        let interrupt = self.STAT.update_lyc(self.LYC, self.LY);
        if interrupt {
            if let Some(motherboard_strong) = self.motherboard.upgrade() {
                motherboard_strong.set_interrupt(1);
            } else {
                println!("Could not upgrade, motherboard does not exist??");
            }
        }
    }

    // Sets mode and triggers interrupt if needed
    fn set_mode(&mut self, newmode: u8) {
        let interrupt = self.STAT.set_mode(newmode);
        if interrupt {
            if let Some(motherboard_strong) = self.motherboard.upgrade() {
                motherboard_strong.set_interrupt(1);
            } else {
                println!("Could not upgrade, motherboard does not exist??");
            }
        }
    }
}


