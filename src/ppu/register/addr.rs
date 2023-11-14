use super::constans::PPU_MEM_RANGE;

#[derive(Debug, PartialEq)]
pub struct AddrRegister {
    // value: (Hi, Lo),
    pub value: (u8, u8),
    pub hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0), // (hi, lo) high byte first
            hi_ptr: true,
        }
    }

    pub fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xFF) as u8;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.is_mirror_down() {
            self.mirror_down();
        }

        self.toggle_latch();
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;

        self.value.1 = self.value.1.wrapping_add(inc);

        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        if self.is_mirror_down() {
            self.mirror_down();
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn toggle_latch(&mut self) {
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn mirror_down(&mut self) {
        let value = self.get() & PPU_MEM_RANGE.end;
        self.set(value);
    }

    pub fn is_mirror_down(&self) -> bool {
        self.get() > PPU_MEM_RANGE.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let addr_reg = AddrRegister::new();
        assert_eq!(addr_reg.value, (0, 0));
        assert_eq!(addr_reg.hi_ptr, true);
    }

    #[test]
    fn test_set_get() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.set(0xABCD);
        assert_eq!(addr_reg.get(), 0xABCD);
    }

    #[test]
    fn test_update() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.update(0x12);
        assert_eq!(addr_reg.value, (0x12, 0));
        assert_eq!(addr_reg.hi_ptr, false);
        addr_reg.update(0x34);
        assert_eq!(addr_reg.value, (0x12, 0x34));
        assert_eq!(addr_reg.hi_ptr, true);
    }

    #[test]
    fn test_increment() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.set(0x1234);
        addr_reg.increment(1);
        assert_eq!(addr_reg.get(), 0x1235);
        addr_reg.increment(0xFF);
        assert_eq!(addr_reg.get(), 0x1334);
    }

    #[test]
    fn test_reset_latch() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.hi_ptr = false;
        addr_reg.reset_latch();
        assert_eq!(addr_reg.hi_ptr, true);
    }

    #[test]
    fn test_toggle_latch() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.toggle_latch();
        assert_eq!(addr_reg.hi_ptr, false);
        addr_reg.toggle_latch();
        assert_eq!(addr_reg.hi_ptr, true);
    }

    #[test]
    fn test_mirror_down() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.set(0x3FFF);
        addr_reg.mirror_down();
        assert_eq!(addr_reg.get(), 0x3FFF);
        addr_reg.set(0x4000);
        addr_reg.mirror_down();
        assert_eq!(addr_reg.get(), 0x0000);
    }

    #[test]
    fn test_is_mirror_down() {
        let mut addr_reg = AddrRegister::new();
        addr_reg.set(0x3FFF);
        assert_eq!(addr_reg.is_mirror_down(), false);
        addr_reg.set(0x4000);
        assert_eq!(addr_reg.is_mirror_down(), true);
    }
}
