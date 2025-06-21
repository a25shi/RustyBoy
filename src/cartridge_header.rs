#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CartridgeHeader {
    /// 0104-0133: Logo
    pub logo: [u8; 48],
    /// 0134-0143: Title
    pub title: [u8; 16],
    ///0143: CGB Flag
    pub cgb_flag: u8,
    /// 0146: SGB Flag
    pub sgb_flag: u8,
    /// 0147: Cartridge Type
    pub cartridge_type: u8,
    /// 0148: ROM Size
    pub rom_size: u8,
    /// 0149: RAM Size
    pub ram_size: u8,
    /// 014C: Mask ROM Version number
    pub version: u8,
    /// 014D: Header Checksum
    pub header_checksum: u8,
    /// 014E-014F: Global Checksum
    pub global_checksum: u16,
}

impl CartridgeHeader {
    /// Return  Err(Some(Self)) if the load was successful but the checksum don't match.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, (Option<Self>, String)> {
        if bytes.len() < 0x150 {
            return Err((None, "file has less than 0x150 bytes".to_string()));
        }
        let this = Self {
            logo: bytes[0x0104..=0x0133].try_into().unwrap(),
            title: bytes[0x0134..=0x0143].try_into().unwrap(),
            cgb_flag: bytes[0x143],
            sgb_flag: bytes[0x0146],
            cartridge_type: bytes[0x0147],
            rom_size: bytes[0x0148],
            ram_size: bytes[0x0149],
            version: bytes[0x014C],
            header_checksum: bytes[0x014D],
            global_checksum: u16::from_le_bytes([bytes[0x014E], bytes[0x014F]]),
        };

        {
            if Self::compute_check_sum(bytes) != this.header_checksum {
                return Err((Some(this), "checksum don't match".to_string()));
            }
        }
        Ok(this)
    }
    pub fn compute_check_sum(bytes: &[u8]) -> u8 {
        bytes[0x134..=0x014C]
            .iter()
            .fold(0u8, |x, &b| x.wrapping_add(!b))
    }
}