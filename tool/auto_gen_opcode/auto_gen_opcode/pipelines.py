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
        file_path = output_dir / filename

        rust_code = file_format(item)

        with open(file_path, "w", encoding="utf-8") as file:
            file.write(rust_code)

        return item


def file_format(item):
    rust_code = "\n"
    rust_code += "pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![\n"

    for line in item["line"]:
        rust_code += f'\tOpCode::new({line["opcode"]}, "{line["name"]}", {line["bytes"]}, {line["cycles"]}, AddressingMode::{line["addressing_mode"]}),\n'

    rust_code += "];\n"

    return rust_code
