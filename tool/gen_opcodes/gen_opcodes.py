from pathlib import Path

import scraping
from tqdm import tqdm


def main():
    """
    Generate Rust code for the opcodes of a Famicom emulator.

    This function scrapes the opcode information, formats it into Rust code,
    and writes it to a file at /workspaces/famicom_emulator/src/opcodes.rs.

    Returns:
        None
    """

    opcodes = scraping.full_opcode_info()

    if not opcodes:
        return

    rust_code = generate_rust_code(opcodes)

    output_dir = Path.cwd() / "src"
    output_path = output_dir / "opcodes.rs"

    # Create a tqdm progress bar
    with tqdm(
        total=len(rust_code),
        unit="char",
        unit_scale=True,
        desc="Writing  opcode info",
    ) as pbar:
        with open(output_path, "w", encoding="utf-8") as file:
            for char in rust_code:
                file.write(char)
                pbar.update(1)

    print(f"\n[DONE] output {output_path}")


def generate_rust_code(lines: list[dict[str, str]]) -> str:
    """
    Generate Rust code for the opcodes of a Famicom emulator.

    Args:
        `lines`: A list of dictionaries containing opcode information.

    Returns:
        `str`: A string containing the Rust code for the opcodes.
    """
    rust_code = (
        "use crate::cpu::AddressingMode;\n"
        "use std::collections::HashMap;\n"
        "use Operation::*;\n\n"
        "#[rustfmt::skip]\n"
        "#[derive(Debug, Clone, Copy, PartialEq)]\n"
        "pub enum Operation {"
    )

    # Get unique opcode names
    names = [line["name"].replace("*", "") for line in lines]
    unique_names = list(dict.fromkeys(names))

    with tqdm(
        total=len(unique_names),
        unit="opcode",
        unit_scale=True,
        desc="Generate Rust format",
    ) as pbar:
        # Add opcode names to Rust code
        for i, unique_name in enumerate(unique_names):
            if i % 14 == 0:
                rust_code += "\n\t"
            rust_code += f"{unique_name}"
            if i != len(unique_names) - 1:
                rust_code += ","
            pbar.update(1)
    rust_code += "\n}\n\n"

    rust_code += (
        "#[derive(Debug, Clone, Copy)]\n"
        "pub struct OpCode {\n"
        "\tpub code: u8,\n"
        "\tpub mnemonic: Operation,\n"
        "\tpub len: u8,\n"
        "\tpub cycles: u8,\n"
        "\tpub mode: AddressingMode,\n"
        "}\n\n"
        "impl OpCode {\n"
        "\tfn new(code: u8, mnemonic: Operation, len: u8, cycles: u8, mode: AddressingMode) -> Self {\n"
        "\t\tOpCode {\n"
        "\t\t\tcode,\n"
        "\t\t\tmnemonic,\n"
        "\t\t\tlen,\n"
        "\t\t\tcycles,\n"
        "\t\t\tmode,\n"
        "\t\t}\n"
        "\t}\n"
        "}\n\n"
        "lazy_static! {\n"
        "\tpub static ref CPU_OPS_CODES: Vec<OpCode> = vec![\n"
    )

    with tqdm(
        total=len(lines), unit="opcode", unit_scale=True, desc="Generate Rust format"
    ) as pbar:
        # Add opcodes to Rust code
        for line in lines:
            rust_code += (
                f"\t\tOpCode::new("
                f"{line['opcode']}, "
                f"{line['name'].replace('*', '')}, "
                f"{line['bytes']}, "
                f"{line['cycles']}, "
                f"AddressingMode::{line['addressing_mode']}),\n"
            )
            pbar.update(1)

    rust_code += (
        "\t];\n"
        "\tpub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {\n"
        "\t\tlet mut map = HashMap::new();\n"
        "\t\tfor cpuop in &*CPU_OPS_CODES {\n"
        "\t\t\tmap.insert(cpuop.code, cpuop);\n"
        "\t\t}\n"
        "\t\tmap\n"
        "\t};\n"
        "}\n"
    )

    return rust_code


if __name__ == "__main__":
    main()
