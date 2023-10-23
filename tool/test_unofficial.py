from gen_opcodes.unofficial import parse_opcode_lines


def test_parse_opcode_lines():
    result_list = [
        "AAC (ANC) [ANC]",
        "Immediate   |AAC #arg   |$0B| 2 | 2",
        "Immediate   |AAC #arg   |$2B| 2 | 2",
        "AAX (SAX) [AXS]",
        "Zero Page   |AAX arg    |$87| 2 | 3",
        "Zero Page,Y |AAX arg,Y  |$97| 2 | 4",
        "(Indirect,X)|AAX (arg,X)|$83| 2 | 6",
        "Absolute    |AAX arg    |$8F| 3 | 4",
        "KIL (JAM) [HLT]" "Implied     |KIL        |$02| 1 | -",
        "Implied     |KIL        |$12| 1 | -",
        "LAR (LAE) [LAS]" "Absolute,Y  |LAR arg,Y  |$BB| 3 | 4 *",
    ]

    expected_output = [
        {
            "opcode": "0x0B",
            "name": "ANC",
            "bytes": "2",
            "cycles": "2",
            "addressing_mode": "Immediate",
        },
        {
            "opcode": "0x2B",
            "name": "ANC",
            "bytes": "2",
            "cycles": "2",
            "addressing_mode": "Immediate",
        },
        {
            "opcode": "0x87",
            "name": "SAX",
            "bytes": "2",
            "cycles": "3",
            "addressing_mode": "ZeroPage",
        },
        {
            "opcode": "0x97",
            "name": "SAX",
            "bytes": "2",
            "cycles": "4",
            "addressing_mode": "ZeroPage_Y",
        },
        {
            "opcode": "0x83",
            "name": "SAX",
            "bytes": "2",
            "cycles": "6",
            "addressing_mode": "Indirect_X",
        },
        {
            "opcode": "0x8F",
            "name": "SAX",
            "bytes": "3",
            "cycles": "4",
            "addressing_mode": "Absolute",
        },
        {
            "opcode": "0x02",
            "name": "SAX",
            "bytes": "1",
            "cycles": "0",
            "addressing_mode": "KIL (JAM) [HLT]Implied",
        },
        {
            "opcode": "0x12",
            "name": "SAX",
            "bytes": "1",
            "cycles": "0",
            "addressing_mode": "Implicit",
        },
        {
            "opcode": "0xBB",
            "name": "SAX",
            "bytes": "3",
            "cycles": "4",
            "addressing_mode": "LAR (LAE) [LAS]Absolute,Y",
        },
    ]

    parsed_opcodes = parse_opcode_lines(result_list)

    assert parsed_opcodes == expected_output


# [{'opcode': '0x0B', 'name': 'ANC', 'bytes': '2', 'cycles': '2', 'addressing_mode': 'Immediate'},
# {'opcode': '0x2B', 'name': 'ANC', 'bytes': '2', 'cycles': '2', 'addressing_mode': 'Immediate'},
# {'opcode': '0x87', 'name': 'SAX', 'bytes': '2', 'cycles': '3', 'addressing_mode': 'ZeroPage'},
# {'opcode': '0x97', 'name': 'SAX', 'bytes': '2', 'cycles': '4', 'addressing_mode': 'ZeroPage_Y'},
# {'opcode': '0x83', 'name': 'SAX', 'bytes': '2', 'cycles': '6', 'addressing_mode': 'Indirect_X'},
# {'opcode': '0x8F', 'name': 'SAX', 'bytes': '3', 'cycles': '4', 'addressing_mode': 'Absolute'},
# {'opcode': '0x02', 'name': 'SAX', 'bytes': '1', 'cycles': '0', 'addressing_mode': 'KIL (JAM) [HLT]Implied'},
# {'opcode': '0x12', 'name': 'SAX', 'bytes': '1', 'cycles': '0', 'addressing_mode': 'Implicit'},
# {'opcode': '0xBB', 'name': 'SAX', 'bytes': '3', 'cycles': '4', 'addressing_mode': 'LAR (LAE) [LAS]Absolute,Y'}]
