use crate::screen::s_registers::{Palette, LCDC, STAT};
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
    pub OBP1: Palette
}

impl Screen {
    pub fn new() -> Self {
        Self {
            VRAM: [0; 0x2000],
            OAM: [0; 0xa0],
            LCDC: LCDC::new(),
            STAT: STAT::new(),
            SCY: 0,
            SCX: 0,
            WY: 0,
            WY_counter: 0,
            WX: 0,
            LY: 0,
            LYC: 0,
            scan_counter: 456,
            next_mode: 2,
            BGP: Palette::new(0xfc),
            OBP0: Palette::new(0xff),
            OBP1: Palette::new(0xff),
        }
    }
}


