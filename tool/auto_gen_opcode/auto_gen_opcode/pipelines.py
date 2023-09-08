# Define your item pipelines here
#
# Don't forget to add your pipeline to the ITEM_PIPELINES setting
# See: https://docs.scrapy.org/en/latest/topics/item-pipeline.html


# useful for handling different item types with a single interface

from pathlib import Path


class AutoGenOpcodePipeline:
    def process_item(self, item, spider):
        current_dir = Path.cwd()
        project_dir = current_dir.parent.parent.parent.parent
        output_dir = project_dir / "src"
        filename = "opcodes.rs"
        # filename = "opcodes_test.rs"
        file_path = output_dir / filename

        rust_code = file_format(item)

        with open(file_path, "w", encoding="utf-8") as file:
            file.write(rust_code)

        return item


def file_format(item):
    rust_code = "use crate::cpu::AddressingMode;\n"
    rust_code += "use std::collections::HashMap;\n"
    rust_code += "use Operation::*;\n"
    rust_code += """

#[rustfmt::skip]
#[derive(Clone, Copy)]
pub enum Operation {
    ADC,AND,ASL,BCC,BCS,BEQ,BIT,BMI,BNE,BPL,BRK,BVC,BVS,CLC,
    CLD,CLI,CLV,CMP,CPX,CPY,DEC,DEX,DEY,EOR,INC,INX,INY,JMP,
    JSR,LDA,LDX,LDY,LSR,NOP,ORA,PHA,PHP,PLA,PLP,ROL,ROR,RTI,
    RTS,SBC,SEC,SED,SEI,STA,STX,STY,TAX,TAY,TSX,TXA,TXS,TYA,
}

pub struct OpCode {
    pub code: u8,
    pub mnemonic: Operation,
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, mnemonic: Operation, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic,
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}
"""

    rust_code += "\n"
    rust_code += "lazy_static! {"
    rust_code += "\n"
    rust_code += "\tpub static ref CPU_OPS_CODES: Vec<OpCode> = vec![\n"

    for line in item["line"]:
        rust_code += f'\t\tOpCode::new({line["opcode"]}, {line["name"]}, {line["bytes"]}, {line["cycles"]}, AddressingMode::{line["addressing_mode"]}),\n'

    rust_code += "\t];\n"

    rust_code += """
        pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
            let mut map = HashMap::new();
            for cpuop in &*CPU_OPS_CODES {
                map.insert(cpuop.code, cpuop);
            }
            map
        };
"""

    rust_code += "}\n"

    return rust_code
