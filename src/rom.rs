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

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOURSCREEN,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        let header = Header::parse_header(raw);

        if header.nestag != NESTAG {
            return Err("File is not in iNES file format".to_string());
        }

        let has_ines_ver2 = header
            .control2
            .intersects(RomControlByte2::INES_V2_1 | RomControlByte2::INES_V2_2);

        if has_ines_ver2 {
            return Err("NES2.0 format is not supported".to_string());
        };

        let mapper_lower = (header.control1.bits() & RomControlByte1::MAPPER_LOWER.bits()) >> 4;
        let mapper_upper = (header.control2.bits() & RomControlByte2::MAPPER_UPPER.bits()) >> 4;
        let mapper = mapper_lower | mapper_upper;

        let four_screen = header.control1.contains(RomControlByte1::FOUR_SCREEN);
        let vertical_mirroring = header
            .control1
            .contains(RomControlByte1::VERTICAL_MIRRORING);
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOURSCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = header.prg_rom_size as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = header.chr_rom_size as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = header.control1.contains(RomControlByte1::TRAINER);

        let prg_rom_start = HEADER_SIZE + if skip_trainer { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        })
    }
}

pub mod test {

    use super::*;

    pub struct TestRom {
        pub header: Vec<u8>,
        pub trainer: Option<Vec<u8>>,
        pub pgp_rom: Vec<u8>,
        pub chr_rom: Vec<u8>,
    }

    pub fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer.as_ref().map_or(0, |t| t.len())
                + rom.pgp_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.pgp_rom);
        result.extend(&rom.chr_rom);

        result
    }

    pub fn test_rom(program: Vec<u8>) -> Rom {
        let mut test_rom = TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        };

        // 先頭要素からプログラムのサイズ分を書き換える
        test_rom.pgp_rom[0..program.len()].copy_from_slice(&program);

        Rom::new(&create_rom(test_rom)).unwrap()
    }

    #[test]
    fn test() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_with_trainer() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E,
                0x45,
                0x53,
                0x1A,
                0x02,
                0x01,
                0x31 | 0b100,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
            ],
            trainer: Some(vec![0; 512]),
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_nes2_is_not_supported() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x31, 0x8, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 1 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom);
        match rom {
            Result::Ok(_) => assert!(false, "should not load rom"),
            Result::Err(str) => assert_eq!(str, "NES2.0 format is not supported"),
        }
    }
}
