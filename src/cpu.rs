use crate::opcodes::{self, Operation::*, OPCODES_MAP};
use bitflags::bitflags;
use std::collections::HashMap;

bitflags! {
    #[derive(Clone,Copy)]
    pub struct ProcessorStatus: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT_DISABLE = 1 << 2;
        const DECIMAL = 1 << 3;
        const BREAK = 1 << 4;
        // 1 << 5 (0b0010_0000) Unused bit
        const OVERFLOW = 1 << 6;
        const NEGATIVE = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
    Relative,
    Accumulator,
    Implicit,
}

pub struct CPU {
    pub accumulator: u8,
    pub status: ProcessorStatus,
    pub program_counter: u16,
    pub index_register_x: u8,
    pub index_register_y: u8,
    memory: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            status: ProcessorStatus::empty(),
            program_counter: 0,
            index_register_x: 0,
            index_register_y: 0,
            memory: [0; 0x10000],
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.index_register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.index_register_y) as u16;
                addr
            }

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.index_register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.index_register_y as u16);
                addr
            }

            AddressingMode::Indirect => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base).wrapping_add(1) as u16);
                let addr = (hi as u16) << 8 | (lo as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr = (base as u8).wrapping_add(self.index_register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                let addr = (hi as u16) << 8 | (lo as u16);
                addr
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref_addr = deref_base.wrapping_add(self.index_register_y as u16);
                deref_addr
            }

            AddressingMode::Relative => {
                let base = self.mem_read(self.program_counter) as i8;
                let addr = (self.program_counter as i16).wrapping_add(base as i16) as u16;

                addr
            }

            AddressingMode::Accumulator => self.accumulator as u16,

            AddressingMode::Implicit => 0 as u16,

            AddressingMode::NoneAddressing => panic!("mode {:?} is not supported", mode),
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
        self.index_register_y = 0;
        self.status = ProcessorStatus::empty();

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

    fn lda(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

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

    fn adc(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);
        let acc = self.accumulator;

        let (result, carry1) = value.overflowing_add(self.accumulator);
        let (final_result, carry2) =
            result.overflowing_add(self.status_bit(&ProcessorStatus::CARRY));

        self.accumulator = final_result;

        self.status.set(ProcessorStatus::CARRY, carry1 | carry2);
        self.status.set(
            ProcessorStatus::OVERFLOW,
            (acc ^ value) & 0x80 == 0 && (acc ^ final_result) & 0x80 != 0,
        );
        self.update_zero_and_negative_flags(self.accumulator);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);
        let acc = self.accumulator;

        let (result, carry1) = acc.overflowing_sub(value);
        let (final_result, carry2) =
            result.overflowing_sub(1 - (self.status_bit(&ProcessorStatus::CARRY)));

        self.accumulator = final_result;

        self.status.set(ProcessorStatus::CARRY, !(carry1 | carry2));
        self.status.set(
            ProcessorStatus::OVERFLOW,
            (acc ^ value) & 0x80 != 0 && (acc ^ final_result) & 0x80 != 0,
        );
        self.update_zero_and_negative_flags(self.accumulator);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);
        self.accumulator = self.accumulator & value;

        self.update_zero_and_negative_flags(self.accumulator);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);
        self.accumulator = self.accumulator ^ value;

        self.update_zero_and_negative_flags(self.accumulator);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);
        self.accumulator = self.accumulator | value;

        self.update_zero_and_negative_flags(self.accumulator);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        let (shifted_value, _) = value.overflowing_shl(1);

        self.update_zero_and_negative_flags(value);

        self.status
            .set(ProcessorStatus::CARRY, (value >> 7) & 1 > 0);

        match mode {
            AddressingMode::Accumulator => self.accumulator = shifted_value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, shifted_value)
            }
        }
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        let (shifted_value, _) = value.overflowing_shr(1);

        self.update_zero_and_negative_flags(value);

        self.status
            .set(ProcessorStatus::CARRY, (value >> 7) & 1 > 0);

        match mode {
            AddressingMode::Accumulator => self.accumulator = shifted_value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, shifted_value)
            }
        }
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        let carry_bit = (value >> 7) & 1;

        let (mut rotated_value, _) = value.overflowing_shl(1);

        rotated_value = rotated_value | carry_bit;

        self.update_zero_and_negative_flags(value);

        self.status
            .set(ProcessorStatus::CARRY, (value >> 7) & 1 > 0);

        match mode {
            AddressingMode::Accumulator => self.accumulator = rotated_value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, rotated_value)
            }
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        let carry_bit = value << 7;

        let (shifted_value, _) = value.overflowing_shr(1);

        let rotated_value = shifted_value | carry_bit;

        self.update_zero_and_negative_flags(value);

        self.status
            .set(ProcessorStatus::CARRY, (value >> 7) & 1 > 0);

        match mode {
            AddressingMode::Accumulator => self.accumulator = rotated_value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, rotated_value)
            }
        }
    }

    fn branch(&mut self, mode: &AddressingMode, status: &ProcessorStatus, condition: bool) {
        let addr = self.get_operand_address(mode) as i8;

        if self.status.contains(*status) == condition {
            self.program_counter = self.program_counter.wrapping_add_signed(addr.into())
        }
    }

    fn status_bit(&self, reg: &ProcessorStatus) -> u8 {
        self.status.bits() & reg.bits()
    }

    fn fetch_data(&self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        match mode {
            AddressingMode::Accumulator => return addr as u8,
            _ => self.mem_read(addr),
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.set(ProcessorStatus::ZERO, true);
        } else {
            self.status.set(ProcessorStatus::ZERO, false);
        }
        if result & 0b1000_0000 != 0 {
            self.status.set(ProcessorStatus::NEGATIVE, true);
        } else {
            self.status.set(ProcessorStatus::NEGATIVE, false);
        }
    }

    pub fn run(&mut self) {
        let ref opcodes = *opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);

            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode: {:x} is not found", code));

            match opcode.mnemonic {
                LDA => {
                    self.lda(&opcode.mode);
                }
                TAX => self.tax(),
                INX => self.inx(),
                ADC => self.adc(&opcode.mode),
                SBC => self.sbc(&opcode.mode),
                AND => self.and(&opcode.mode),
                EOR => self.eor(&opcode.mode),
                ORA => self.ora(&opcode.mode),
                ASL => self.asl(&opcode.mode),
                LSR => self.lsr(&opcode.mode),
                ROL => self.rol(&opcode.mode),
                ROR => self.ror(&opcode.mode),
                BRK => return,
                BCC | BCS => self.branch(
                    &opcode.mode,
                    &ProcessorStatus::CARRY,
                    self.status.intersects(ProcessorStatus::CARRY),
                ),
                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    mod opcode_tests {
        use super::*;

        mod lda {
            use super::*;

            #[test]
            fn test_lda_effects() {
                let mut cpu = CPU::new();

                cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
                assert_eq!(cpu.accumulator, 0x05);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::ZERO),
                    false
                );

                cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
                assert!(cpu.status.contains(ProcessorStatus::ZERO));

                cpu.load_and_run(vec![0xa9, 0x80, 0x00]);
                assert!(cpu.status.contains(ProcessorStatus::NEGATIVE));
            }

            #[test]
            fn test_lda_immediate() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xA9, 0x10, 0x00]);
                assert_eq!(cpu.accumulator, 0x10);
            }

            #[test]
            fn test_lda_zero_page() {
                let mut cpu = CPU::new();
                cpu.mem_write(0x10, 0x78);
                cpu.load_and_run(vec![0xA5, 0x10, 0x00]);
                assert_eq!(cpu.accumulator, 0x78);
            }

            #[test]
            fn test_lda_zero_page_x() {
                let mut cpu = CPU::new();
                cpu.mem_write(0x28, 0x07);
                cpu.load(vec![0xB5, 0x08, 0x00]);
                cpu.reset();
                cpu.index_register_x = 0x20;
                cpu.run();
                assert_eq!(cpu.accumulator, 0x07);
            }

            #[test]
            fn test_lda_absolute() {
                let mut cpu = CPU::new();
                cpu.mem_write(0x5228, 0xF0);
                cpu.load_and_run(vec![0xAD, 0x28, 0x52, 0x00]);
                assert_eq!(cpu.accumulator, 0xF0);
            }

            #[test]
            fn test_lda_absolute_x() {
                let mut cpu = CPU::new();
                cpu.mem_write(0xF0B9, 0x98);
                cpu.load(vec![0xBD, 0xA8, 0xF0, 0x00]);
                cpu.reset();
                cpu.index_register_x = 0x11;
                cpu.run();
                assert_eq!(cpu.accumulator, 0x98);
            }

            #[test]
            fn test_lda_absolute_y() {
                let mut cpu = CPU::new();
                cpu.mem_write(0x5A00, 0xEA);
                cpu.load(vec![0xB9, 0xB0, 0x59, 0x00]);
                cpu.reset();
                cpu.index_register_y = 0x50;
                cpu.run();
                assert_eq!(cpu.accumulator, 0xEA);
            }

            #[test]
            fn test_lda_indirect_x() {
                let mut cpu = CPU::new();
                cpu.mem_write_u16(0x85, 0x2030);
                cpu.mem_write(0x2030, 0xE1);
                cpu.load(vec![0xA1, 0x80, 0x00]);
                cpu.reset();
                cpu.index_register_x = 0x05;
                cpu.run();
                assert_eq!(cpu.accumulator, 0xE1);
            }

            #[test]
            fn test_lda_indirect_y() {
                let mut cpu = CPU::new();
                cpu.mem_write_u16(0x80, 0x2030);
                cpu.mem_write(0x2035, 0xE6);
                cpu.load(vec![0xB1, 0x80, 0x00]);
                cpu.reset();
                cpu.index_register_y = 0x05;
                cpu.run();
                assert_eq!(cpu.accumulator, 0xE6);
            }
        }
        mod tax {

            use super::*;

            #[test]
            fn test_tax_effects() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xa9, 0x10, 0xaa, 0x00]);

                assert_eq!(cpu.index_register_x, 16);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                    false
                );

                cpu.load_and_run(vec![0xa9, 0x00, 0xaa, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::ZERO), true);

                cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod imx {

            use super::*;

            #[test]
            fn test_inx_effects() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xe8, 0x00]);

                assert_eq!(cpu.index_register_x, 1);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                    false
                );

                cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);
                assert_eq!(cpu.index_register_x, 0);
                assert_eq!(cpu.status.contains(ProcessorStatus::ZERO), true);

                cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0xe8, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);

                cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
                assert_eq!(cpu.index_register_x, 1)
            }
        }
        mod brk {

            use super::*;

            #[test]
            fn test_brk_effects() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0x00]);
                assert_eq!(
                    cpu.program_counter, 0x8001,
                    "オペコードBRKが実行された際のプログラムカウンタが正しくありません"
                );
            }
        }
        mod adc {
            use super::*;

            mod effects {
                use super::*;

                #[test]
                fn test_adc_no_carry() {
                    //キャリーなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x02;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0x12);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }

                #[test]
                fn test_adc_has_carry() {
                    // 計算前にキャリーあり

                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x01;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x12);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }
                #[test]
                fn test_adc_occur_carry() {
                    // 計算中にキャリー発生

                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x01, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0xFF;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::OVERFLOW | ProcessorStatus::NEGATIVE),
                        false
                    );
                }

                #[test]
                fn test_adc_occur_overflow_plus() {
                    //キャリーとオーバーフローが発生し計算結果がプラスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x01, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x80);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_minus() {
                    //キャリーとオーバーフローが発生し計算結果がマイナスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x81;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x2);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_minus_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生して計算結果がプラスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x6F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x10;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x80);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_plus_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生して計算結果がマイナスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x81;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x03);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_no_overflow() {
                    //オーバーフローなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x82;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0x01);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }
            }
        }
        mod sbc {
            use super::*;
            mod effects {
                use super::*;

                #[test]
                fn test_sbc_no_carry() {
                    //キャリーなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x20;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0F);
                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::ZERO
                                | ProcessorStatus::NEGATIVE
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }

                #[test]
                fn test_sbc_has_carry() {
                    // 計算前にキャリーあり

                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x20;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x10);
                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::ZERO
                                | ProcessorStatus::NEGATIVE
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }
                #[test]
                fn test_sbc_occur_carry() {
                    // 計算中にキャリー発生

                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x02, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x01;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0xFE);
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }

                #[test]
                fn test_sbc_occur_overflow() {
                    //オーバーフローが発生
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0xFD);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::OVERFLOW | ProcessorStatus::NEGATIVE),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        false
                    );
                }

                #[test]
                fn test_sbc_occur_overflow_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        false
                    )
                }

                #[test]
                fn test_sbc_no_overflow() {
                    //オーバーフローなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7E;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0xFE);
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }
            }
        }
        mod and {
            use super::*;

            #[test]
            fn test_and() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x29, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0x60);
                assert_eq!(cpu.status.is_empty(), true);
            }
        }
        mod eor {
            use super::*;

            #[test]
            fn test_eor() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x49, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0x9E);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod ora {
            use super::*;

            #[test]
            fn test_ora() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x09, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0xFE);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod asl {
            use super::*;

            #[test]
            fn test_asl_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x0A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b11101010;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b11010100);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_asl_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01101010);
                cpu.load(vec![0x06, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b11010100);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod lsr {
            use super::*;

            #[test]
            fn test_lsr_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x4A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b11101010;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b01110101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_lsr_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01101010);
                cpu.load(vec![0x46, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b00110101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod rol {
            use super::*;

            #[test]
            fn test_rol_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x2A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b10101011;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b01010111);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_rol_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01100101);
                cpu.load(vec![0x26, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b11001010);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod ror {
            use super::*;

            #[test]
            fn test_ror_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x6A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b10101011;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b11010101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_ror_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01100101);
                cpu.load(vec![0x66, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b10110010);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod branch {
            use super::*;
            mod bcc {
                use super::*;
                #[test]
                fn test_bcc() {
                    let mut cpu = CPU::new();
                    cpu.load_and_run(vec![0x90, 0x02, 0x00, 0x00, 0x00]);
                    assert_eq!(cpu.program_counter, 0x8005);
                }
            }
            mod bcs {
                use super::*;

                #[test]
                fn test_bcs() {
                    let mut cpu = CPU::new();
                    cpu.load(vec![0xB0, 0x02, 0x00, 0x00, 0x00]);
                    cpu.reset();
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.program_counter, 0x8005);
                }
            }
        }
    }
        mod operand_address_tests {

            use super::*;

            #[test]
            fn test_get_operand_address() {
                let mut cpu = CPU::new();
                cpu.program_counter = 0x90;
                let mut mode = AddressingMode::Immediate;
                let mut effective_address = cpu.get_operand_address(&mode);
                assert_eq!(
                    effective_address, cpu.program_counter,
                    "オペランドアドレスがプログラムカウンタと一致していません"
                );

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x44;
                mode = AddressingMode::ZeroPage;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x44);

                cpu.reset();
                mode = AddressingMode::ZeroPage;
                for address in 0x00..=0xFF {
                    cpu.memory[cpu.program_counter as usize] = address;
                    effective_address = cpu.get_operand_address(&mode);
                    assert_eq!(effective_address, address as u16);
                }

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x44;
                cpu.index_register_x = 0x10;
                mode = AddressingMode::ZeroPage_X;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x54);

                cpu.reset();
                cpu.index_register_y = 0x02;
                cpu.memory[cpu.program_counter as usize] = 0x50;
                mode = AddressingMode::ZeroPage_Y;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x52);

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x80;
                cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0x49;
                mode = AddressingMode::Absolute;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x4980);

                cpu.reset();
                cpu.index_register_x = 0x20;
                cpu.memory[cpu.program_counter as usize] = 0x30;
                cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0x98;
                mode = AddressingMode::Absolute_X;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x9850);

                cpu.reset();
                cpu.index_register_y = 0x42;
                cpu.memory[cpu.program_counter as usize] = 0x50;
                cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0xE0;
                mode = AddressingMode::Absolute_Y;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0xE092);

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x22;
                cpu.memory[0x22] = 0x50;
                cpu.memory[0x23] = 0xAC;
                mode = AddressingMode::Indirect;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0xAC50);

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x40;
                cpu.index_register_x = 0x05;
                cpu.memory[0x45] = 0x10;
                cpu.memory[0x46] = 0x09;
                mode = AddressingMode::Indirect_X;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x0910);

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0xA0;
                cpu.index_register_y = 0x05;
                cpu.memory[0xA0] = 0x50;
                cpu.memory[0xA1] = 0xB2;
                mode = AddressingMode::Indirect_Y;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0xB255);

                cpu.reset();
                cpu.memory[cpu.program_counter as usize] = 0x60;
                mode = AddressingMode::Relative;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x60);

                cpu.reset();
                cpu.accumulator = 0x42;
                mode = AddressingMode::Accumulator;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0x42);

                cpu.reset();
                mode = AddressingMode::Implicit;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, 0);
            }
        }

        mod memory_access {

            use super::*;

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
        }

        mod cpu_instruction_tests {

            use super::*;

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
                cpu.status.insert(ProcessorStatus::NEGATIVE);
                cpu.reset();
                assert_eq!(cpu.accumulator, 0);
                assert_eq!(cpu.index_register_x, 0);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }

            #[test]
            fn test_5_ops_working_together() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

                assert_eq!(cpu.index_register_x, 0xc1)
            }
        }
    }
}
