use crate::cpu::{AddressingMode, Mem, CPU};
use crate::opcodes;
use crate::opcodes::{OpCode, Operation};

pub fn trace(cpu: &CPU) -> String {
    let program_counter_str = format!("{:04X} ", cpu.program_counter);

    let ref opcodes_map = *opcodes::OPCODES_MAP;
    let current_code = cpu.mem_read(cpu.program_counter);
    let current_opcode = opcodes_map.get(&current_code).unwrap();

    let formatted_opcode_bytes = format_opcode(cpu, **current_opcode, cpu.program_counter);

    let formatted_opcode_str = formatted_opcode_bytes
        .iter()
        .map(|byte| format!("{:02X} ", byte))
        .collect::<String>();

    let asm_opcode = format_asm_opcode(
        cpu,
        current_opcode.mode,
        current_opcode.mnemonic,
        formatted_opcode_bytes,
    );

    let register_info = format_register(cpu);

    format!(
        "{:<6}{:<10}{:<32}{}",
        program_counter_str, formatted_opcode_str, asm_opcode, register_info
    )
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

fn format_opcode(cpu: &CPU, opcode: OpCode, program_counter: u16) -> Vec<u8> {
    let mut format_opcode: Vec<u8> = Vec::new();

    // let opcode_len = if opcode.len == 3 { 2 } else { opcode.len };

    // for i in 0..=(opcode_len - 1) {
    for i in 0..=(opcode.len - 1) {
        let element = cpu.mem_read(program_counter + i as u16);
        format_opcode.push(element)
    }

    format_opcode
}

fn format_asm_opcode(
    cpu: &CPU,
    mode: AddressingMode,
    mnemonic: Operation,
    opcode: Vec<u8>,
) -> String {
    match mode {
        AddressingMode::Immediate => format!("{:03?} #${:02X}", mnemonic, opcode[1]),
        AddressingMode::ZeroPage => format!(
            "{:03?} ${:02X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.mem_read(opcode[1] as u16)
        ),
        AddressingMode::ZeroPage_X => format!(
            "{:03?} ${:02X},X @ {:02X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.get_operand_address(&mode),
            cpu.mem_read(opcode[1] as u16)
        ),
        AddressingMode::Absolute => {
            format!("{:03?} ${:02X}{:02X}", mnemonic, opcode[2], opcode[1],)
        }
        AddressingMode::Absolute_X => format!(
            "{:03?} ${:04X},X @ {:04X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.get_operand_address(&mode),
            cpu.mem_read(cpu.get_operand_address(&mode))
        ),
        AddressingMode::Absolute_Y => format!(
            "{:03?} ${:04X},Y @ {:04X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.get_operand_address(&mode),
            cpu.mem_read(cpu.get_operand_address(&mode))
        ),
        AddressingMode::Indirect => format!(
            "{:03?} (${:04X}) = {:02X}",
            mnemonic,
            opcode[1],
            cpu.mem_read(cpu.get_operand_address(&mode))
        ),
        AddressingMode::Indirect_X => format!(
            "{:03?} (${:04X},X) @ {:04X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.get_operand_address(&mode),
            cpu.mem_read(cpu.get_operand_address(&mode))
        ),
        // deref_base,deref_addrはcpu.get_operand_address()のアドレス算出の途中計算で使用され外部から参照されないため、
        // ここでアドレスを算出する
        AddressingMode::Indirect_Y => {
            let base = opcode[1];
            let lo = cpu.mem_read(base as u16);
            let hi = cpu.mem_read((base).wrapping_add(1) as u16);
            let deref_base = (hi as u16) << 8 | (lo as u16);
            let deref_addr = deref_base.wrapping_add(cpu.index_register_y as u16);

            format!(
                "{:03?} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                mnemonic,
                opcode[1],
                deref_base,
                deref_addr,
                cpu.mem_read(deref_addr)
            )
        }
        AddressingMode::NoneAddressing => format!(""),
        AddressingMode::Relative => {
            format!("{:03?} ${:04X}", mnemonic, cpu.get_operand_address(&mode))
        }
        AddressingMode::Accumulator => format!(
            "{:03?} ${:02X} = {:02X}",
            mnemonic,
            opcode[1],
            cpu.mem_read(cpu.get_operand_address(&mode))
        ),
        AddressingMode::Implicit => format!("{:03?}", mnemonic),
        _ => panic!("Not implemented {:?}", mode,),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::ProcessorStatus;
    use crate::rom::test::test_rom;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom());
        bus.mem_write(100, 0xA9); //Immediate
        bus.mem_write(101, 0x10);
        bus.mem_write(102, 0xA5); //Zeropage
        bus.mem_write(103, 0x20);
        bus.mem_write(104, 0xB5); //Zeropage_X
        bus.mem_write(105, 0x30);
        bus.mem_write(106, 0xAD); //Absolute
        bus.mem_write(107, 0x40);
        bus.mem_write(108, 0xBD); //Absolute_X
        bus.mem_write(109, 0x50);
        bus.mem_write(110, 0x23);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.accumulator = 1;
        cpu.index_register_x = 2;
        cpu.index_register_y = 3;
        cpu.status = ProcessorStatus::from_bits_truncate(0x24);

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        //Immediate
        assert_eq!(
            "0064  A9 10     LDA #$10                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        // Zeropage
        assert_eq!(
            "0066  A5 20     LDA $20 = 00                    A:10 X:02 Y:03 P:24 SP:FD",
            result[1]
        );
        // Zeropage_X
        assert_eq!(
            "0068  B5 30     LDA $30,X @ B7 = 00             A:00 X:02 Y:03 P:26 SP:FD",
            result[2]
        );
        //
        // AD Absoulute  LDA $0678 = 00
        assert_eq!(
            "006A  AD 40     LDA $0040 = 00                  A:00 X:02 Y:03 P:26 SP:FD",
            result[3]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        //data
        bus.mem_write(0x33, 00);
        bus.mem_write(0x34, 04);

        //target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.index_register_y = 0;
        cpu.status = ProcessorStatus::from_bits_truncate(0x24);

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
