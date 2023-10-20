use crate::cpu::{AddressingMode, Mem, CPU};
use crate::opcodes;
use crate::opcodes::{OpCode, Operation};

pub fn trace(cpu: &CPU) -> String {
    let formatted_program_counter = format!("{:04X}", cpu.program_counter);

    let opcodes_map = &opcodes::OPCODES_MAP;
    let current_code = cpu.mem_read(cpu.program_counter);
    let current_opcode = opcodes_map.get(&current_code).expect(
        format!(
            "Invalid opcode pc: {:#X} code: {:#X} ",
            cpu.program_counter, current_code,
        )
        .as_str(),
    );

    let instruction = fetch_instruction_bytes(cpu, **current_opcode, cpu.program_counter);

    let formatted_instruction = format_instruction(instruction.clone());

    let asm_opcode = format_asm_opcode(
        cpu,
        cpu.program_counter + 1, // 1byte目はopcodeなので、2byte目からアドレスを取得する
        current_opcode.mode,
        current_opcode.mnemonic,
        instruction[1..].to_vec(),
    );

    let register_info = format_register(cpu);

    format!(
        "{:<6}{:<10}{:<32}{}",
        formatted_program_counter, formatted_instruction, asm_opcode, register_info
    )
}

fn fetch_instruction_bytes(cpu: &CPU, opcode: OpCode, program_counter: u16) -> Vec<u8> {
    let mut instruction_bytes: Vec<u8> = Vec::new();

    // instrcutionの先頭の1byteを除いた長さ
    for i in 0..=(opcode.len - 1) {
        let byte = cpu.mem_read(program_counter + i as u16);
        instruction_bytes.push(byte)
    }

    instruction_bytes
}

fn format_instruction(instruction_bytes: Vec<u8>) -> String {
    instruction_bytes
        .iter()
        .map(|byte| format!("{:02X} ", byte))
        .collect::<String>()
}

fn format_register(cpu: &CPU) -> String {
    String::from(format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.accumulator,
        cpu.index_register_x,
        cpu.index_register_y,
        cpu.status.bits(),
        cpu.stack_pointer,
    ))
}

fn format_asm_opcode(
    cpu: &CPU,
    program_counter: u16,
    mode: AddressingMode,
    mnemonic: Operation,
    operand_bytes: Vec<u8>,
) -> String {
    match mode {
        AddressingMode::Immediate => format_imm_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::ZeroPage => format_zero_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::ZeroPage_X => format_zero_x_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::ZeroPage_Y => format_zero_y_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::Absolute => format_absolute_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::Absolute_X => format_absolute_x_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::Absolute_Y => format_absolute_y_mode_asm(cpu, program_counter, mnemonic),
        AddressingMode::Indirect => {
            format_indirect_mode_asm(cpu, program_counter, mnemonic, operand_bytes, mode)
        }
        AddressingMode::Indirect_X => format_indirect_x_mode_asm(cpu, mnemonic, operand_bytes),
        AddressingMode::Indirect_Y => format_indirect_y_mode_asm(cpu, mnemonic, operand_bytes),
        AddressingMode::NoneAddressing => format!(""),
        AddressingMode::Relative => format_relative_mode_asm(cpu, mnemonic, program_counter),
        AddressingMode::Accumulator => format!("{:03?} A", mnemonic),
        AddressingMode::Implicit => format!("{:03?}", mnemonic),
    }
}

fn format_imm_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let memory_value = cpu.mem_read(program_counter);
    format!("{:03?} #${:02X}", mnemonic, memory_value)
}

fn format_zero_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let target_addr = cpu.mem_read(program_counter) as u16;
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} ${:02X} = {:02X}",
        mnemonic, target_addr, memory_value
    )
}

fn format_zero_x_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let addr = cpu.mem_read(program_counter);
    let target_addr = addr.wrapping_add(cpu.index_register_x) as u16;
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} ${:02X},X @ {:02X} = {:02X}",
        mnemonic, addr, target_addr, memory_value
    )
}

