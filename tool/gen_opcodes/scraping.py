from typing import Any, Callable, Dict, List

import official
import requests
import unofficial
from tqdm import tqdm


def full_opcode_info() -> List[Dict[str, str]]:
    """
    Returns a list of dictionaries containing information about all the opcodes
    used in the 6502 microprocessor. The information is scraped from two different
    sources - an official reference page and an unofficial page containing
    undocumented opcodes.

    Returns:
    --------
    List[Dict[str, str]]:
        A list of dictionaries containing information about all the opcodes used
        in the 6502 microprocessor.
    """
    official_url = "https://www.nesdev.org/obelisk-6502-guide/reference.html"
    un_official_url = "https://www.nesdev.org/undocumented_opcodes.txt"

    official_opcode_info = get_opcode_info(official_url, official.opcode_parse)
    un_official_opcode_info = get_opcode_info(un_official_url, unofficial.opcode_parse)

    if official_opcode_info is not None and un_official_opcode_info is not None:
        full_opcode_info = official_opcode_info + un_official_opcode_info
        return full_opcode_info
    else:
        return []


def get_opcode_info(
    url: str, parse_func: Callable[[bytes], Dict[str, Any]]
) -> List[Dict[str, str]] | None:
    """
    Fetches opcode information from the given URL and returns a list of opcode dictionaries.

    Args:
        url (str): The URL to fetch the opcode information from.
        parse_func (Callable[[bytes], Dict[str, Any]]): A function that takes in the response content as bytes and returns a dictionary of opcode information.

    Returns:
        List[Dict[str, str]] | None: A list of opcode dictionaries, or None if the response status code is not 200.
    """
    response = requests.get(url)

    if response.status_code != 200:
        response.raise_for_status()

    content = response.content
    with tqdm(
        total=len(content),
        unit="byte",
        unit_scale=True,
        desc="Fetching opcode info",
    ) as pbar:
        opcode_info: Dict[str, Any] = parse_func(content)
        pbar.update(len(content))

    return opcode_info["line"]
