use std::ops::{Bound, RangeBounds};

pub struct AddressRange {
    pub start: u16,
    pub end: u16,
}

impl RangeBounds<u16> for AddressRange {
    fn start_bound(&self) -> Bound<&u16> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&u16> {
        Bound::Included(&self.end)
    }
}

#[macro_export]
macro_rules! address_range {
    ($name:ident, $start:expr, $end:expr) => {
        pub const $name: AddressRange = AddressRange {
            start: $start,
            end: $end,
        };
    };
}

// addr         size       description
// $0000-$0FFF  $1000     Pattern table 0
// $1000-$1FFF  $1000     Pattern table 1
// $2000-$23FF  $0400     Name table 0
// $2400-$27FF  $0400     Name table 1
// $2800-$2BFF  $0400     Name table 2
// $2C00-$2FFF  $0400     Name table 3
// $3000-$3EFF  $0F00     Mirrors of $2000-$2EFF
// $3F00-$3F1F  $0020     Palette RAM indexes
// $3F20-$3FFF  $00E0     Mirrors of $3F00-$3F1F

address_range!(PPU_MEM_RANGE, 0x0000, 0x3FFF);
address_range!(CHR_ROM_RANGE, 0x0000, 0x1FFF);
address_range!(CHR_ROM_0_RANGE, 0x0000, 0x0FFF);
address_range!(CHR_ROM_1_RANGEE, 0x1000, 0x1FFF);
address_range!(NAME_TABLE_RANGE, 0x2000, 0x2FFF);
address_range!(NAME_TABLE_0_RANGE, 0x2000, 0x23FF);
address_range!(NAME_TABLE_1_RANGE, 0x2400, 0x27FF);
address_range!(NAME_TABLE_2_RANGE, 0x2800, 0x2BFF);
address_range!(NAME_TABLE_3_RANGE, 0x2C00, 0x2FFF);
address_range!(NAME_TABLE_MIRROR_RANGE, 0x3000, 0x3EFF);
address_range!(PALLET_RAM_INDEX_RANGE, 0x3F00, 0x3F1F);
address_range!(PALLET_RAM_MIRROR_RANGE, 0x3F20, 0x3FFF);

pub const ONE_KB: u16 = 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chr_rom_range() {
        assert_eq!(CHR_ROM_RANGE.start_bound(), Bound::Included(&0x0000));
        assert_eq!(CHR_ROM_RANGE.end_bound(), Bound::Included(&0x1FFF));
    }

    #[test]
    fn test_contain_for_chr_rom_range() {
        assert_eq!(CHR_ROM_RANGE.contains(&CHR_ROM_RANGE.start), true);
        assert_eq!(CHR_ROM_RANGE.contains(&(CHR_ROM_RANGE.end / 2)), true); // CHR_ROM_RANGE.end/2 : 0x1FFF の半分
        assert_eq!(CHR_ROM_RANGE.contains(&CHR_ROM_RANGE.end), true);
        assert_eq!(CHR_ROM_RANGE.contains(&(CHR_ROM_RANGE.end + 1)), false);
    }
}
