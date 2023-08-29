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
                    self.accumulator = param;

                    if self.accumulator == 0 {
                        self.status.zero_flag = true
                    } else {
                        self.status.zero_flag = false
                    }

                    if self.accumulator & 0b1000_0000 != 0 {
                        self.status.negative_flag = true
                    } else {
                        self.status.negative_flag = false
                    }
                }
                0x00 => {
                    return;
                }
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
}