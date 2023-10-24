import re
from typing import Any, Dict, List

import parsel


def remove_html_tags(text: str) -> str:
    """
    Removes HTML tags from a given string.

    Args:
        `text` (str): The string to remove HTML tags from.

    Returns:
        `str`: The input string with all HTML tags removed.
    """
    return re.sub(r"<.*?>", "", text)


def remove_tabs(text: str) -> str:
    """
    Replaces all tabs in the given text with spaces.

    Args:
        `text` (str): The text to remove tabs from.

    Returns:
        `str`: The text with all tabs replaced by spaces.
    """
    return re.sub(r"\t", " ", text)


def remove_header_lines(text_list: List[str]) -> List[str]:
    """
    Removes the header lines from a list of strings.

    Args:
        `text_list` (List[str]): A list of strings.

    Returns:
        `List[str]`: The same list of strings with the header lines removed.
    """
    del text_list[0:89]
    return text_list


def remove_addressing_lines(text_list: List[str]) -> List[str]:
    """
    Removes the addressing lines from a list of text.

    Args:
        `text_list` (List[str]): A list of strings representing the text.

    Returns:
        `List[str]`: A list of strings with the addressing lines removed.
    """
    result_list = []
    i = 0
    while i < len(text_list):
        if "=3D" in text_list[i]:
            i += 1
            while i < len(text_list) and "Addressing" not in text_list[i]:
                i += 1
        elif "Addressing" in text_list[i]:
            i += 1
            while i < len(text_list) and not text_list[i].strip():
                i += 1
            if i < len(text_list):
                i += 1
        else:
            result_list.append(text_list[i])
            i += 1
    return result_list


def remove_empty_lines(text_list: List[str]) -> List[str]:
    """
    Removes empty lines from a list of strings.

    Args:
        `text_list` (List[str]): A list of strings.

    Returns:
        `List[str]`: A list of strings with empty lines removed.
    """
    return [line for line in text_list if line.strip()]


def parse_opcode_line(line: str, name: str) -> Dict[str, str]:
    """
    Parses an opcode line and returns a dictionary containing the opcode's name, addressing mode, opcode, bytes, and cycles.

    Args:
        `line` (str): The opcode line to parse.
        `name` (str): The name of the opcode.

    Returns:
        `Dict[str, str]`: A dictionary containing the opcode's name, addressing mode, opcode, bytes, and cycles.
    """
    formatted_line = format_line(line)

    item_line = {
        "name": extract_mnmonic(name),
        "addressing_mode": extract_addressing_mode(formatted_line[0]),
        "opcode": extract_opcode(formatted_line[2]),
        "bytes": formatted_line[3],
        "cycles": extract_cycle(formatted_line[4]),
        "group": "UnOfficial",  # 対象のurl先にはgroupがないので、ここで追加
    }

    return item_line


def parse_opcode_lines(result_list: List[str]) -> List[Dict[str, str]] | None:
    """
    Parses a list of opcode lines and returns a list of dictionaries containing the parsed information.

    Args:
        `result_list` (List[str]): A list of opcode lines to parse.

    Returns:
        `List[Dict[str, str]] | None`: A list of dictionaries containing the parsed information, or None if the input list is empty.
    """
    line = []
    i = 0
    while i < len(result_list):
        if "(" in result_list[i]:
            j = i + 1
            while "|" in result_list[j]:
                line.append(parse_opcode_line(line=result_list[j], name=result_list[i]))
                j += 1
                if j == len(result_list):
                    break
            i = j
    return line


def opcode_parse(content: bytes) -> Dict[str, Any]:
    """
    Parses the given content to extract opcode information.

    Args:
        `content` (bytes): The content to parse.

    Returns:
        `Dict[str, Any]`: A dictionary containing the parsed opcode information.
    """
    selector = parsel.Selector(body=content)
    text = selector.xpath("/html/body").get()
    if text is None:
        return {"line": None}
    text = remove_html_tags(text)
    text = remove_tabs(text)
    text_list = text.split("\n")
    text_list = remove_header_lines(text_list)
    result_list = remove_addressing_lines(text_list)
    result_list = remove_empty_lines(result_list)
    line = parse_opcode_lines(result_list)
    return {"line": line}


def format_line(data: str) -> List[str]:
    """
    Formats a line of opcode data into a list of strings.

    Args:
        `data` (str): A string containing opcode data separated by '|'.

    Returns:
        `List[str]`: A list of strings containing the formatted opcode data.
    """
    split_parts = data.split("|")
    line_parts = [part.strip() for part in split_parts if part != ""]

    return line_parts


def extract_mnmonic(data: str) -> str:
    """
    Extracts the mnemonic from the given data string.

    Args:
        `data` (str): The data string to extract the mnemonic from.

    Returns:
        `str`: The extracted mnemonic.
    """
    parts = data.split(" ")
    # 空白を削除
    removed_space_parts = [part for part in parts if part != ""]
    # 括弧を削除
    mnmonic = re.sub(r"\(|\)", "", removed_space_parts[1])

    # 文字列の先頭に"*"を追加 ex: *SLO"
    # mnmonic = f"*{mnmonic}"

    return mnmonic


def extract_addressing_mode(mode: str) -> str:
    """
    Extracts the addressing mode from the given mode string.

    Args:
        `mode` (str): The mode string to extract the addressing mode from.

    Returns:
        `str`: The extracted addressing mode.
    """
    addressing_mode = correct_errate_list(mode)

    return addressing_mode


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


def correct_errate_list(mode: str) -> str:
    """
    Returns the corrected addressing mode for a given mode string.

    Args:
        `mode` (str): The addressing mode to correct.

    Returns:
        `str`: The corrected addressing mode, or the original mode if no correction is needed.
    """
    return ADDRESSING_MODE.get(mode, mode) if mode in ADDRESSING_MODE else mode


def extract_opcode(opcode: str) -> str:
    """
    Extracts the opcode from a given string and formats it as a hexadecimal value.

    Args:
        `opcode` (str): The opcode to extract.

    Returns:
        `str`: The formatted opcode as a hexadecimal value.
    """
    formatted_opcode = f"0x{opcode[1:]}"

    return formatted_opcode


def extract_cycle(cycle: str) -> str:
    """
    Extracts the cycle from the given string and returns it as a formatted string.

    Args:
        `cycle` (str): The cycle string to extract from.

    Returns:
        `str`: The formatted cycle string.
    """
    match cycle:
        case cycle if "*" in cycle:
            formatted_cycle = cycle.replace("*", "")
            formatted_cycle = formatted_cycle.strip()
        case cycle if "-" in cycle:
            formatted_cycle = cycle.replace("-", "0")
        case _:
            formatted_cycle = cycle

    return formatted_cycle
