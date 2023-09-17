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
    pub memory: [u8; 0x10000],
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

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
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

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
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

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let decremented_value = value.wrapping_sub(1);
        self.mem_write(addr, decremented_value);
        self.update_zero_and_negative_flags(decremented_value);
    }

    fn dex(&mut self) {
        self.index_register_x = self.index_register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn dey(&mut self) {
        self.index_register_y = self.index_register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.index_register_y)
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

    fn bit(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        self.status
            .set(ProcessorStatus::ZERO, self.accumulator & value as u8 != 0);
        self.status
            .set(ProcessorStatus::OVERFLOW, (value >> 5) << 1 != 0);
        self.status.set(ProcessorStatus::NEGATIVE, value >> 6 != 0);
    }

    fn status_bit(&self, reg: &ProcessorStatus) -> u8 {
        self.status.bits() & reg.bits()
    }

    fn compare(&mut self, mode: &AddressingMode, register: u8) {
        let value = self.fetch_data(mode);

        if register >= value {
            self.status.insert(ProcessorStatus::CARRY);
        }

        self.update_zero_and_negative_flags(register.wrapping_sub(value))
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
                DEC => self.dec(&opcode.mode),
                DEX => self.dex(),
                DEY => self.dey(),
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
                BNE | BEQ => self.branch(
                    &opcode.mode,
                    &ProcessorStatus::ZERO,
                    self.status.intersects(ProcessorStatus::ZERO),
                ),
                BVC | BVS => self.branch(
                    &opcode.mode,
                    &ProcessorStatus::OVERFLOW,
                    self.status.intersects(ProcessorStatus::OVERFLOW),
                ),
                BPL | BMI => self.branch(
                    &opcode.mode,
                    &ProcessorStatus::NEGATIVE,
                    self.status.intersects(ProcessorStatus::NEGATIVE),
                ),
                BIT => self.bit(&opcode.mode),
                CLC => self.status.remove(ProcessorStatus::CARRY),
                SEC => self.status.insert(ProcessorStatus::CARRY),
                CLI => self.status.remove(ProcessorStatus::INTERRUPT_DISABLE),
                SEI => self.status.insert(ProcessorStatus::INTERRUPT_DISABLE),
                CLD => self.status.remove(ProcessorStatus::DECIMAL),
                SED => self.status.insert(ProcessorStatus::DECIMAL),
                CLV => self.status.remove(ProcessorStatus::OVERFLOW),
                CMP => self.compare(&opcode.mode, self.accumulator),
                CPX => self.compare(&opcode.mode, self.index_register_x),
                CPY => self.compare(&opcode.mode, self.index_register_y),
                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }
}