fn format_zero_y_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let addr = cpu.mem_read(program_counter);
    let target_addr = addr.wrapping_add(cpu.index_register_y) as u16;
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} ${:02X},Y @ {:02X} = {:02X}",
        mnemonic, addr, target_addr, memory_value
    )
}

fn format_absolute_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let target_addr = cpu.mem_read_u16(program_counter);

    match mnemonic {
        Operation::JMP | Operation::JSR => {
            format!("{:03?} ${:04X}", mnemonic, target_addr)
        }
        _ => {
            let memory_value = cpu.mem_read(target_addr);
            format!(
                "{:03?} ${:04X} = {:02X}",
                mnemonic, target_addr, memory_value
            )
        }
    }
}

fn format_absolute_x_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let addr = cpu.mem_read_u16(program_counter);
    let target_addr = addr.wrapping_add(cpu.index_register_x as u16);
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} ${:04X},X @ {:04X} = {:02X}",
        mnemonic, addr, target_addr, memory_value
    )
}

fn format_absolute_y_mode_asm(cpu: &CPU, program_counter: u16, mnemonic: Operation) -> String {
    let addr = cpu.mem_read_u16(program_counter);
    let target_addr = addr.wrapping_add(cpu.index_register_y as u16);
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} ${:04X},Y @ {:04X} = {:02X}",
        mnemonic, addr, target_addr, memory_value
    )
}

const LOW_PAGE_END: u16 = 0x00FF;
const HIGH_PAGE_START: u16 = 0xFF00;

fn format_indirect_mode_asm(
    cpu: &CPU,
    program_counter: u16,
    mnemonic: Operation,
    operand_bytes: Vec<u8>,
    mode: AddressingMode,
) -> String {
    match mnemonic {
        Operation::JMP => {
            let addr = cpu.mem_read_u16(program_counter);

            let target_addr = if addr & LOW_PAGE_END == LOW_PAGE_END {
                let lo = cpu.mem_read(addr);
                let hi = cpu.mem_read(addr & HIGH_PAGE_START);
                (hi as u16) << 8 | (lo as u16)
            } else {
                cpu.mem_read_u16(addr)
            };

            format!("{:03?} (${:04X}) = {:04X}", mnemonic, addr, target_addr)
        }
        _ => {
            format!(
                "{:03?} (${:04X}) = {:02X}",
                mnemonic,
                operand_bytes[0],
                cpu.mem_read(cpu.get_operand_address(&mode))
            )
        }
    }
}

fn format_indirect_x_mode_asm(cpu: &CPU, mnemonic: Operation, operand_bytes: Vec<u8>) -> String {
    let addr = (operand_bytes[0] as u8).wrapping_add(cpu.index_register_x);
    let lo = cpu.mem_read(addr as u16);
    let hi = cpu.mem_read(addr.wrapping_add(1) as u16);
    let target_addr = (hi as u16) << 8 | (lo as u16);
    let memory_value = cpu.mem_read(target_addr);

    format!(
        "{:03?} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
        mnemonic, operand_bytes[0], addr, target_addr, memory_value
    )
}

fn format_indirect_y_mode_asm(cpu: &CPU, mnemonic: Operation, operand_bytes: Vec<u8>) -> String {
    let base = operand_bytes[0];
    let lo = cpu.mem_read(base as u16);
    let hi = cpu.mem_read((base).wrapping_add(1) as u16);
    let deref_base = (hi as u16) << 8 | (lo as u16);
    let deref_addr = deref_base.wrapping_add(cpu.index_register_y as u16);
    let memory_value = cpu.mem_read(deref_addr);

    format!(
        "{:03?} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
        mnemonic, operand_bytes[0], deref_base, deref_addr, memory_value
    )
}

fn format_relative_mode_asm(cpu: &CPU, mnemonic: Operation, program_counter: u16) -> String {
    let base = cpu.mem_read(program_counter) as i8;
    let addr = (program_counter as i16).wrapping_add(base as i16) as u16;
    let target_addr = addr + 1;

    format!("{:03?} ${:04X}", mnemonic, target_addr)
}
