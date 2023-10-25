use crate::opcodes::{self, Operation::*, OPCODES_MAP};
use bitflags::bitflags;
extern crate log;
use crate::bus::Bus;

bitflags! {
    #[derive(Clone,Copy)]
    pub struct ProcessorStatus: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT_DISABLE = 1 << 2;
        const DECIMAL = 1 << 3;
        const BREAK = 1 << 4;
        const BREAK_2 = 1 << 5;
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

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

pub struct CPU {
    pub accumulator: u8,
    pub status: ProcessorStatus,
    pub program_counter: u16,
    pub index_register_x: u8,
    pub index_register_y: u8,
    pub stack_pointer: u8,
    pub bus: Bus,
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            accumulator: 0,
            status: ProcessorStatus::empty(),
            program_counter: 0,
            index_register_x: 0,
            index_register_y: 0,
            stack_pointer: 0xFD,
            bus: bus,
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
                let base = self.mem_read_u16(self.program_counter);
                let addr = self.mem_read_u16(base);
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

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.stack_pointer = 0xfd;
        self.status = ProcessorStatus::from_bits_truncate(0x24);

        self.program_counter = self.mem_read_u16(0xfffc);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.bus.cpu_vram[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
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

    fn ldx(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        self.index_register_x = value;
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        self.index_register_y = value;
        self.update_zero_and_negative_flags(self.index_register_y)
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.accumulator);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.index_register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.index_register_y);
    }

    fn tax(&mut self) {
        self.index_register_x = self.accumulator;
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn tsx(&mut self) {
        self.index_register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn tay(&mut self) {
        self.index_register_y = self.accumulator;
        self.update_zero_and_negative_flags(self.index_register_y)
    }

    fn tya(&mut self) {
        self.accumulator = self.index_register_y;
        self.update_zero_and_negative_flags(self.accumulator)
    }

    fn txa(&mut self) {
        self.accumulator = self.index_register_x;
        self.update_zero_and_negative_flags(self.accumulator)
    }

    fn txs(&mut self) {
        self.stack_pointer = self.index_register_x;
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let incremented_value = value.wrapping_add(1);
        self.mem_write(addr, incremented_value);
        self.update_zero_and_negative_flags(incremented_value);
    }

    fn inx(&mut self) {
        self.index_register_x = self.index_register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.index_register_x)
    }

    fn iny(&mut self) {
        self.index_register_y = self.index_register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.index_register_y)
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
        let mut value = self.fetch_data(mode);

        if value >> 7 == 1 {
            self.status.insert(ProcessorStatus::CARRY);
        } else {
            self.status.remove(ProcessorStatus::CARRY);
        }

        value = value << 1;

        match mode {
            AddressingMode::Accumulator => self.accumulator = value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, value);
            }
        }

        self.update_zero_and_negative_flags(value);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let value = match mode {
            AddressingMode::Accumulator => self.accumulator,
            _ => self.fetch_data(mode),
        };

        self.status.set(ProcessorStatus::CARRY, value & 0x01 == 1);

        let negative_flag = value >> 1;
        self.update_zero_and_negative_flags(negative_flag);

        match mode {
            AddressingMode::Accumulator => self.accumulator = negative_flag,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, negative_flag)
            }
        }
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let mut value = self.fetch_data(mode);

        let old_carry = self.status.contains(ProcessorStatus::CARRY);

        if value >> 7 == 1 {
            self.status.insert(ProcessorStatus::CARRY);
        } else {
            self.status.remove(ProcessorStatus::CARRY);
        }

        value = value << 1;

        if old_carry {
            value = value | 1;
        }

        match mode {
            AddressingMode::Accumulator => {
                self.accumulator = value;
            }
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, value)
            }
        }

        self.update_negative_flags(value);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let mut value = self.fetch_data(mode);
        let old_carry = self.status.contains(ProcessorStatus::CARRY);

        if value & 1 == 1 {
            self.status.insert(ProcessorStatus::CARRY);
        } else {
            self.status.remove(ProcessorStatus::CARRY);
        }
        value = value >> 1;
        if old_carry {
            value = value | 0b10000000;
        }

        match mode {
            AddressingMode::Accumulator => {
                self.accumulator = value;
            }
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, value)
            }
        }

        self.update_negative_flags(value);
    }

    fn update_negative_flags(&mut self, value: u8) {
        if value & 0b1000_0000 != 0 {
            self.status.set(ProcessorStatus::NEGATIVE, true);
        } else {
            self.status.set(ProcessorStatus::NEGATIVE, false);
        }
    }

    fn branch(&mut self, status: &ProcessorStatus, condition: bool) {
        let jmp = self.mem_read(self.program_counter) as i8;
        if condition {
            if self.status.bits() & status.bits() != 0 {
                self.program_counter = self
                    .program_counter
                    .wrapping_add(1)
                    .wrapping_add(jmp as u16);
            }
        } else {
            if self.status.bits() & status.bits() == 0 {
                self.program_counter = self
                    .program_counter
                    .wrapping_add(1)
                    .wrapping_add(jmp as u16);
            }
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let value = self.fetch_data(mode);

        let zero = self.accumulator & value;
        if zero == 0 {
            self.status.insert(ProcessorStatus::ZERO)
        } else {
            self.status.remove(ProcessorStatus::ZERO)
        }

        self.status
            .set(ProcessorStatus::NEGATIVE, value & 0b1000_0000 > 0);
        self.status
            .set(ProcessorStatus::OVERFLOW, value & 0b0100_0000 > 0);
    }

    fn status_bit(&self, reg: &ProcessorStatus) -> u8 {
        self.status.bits() & reg.bits()
    }

    fn compare(&mut self, mode: &AddressingMode, register: u8) {
        let value = self.fetch_data(mode);

        if register >= value {
            self.status.insert(ProcessorStatus::CARRY);
        } else {
            self.status.remove(ProcessorStatus::CARRY);
        }

        self.update_zero_and_negative_flags(register.wrapping_sub(value))
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.accumulator)
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.index_register_x)
    }
    fn cpy(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.index_register_y)
    }

    fn pha(&mut self) {
        self.push(self.accumulator);
    }

    fn php(&mut self) {
        let mut status = self.status.clone();
        status.insert(ProcessorStatus::BREAK | ProcessorStatus::BREAK_2);
        self.push(status.bits());
    }

    fn plp(&mut self) {
        self.status = ProcessorStatus::from_bits_truncate(self.pop());
        self.status.remove(ProcessorStatus::BREAK);
        self.status.insert(ProcessorStatus::BREAK_2);
    }

    fn pla(&mut self) {
        self.accumulator = self.pop();
        self.update_zero_and_negative_flags(self.accumulator)
    }

    fn push(&mut self, data: u8) {
        let addr = self.get_stack_addr();
        self.mem_write(addr, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.push(hi);
        self.push(lo);
    }

    fn pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let addr = self.get_stack_addr();
        let data = self.mem_read(addr);
        data
    }

    fn pop_u16(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        (hi << 8) | lo
    }

    fn get_stack_addr(&self) -> u16 {
        0x0100 as u16 + (self.stack_pointer as u16)
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::Indirect => self.jmp_indirect(),
            _ => {
                let addr = self.get_operand_address(mode);
                self.program_counter = addr;
            }
        }
    }

    fn jmp_indirect(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);

        let indirect_ref = if mem_address & 0x00FF == 0x00FF {
            let lo = self.mem_read(mem_address);
            let hi = self.mem_read(mem_address & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.mem_read_u16(mem_address)
        };

        self.program_counter = indirect_ref;
    }

    fn jsr(&mut self, mode: &AddressingMode, opcode_len: &u8) {
        let addr = self.get_operand_address(mode);
        let operand_byte = *opcode_len as u16 - 1; //jsrのオペランド部のバイト数
        let operand_byte_count = operand_byte - 1;
        let return_addr = self.program_counter + operand_byte_count;
        self.push_u16(return_addr);
        self.program_counter = addr;
    }

    fn rts(&mut self) {
        let return_addr = self.pop_u16() + 1;
        self.program_counter = return_addr;
    }

    fn brk(&mut self) {
        // TODO 割り込み処理の実装後に実装
        // self.status.insert(ProcessorStatus::BREAK);
        // self.push_u16(self.program_counter + 1);
        // self.php();
        // self.status.insert(ProcessorStatus::INTERRUPT_DISABLE);
        // self.program_counter = self.mem_read_u16(0xFFFE);
    }

    fn rti(&mut self) {
        self.plp();
        self.program_counter = self.pop_u16();
        return;
    }

    // TODO 割り込み処理の実装後に実装
    // fn irq(&mut self) {
    //     if !self.status.contains(ProcessorStatus::INTERRUPT_DISABLE) {
    //         self.push_u16(self.program_counter);
    //         self.php();
    //         self.status
    //             .remove(ProcessorStatus::INTERRUPT_DISABLE | ProcessorStatus::BREAK);
    //         self.program_counter = self.mem_read_u16(0xFFFE);
    //         self.rti()
    //     }
    // }

    fn rax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax();
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.accumulator & self.index_register_x;
        self.mem_write(addr, value);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
    }

    fn isb(&mut self, mode: &AddressingMode) {
        self.inc(mode);
        self.sbc(mode);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        self.ror(mode);
        self.adc(mode);
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
        self.run_with_callback(|_| {})
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes = *opcodes::OPCODES_MAP;

        loop {
            callback(self);
            let code = self.mem_read(self.program_counter);

            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode: {:x} is not found", code));
            match opcode.mnemonic {
                LDA => self.lda(&opcode.mode),
                LDX => self.ldx(&opcode.mode),
                LDY => self.ldy(&opcode.mode),
                STA => self.sta(&opcode.mode),
                STX => self.stx(&opcode.mode),
                STY => self.sty(&opcode.mode),
                TAX => self.tax(),
                TAY => self.tay(),
                TYA => self.tya(),
                TXA => self.txa(),
                TSX => self.tsx(),
                TXS => self.txs(),
                INC => self.inc(&opcode.mode),
                INX => self.inx(),
                INY => self.iny(),
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
                // BRK => self.brk(),
                RTI => self.rti(),
                BCC => self.branch(&ProcessorStatus::CARRY, false),
                BCS => self.branch(&ProcessorStatus::CARRY, true),
                BVC => self.branch(&ProcessorStatus::OVERFLOW, false),
                BVS => self.branch(&ProcessorStatus::OVERFLOW, true),
                BPL => self.branch(&ProcessorStatus::NEGATIVE, false),
                BMI => self.branch(&ProcessorStatus::NEGATIVE, true),
                BNE => self.branch(&ProcessorStatus::ZERO, false),
                BEQ => self.branch(&ProcessorStatus::ZERO, true),
                BIT => self.bit(&opcode.mode),
                CLC => self.status.remove(ProcessorStatus::CARRY),
                SEC => self.status.insert(ProcessorStatus::CARRY),
                CLI => self.status.remove(ProcessorStatus::INTERRUPT_DISABLE),
                SEI => self.status.insert(ProcessorStatus::INTERRUPT_DISABLE),
                CLD => self.status.remove(ProcessorStatus::DECIMAL),
                SED => self.status.insert(ProcessorStatus::DECIMAL),
                CLV => self.status.remove(ProcessorStatus::OVERFLOW),
                CMP => self.cmp(&opcode.mode),
                CPX => self.cpx(&opcode.mode),
                CPY => self.cpy(&opcode.mode),
                NOP => {}
                PHA => self.pha(),
                PLA => self.pla(),
                PHP => self.php(),
                PLP => self.plp(),
                JMP => self.jmp(&opcode.mode),
                JSR => self.jsr(&opcode.mode, &opcode.len),
                RTS => self.rts(),
                LAX => self.rax(&opcode.mode),
                SAX => self.sax(&opcode.mode),
                DCP => self.dcp(&opcode.mode),
                ISB => self.isb(&opcode.mode),
                SLO => self.slo(&opcode.mode),
                RLA => self.rla(&opcode.mode),
                SRE => self.sre(&opcode.mode),
                RRA => self.rra(&opcode.mode),
                _ => todo!("mnemonic: {:?} ", opcode.mnemonic),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }
}
