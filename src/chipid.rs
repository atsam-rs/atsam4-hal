use crate::pac::CHIPID;

#[derive(Debug)]
pub enum EmbeddedProcessor {
    Arm946Es,
    Arm7Tdmi,
    CortexM3,
    Arm920T,
    Arm926Ejs,
    CortexA5,
    CortexM4,
}

#[derive(Debug)]
pub enum Architecture {
    AtSam4E,
    AtSam4SxA,
    AtSam4SxB,
    AtSam4SxC,
}

#[derive(Debug)]
pub enum FlashMemoryType {
    Rom,
    Romless,
    Sram,
    Flash,
    RomFlash, // flash1_byte_size = ROM size, flash2_byte_size = Flash size
}

#[derive(Debug)]
pub struct ChipId {
    pub version: u8,
    pub embedded_processor: Option<EmbeddedProcessor>,
    pub flash1_byte_size: Option<usize>,
    pub flash2_byte_size: Option<usize>,
    pub internal_sram_byte_size: Option<usize>,
    pub architecture: Option<Architecture>,
    pub flash_memory_type: Option<FlashMemoryType>,
}

impl ChipId {
    pub fn new(chip_id: CHIPID) -> Self {
        let cidr = chip_id.cidr.read();
        let version = cidr.version().bits();
        let embedded_processor = match cidr.eproc().bits() {
            1 => Some(EmbeddedProcessor::Arm946Es),
            2 => Some(EmbeddedProcessor::Arm7Tdmi),
            3 => Some(EmbeddedProcessor::CortexM3),
            4 => Some(EmbeddedProcessor::Arm920T),
            5 => Some(EmbeddedProcessor::Arm926Ejs),
            6 => Some(EmbeddedProcessor::CortexA5),
            7 => Some(EmbeddedProcessor::CortexM4),
            _ => None,
        };
        let flash1_byte_size = Self::get_flash_size_from_register(cidr.nvpsiz().bits());
        let flash2_byte_size = Self::get_flash_size_from_register(cidr.nvpsiz2().bits());
        let internal_sram_byte_size = match cidr.sramsiz().bits() {
            0 => Some(48 * 1024),
            1 => Some(192 * 1024),
            2 => Some(2 * 1024),
            3 => Some(6 * 1024),
            4 => Some(24 * 1024),
            5 => Some(4 * 1024),
            6 => Some(80 * 1024),
            7 => Some(160 * 1024),
            8 => Some(8 * 1024),
            9 => Some(16 * 1024),
            10 => Some(32 * 1024),
            11 => Some(64 * 1024),
            12 => Some(128 * 1024),
            13 => Some(256 * 1024),
            14 => Some(96 * 1024),
            15 => Some(512 * 1024),
            _ => None,
        };
        let architecture = match cidr.arch().bits() {
            0x3C => Some(Architecture::AtSam4E),
            0x88 => Some(Architecture::AtSam4SxA),
            0x89 => Some(Architecture::AtSam4SxB),
            0x8A => Some(Architecture::AtSam4SxC),
            _ => None,
        };
        let flash_memory_type = match cidr.nvptyp().bits() {
            0 => Some(FlashMemoryType::Rom),
            1 => Some(FlashMemoryType::Romless),
            2 => Some(FlashMemoryType::Flash),
            3 => Some(FlashMemoryType::RomFlash),
            4 => Some(FlashMemoryType::Sram),
            _ => None,
        };

        ChipId {
            version,
            embedded_processor,
            flash1_byte_size,
            flash2_byte_size,
            internal_sram_byte_size,
            architecture,
            flash_memory_type,
        }
    }

    fn get_flash_size_from_register(register_value: u8) -> Option<usize> {
        match register_value {
            1 => Some(8 * 1024),
            2 => Some(16 * 1024),
            3 => Some(32 * 1024),
            5 => Some(64 * 1024),
            7 => Some(128 * 1024),
            9 => Some(256 * 1024),
            10 => Some(512 * 1024),
            12 => Some(1024 * 1024),
            14 => Some(2048 * 1024),
            _ => None,
        }
    }
}
