use core::panic;

use crate::cpu::Mem;
use crate::rom::Rom;

pub struct Bus {
    pub cpu_vram: [u8; 2048],
    pub rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Self {
            cpu_vram: [0; 2048],
            rom: rom,
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= PRG;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr -= 0x4000;
        }
        self.rom.prg_rom[addr as usize]
    }
}

const PRG: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;
const RAM: u16 = 0x0000;
const RAM_MIRROR_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRROR_END: u16 = 0x3FFF;

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            PRG..=PRG_ROM_END => self.read_prg_rom(addr),
            RAM..=RAM_MIRROR_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRROR_END => {
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                todo!("PPU is not implemented yet");
            }
            _ => {
                println!("Ignoring mem access at {:#X}", addr);
                0
            }
        }
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            PRG..=PRG_ROM_END => panic!("Attemt to write to Cartrige Rom Spase addr:{:X}", addr),
            RAM..=RAM_MIRROR_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRROR_END => {
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                todo!("PPU is not implemented yet")
            }

            _ => println!("Ignoring mem write-access at {:#X}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn test_rom() -> Rom {
        let game_dir = "roms";
        let game_file = "snake.nes";
        let game = fs::read(format!("{}/{}", game_dir, game_file)).unwrap();
        Rom::new(&game).unwrap()
    }

    #[test]
    fn test_mem_read_ram() {
        let bus = Bus::new(test_rom());
        let addr = 0x0000;
        let data = bus.mem_read(addr);
        assert_eq!(data, 0);
    }

    #[test]
    fn test_mem_read_ram_mirror() {
        let bus = Bus::new(test_rom());
        let addr = 0x0800;
        let data = bus.mem_read(addr);
        assert_eq!(data, 0);
    }

    // #[test]
    // fn test_mem_read_ppu_registers() {
    //     todo!("Implement this test");
    // }

    // #[test]
    // fn test_mem_read_ppu_registers_mirror() {
    //     todo!("Implement this test");
    // }

    #[test]
    fn test_mem_read_invalid_address() {
        let bus = Bus::new(test_rom());
        let addr = 0xFFFF;
        let data = bus.mem_read(addr);
        assert_eq!(data, 0);
    }

    #[test]
    fn test_mem_write_ram() {
        let mut bus = Bus::new(test_rom());
        let addr = 0x0000;
        let data = 0x42;
        bus.mem_write(addr, data);
        assert_eq!(bus.cpu_vram[0], data);
    }

    #[test]
    fn test_mem_write_ram_mirror() {
        let mut bus = Bus::new(test_rom());
        let addr = 0x0800;
        let data = 0x42;
        bus.mem_write(addr, data);
        assert_eq!(bus.cpu_vram[0], data);
    }

    #[test]
    #[should_panic]
    fn test_mem_write_invalid_address() {
        let mut bus = Bus::new(test_rom());
        let addr = 0xFFFF;
        let data = 0x42;
        bus.mem_write(addr, data)
    }
}
