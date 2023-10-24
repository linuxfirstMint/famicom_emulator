from typing import Any, Dict, List, Tuple

import parsel


def extract_selector(content: bytes) -> parsel.Selector:
    """
    Extracts a parsel selector object from the given content.

    Args:
        content (bytes): The content to extract the selector from.

    Returns:
        parsel.Selector: The selector object extracted from the content.
    """
    return parsel.Selector(body=content)


def extract_modes(selector: parsel.Selector, index: int) -> List[str]:
    """
    Extracts the addressing modes for an opcode from the given selector and index.

    Args:
        `selector` (parsel.Selector): The selector to use for extracting the modes.
        `index` (int): The index of the table containing the modes.

    Returns:
        `List[str]`: A list of the extracted addressing modes.
    """
    modes = selector.xpath(f"//table[{index}]/tr/td/a/text()").getall()
    cleaned_modes = [clean_text(mode) for mode in modes]
    cleaned_modes = [normalize_spacing(mode) for mode in cleaned_modes]
    return correct_errate_list(cleaned_modes)


def extract_opcodes_bytes(
    selector: parsel.Selector, index: int
) -> Tuple[List[str], List[int]]:
    """
    Extracts opcodes and their corresponding byte counts from a selector object.

    Args:
        `selector` (parsel.Selector): The selector object to extract opcodes from.
        `index` (int): The index of the table containing the opcodes.

    Returns:
        `Tuple[List[str], List[int]]`: A tuple containing two lists: the opcodes as strings
        (prefixed with '0x') and their corresponding byte counts.
    """
    opcode_bytes = selector.xpath(f"//table[{index}]/tr/td/center/text()").getall()
    opcodes, bytes_counts = disassembly(opcode_bytes)
    opcodes = [f"0x{opcode[1:]}" for opcode in opcodes]
    return opcodes, bytes_counts


def extract_cycles_from_selector(selector: parsel.Selector, index: int) -> List[str]:
    """
    Extracts the cycle information from the given selector and index.

    Args:
        `selector` (parsel.Selector): The selector to extract cycle information from.
        `index` (int): The index of the table to extract cycle information from.

    Returns:
        `List[str]`: A list of cycle information extracted from the selector.
    """
    cycles = selector.xpath(f"//table[{index}]/tr/td/text()").getall()
    return extract_cycles(cycles)


def extract_line(
    selector: parsel.Selector, table_index: int = 3
) -> List[Dict[str, Any]]:
    """
    Extracts opcode information from a given selector and table index.

    opcode information is the table where mode, opcode, bytes, and cycles are stored.

    Args:
        selector (parsel.Selector): The selector to extract information from.
        table_index (int): The index of the table to extract information from.

    Returns:
        List[Dict[str, Any]]: A list of dictionaries containing opcode information.
    """
    line = []
    for i, name in enumerate(selector.xpath("//h3/text()").getall()):
        modes = extract_modes(selector, i + table_index)
        opcodes, bytes_counts = extract_opcodes_bytes(selector, i + table_index)
        cycles = extract_cycles_from_selector(selector, i + table_index)

        for k in range(0, len(opcodes), 1):
            line.append(
                {
                    "opcode": opcodes[k],
                    "name": name.split("-")[0].strip(),
                    "bytes": bytes_counts[k],
                    "cycles": cycles[k],
                    "addressing_mode": modes[k],
                    "group": "Official",  # 対象のurl先にはgroupがないので、ここで追加
                }
            )
        table_index += 1
    return line


def opcode_parse(content: bytes) -> Dict[str, Any]:
    """
    Parses the given opcode content and returns a dictionary containing the opcode line.

    Args:
        `content` (bytes): The opcode content to parse.

    Returns:
        `Dict[str, Any]`: A dictionary containing the opcode line.
    """
    selector = extract_selector(content)
    line = extract_line(selector)
    return {"line": line}


def disassembly(base: List[str]) -> Tuple[List[str], List[int]]:
    """
    Disassembles the given list of base codes into opcodes and byte counts.

    Args:
        base (List[str]): A list of base codes.

    Returns:
        Tuple[List[str], List[int]]: A tuple containing two lists - opcodes and byte counts.
    """
    opcodes = []
    byte_couts = []

    # ORA -> Absolute -> Bytes is <p align="center">3</p> due to a different structure in the base, so this is a temporary workaround.
    # The basic structure of bytes is as follows:
    # Example: ORA -> Absolute -> Bytes is <center>2</center>

    if base[0] == "$09":
        base.insert(7, "3")

    for i in range(0, len(base), 2):
        opcodes.append(base[i])
        byte_couts.append(base[i + 1])

    return opcodes, byte_couts


ADDRESSING_MODE = {
    "Zero Page": "ZeroPage",
    "Zero Page,X": "ZeroPage_X",
    "Zero Page,Y": "ZeroPage_Y",
    "Absolute,X": "Absolute_X",
    "Absolute,Y": "Absolute_Y",
    "(Indirect,X)": "Indirect_X",
    "(Indirect),Y": "Indirect_Y",
    "Implied": "Implicit",
}


def correct_errate_list(mode_list: List[str]) -> List[str]:
    """
    Given a list of addressing modes, returns a corrected list with any errors fixed.

    Args:
        `mode_list` (List[str]): A list of addressing modes.

    Returns:
        `List[str]`: A corrected list of addressing modes.
    """
    return [ADDRESSING_MODE.get(mode, mode) for mode in mode_list]


def normalize_spacing(text: str) -> str:
    """
    Normalizes the spacing in a given text by replacing any consecutive whitespace characters with a single space.

    Args:
        `text` (str): The text to normalize.

    Returns:
        `str`: The normalized text.
    """
    return " ".join(text.split())


def extract_cycles(cycles: List[str]) -> List[str]:
    """
    Extracts the first number from each cycle in the given list of cycles and returns a list of formatted cycles.

    Args:
        `cycles` (List[str]): A list of cycles in string format.

    Returns:
        `List[str]`: A list of formatted cycles, where each cycle is represented by its first number.
    """
    formatted_cycles = []
    for cycle in cycles:
        cleaned_cycle = cycle.replace("\r\n", "").strip()

        # is not empty string (e.g. "")
        if not cleaned_cycle:
            continue

        # first number only (e.g. "2+")
        splitted_cycle = cleaned_cycle.split(" ")[0]

        # "+" is not included in the cycle string (e.g. "2+")
        if "+" not in splitted_cycle:
            formatted_cycles.append(splitted_cycle)

    return formatted_cycles


def clean_text(text: str) -> str:
    """
    Removes any leading or trailing whitespace and newlines from the given text.

    Args:
        `text` (str): The text to clean.

    Returns:
        `str`: The cleaned text.
    """
    return text.replace("\r\n", "").strip()
