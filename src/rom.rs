use bitflags::bitflags;

const NESTAG: &[u8; 4] = b"NES\x1a"; // "NES^Z" in ASCII
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;

struct Header {
    nestag: Vec<u8>,
    prg_rom_size: u8,
    chr_rom_size: u8,
    control1: RomControlByte1,
    control2: RomControlByte2,
}

bitflags! {
    #[derive(Clone,Copy)]
    pub struct RomControlByte1: u8 {
        const VERTICAL_MIRRORING = 0b0000_0001; // Bit 0
        const HORIZONTAL_MIRRORING = 0b0000_0000; // Bit 0
        const BATTERY = 0b0000_0000; // Bit 1 (0 for INES 1.0)
        const TRAINER = 0b0000_0100; // Bit 2
        const FOUR_SCREEN = 0b0000_1000; // Bit 3
        const MAPPER_LOWER = 0b1111_0000; // Bits 4-7 (Four lower bits of ROM Mapper Type)
    }

    #[derive(Clone,Copy)]
    pub struct RomControlByte2: u8 {
        const INES_V1_1 = 1 ; // Bits 0-1 (INES 1.0 format)
        const INES_V1_2 = 1 << 1; // Bits 0-1 (INES 1.0 format)
        const INES_V2_1 =  1 << 2; // Bits 2-3 (INES 2.0 format)
        const INES_V2_2 = 1 << 3; // Bits 2-3 (INES 2.0 format)
        const MAPPER_UPPER = 0b1111_0000; // Bits 4-7 (Four upper bits of ROM Mapper Type)

    }

}

impl Header {
    fn parse_header(raw: &Vec<u8>) -> Self {
        Header {
            nestag: raw[0..=3].to_vec(),
            prg_rom_size: raw[4],
            chr_rom_size: raw[5],
            control1: RomControlByte1::from_bits_retain(raw[6]),
            control2: RomControlByte2::from_bits_retain(raw[7]),
        }
    }
}

