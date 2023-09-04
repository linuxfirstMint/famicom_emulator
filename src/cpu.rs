pub struct ProcessorStatus {
    pub carry_flag: bool,
    pub zero_flag: bool,
    pub interrupt_disable: bool,
    pub decimal_mode_flag: bool, //Does not support
    pub break_command: bool,
    pub overflow_flag: bool,
    pub negative_flag: bool,
}

impl ProcessorStatus {
    pub fn clear() -> Self {
        ProcessorStatus {
            carry_flag: false,
            zero_flag: false,
            interrupt_disable: false,
            decimal_mode_flag: false,
            break_command: false,
            overflow_flag: false,
            negative_flag: false,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
}

pub struct CPU {
    pub accumulator: u8,
    pub status: ProcessorStatus,
    pub program_counter: u16,
    pub index_register_x: u8,
    memory: [u8; 0x10000],
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
            memory: [0; 0x10000],
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.index_register_x = 0;
        self.status = ProcessorStatus::clear();

        self.program_counter = self.mem_read_u16(0xfffc);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xfffc, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
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
        self.index_register_x = self.index_register_x.wrapping_add(1);
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

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                0xa9 => {
                    let param = self.mem_read(self.program_counter);
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
        cpu.load_and_run(vec![0x00]);
        assert_eq!(
            cpu.program_counter, 0x8001,
            "オペコードBRKが実行された際のプログラムカウンタが正しくありません"
        );
    }

    #[test]
    fn test_0x9a_lda_load_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.accumulator, 0x05);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status.zero_flag == true);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]);

        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x10, 0xaa, 0x00]);

        assert_eq!(cpu.index_register_x, 16);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xaa, 0x00]);

        assert!(cpu.status.zero_flag == true);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0x00]);

        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_0xe8_inx_increment_register_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 1);
        assert!(cpu.status.zero_flag == false);
        assert!(cpu.status.negative_flag == false);
    }

    #[test]
    fn test_0xe8_inx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 0);
        assert!(cpu.status.zero_flag == true);
    }
    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 1)
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0xe8, 0x00]);

        assert!(cpu.status.negative_flag == true);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xc1)
    }

    #[test]
    fn test_mem_read_write() {
        let mut cpu = CPU::new();

        cpu.mem_write(0x8000, 0xAB);
        cpu.mem_write(0x8001, 0xCD);

        let data1 = cpu.mem_read(0x8000);
        let data2 = cpu.mem_read(0x8001);

        assert_eq!(data1, 0xAB);
        assert_eq!(data2, 0xCD);
    }

    #[test]
    fn test_mem_read_write_u16() {
        let mut cpu = CPU::new();
        cpu.mem_write_u16(0x8000, 0xABCD);
        let value = cpu.mem_read_u16(0x8000);
        assert_eq!(value, 0xABCD)
    }

    #[test]
    fn test_load() {
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x01, 0x02, 0x03];
        cpu.load(program.clone());

        for (i, &byte) in program.iter().enumerate() {
            let memory_index = 0x8000 + i;
            assert!(
                memory_index < cpu.memory.len(),
                "Memory index out of range: 0x{:X}",
                memory_index
            );
            assert_eq!(cpu.memory[memory_index], byte);
        }
        assert_eq!(cpu.program_counter, 0);
    }

    #[test]
    fn test_reset() {
        let mut cpu = CPU::new();
        cpu.accumulator = 1;
        cpu.index_register_x = 1;
        cpu.status.negative_flag = true;
        cpu.reset();
        assert_eq!(cpu.accumulator, 0,);
        assert_eq!(cpu.index_register_x, 0,);
        assert_eq!(cpu.status.negative_flag, false);
    }

    #[test]
    fn test_get_operand_address_immediate() {
        let cpu = CPU::new();
        let mode = AddressingMode::Immediate;
        let result = cpu.get_operand_address(&mode);
        assert_eq!(result, cpu.program_counter);
    }

    #[test]
    fn test_get_operand_address_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[cpu.program_counter as usize] = 0x44;
        let mode = AddressingMode::ZeroPage;
        let result = cpu.get_operand_address(&mode);
        assert_eq!(result, 0x44);
    }
}
