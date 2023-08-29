pub struct ProcessorStatus {
    pub carry_flag: bool,
    pub zero_flag: bool,
    pub interrupt_disable: bool,
    pub decimal_mode_flag: bool, //Does not support
    pub break_command: bool,
    pub overflow_flag: bool,
    pub negative_flag: bool,
}

pub struct CPU {
    pub accumulator: u8,
    pub status: ProcessorStatus,
    pub program_counter: u16,
    pub index_register_x: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            status: ProcessorStatus {
                carry_flag: false,
                zero_flag: false,
                interrupt_disable: false,
                decimal_mode_flag: false,
                break_command: false,
                overflow_flag: false,
                negative_flag: false,
            },
            program_counter: 0,
            index_register_x: 0,
        }
    }

    fn lda(&mut self, value: u8) {
        self.accumulator = value;
        self.update_zero_and_negative_flags(self.accumulator)
    }

    fn tax(&mut self) {
        self.index_register_x = self.accumulator;
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn inx(&mut self) {
        if self.index_register_x == 0xff {
            self.index_register_x = 0;
        } else {
            self.index_register_x += 1;
        }
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.zero_flag = true;
        } else {
            self.status.zero_flag = false
        }
        if result & 0b1000_0000 != 0 {
            self.status.negative_flag = true
        } else {
            self.status.negative_flag = false
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opcode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opcode {
                0xa9 => {
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.lda(param)
                }
                0xaa => self.tax(),
                0xe8 => self.inx(),
                0x00 => return,
                _ => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0x00_brk() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0x00]);
        assert_eq!(
            cpu.program_counter, 1,
            "オペコードBRKが実行された際のプログラムカウンタが正しくありません"
        );
    }

    #[test]
    fn test_0x9a_lda_load_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.accumulator, 0x05);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.zero_flag == true);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0x00]);
        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.accumulator = 0x10;
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.index_register_x, 16);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status.zero_flag == true);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.accumulator = 0x80;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_0xe8_inx_increment_register_x() {
        let mut cpu = CPU::new();
        cpu.index_register_x = 0;
        cpu.interpret(vec![0xe8, 0x00]);
        assert_eq!(cpu.index_register_x, 1);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xe8_inx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.index_register_x = 0xff;
        cpu.interpret(vec![0xe8, 0x00]);
        assert_eq!(cpu.index_register_x, 0);
        assert!(cpu.status.zero_flag == true);
    }
    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.index_register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 1)
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.index_register_x = 0x80;
        cpu.interpret(vec![0xe8, 0x00]);
        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xc1)
    }
}
