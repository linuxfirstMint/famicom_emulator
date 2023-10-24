import parsel
from official import extract_line


def test_extract_line():
    html = """
    <html>
    <head>
      <meta name="GENERATOR" content="Microsoft FrontPage 4.0">
      <meta http-equiv="Content-Type" content="text/html; charset=iso-8859-1">
      <title>6502 Reference</title>
      <link rel="StyleSheet" href="obelisk.css" type="text/css" media="screen,print">
    </head>
    <body>
    <br>
    <hr>
    <h2>Instruction Reference</h2>

    <p>Click on any of following links to go straight to the information
    for that instruction.</p>

    <p><table border="1" cellpadding="0" cellspacing="0" width="450">
      <tr>
        <td width="7%" height="25">
          <p><center><a href="#ADC">ADC</a></center></td>
        <td width="7%">
          <p><center><a href="#AND">AND</a></center></td>
        <td width="7%">
          <p><center><a href="#ASL">ASL</a></center></td>
        <td width="7%">
          <p><center><a href="#BCC">BCC</a></center></td>
        <td width="7%">
          <p><center><a href="#BCS">BCS</a></center></td>
        <td width="7%">
          <p><center><a href="#BEQ">BEQ</a></center></td>
        <td width="7%">
          <p><center><a href="#BIT">BIT</a></center></td>
        <td width="7%">
          <p><center><a href="#BMI">BMI</a></center></td>
        <td width="7%">
          <p><center><a href="#BNE">BNE</a></center></td>
        <td width="7%">
          <p><center><a href="#BPL">BPL</a></center></td>
        <td width="7%">
          <p><center><a href="#BRK">BRK</a></center></td>
        <td width="7%">
          <p><center><a href="#BVC">BVC</a></center></td>
        <td width="7%">
          <p><center><a href="#BVS">BVS</a></center></td>
        <td width="7%">
          <p><center><a href="#CLC">CLC</a></center></td>
      </tr>
      <tr>
        <td width="7%" height="25">
          <p><center><a href="#CLD">CLD</a></center></td>
        <td width="7%">
          <p><center><a href="#CLI">CLI</a></center></td>
        <td width="7%">
          <p><center><a href="#CLV">CLV</a></center></td>
        <td width="7%">
          <p><center><a href="#CMP">CMP</a></center></td>
        <td width="7%">
          <p><center><a href="#CPX">CPX</a></center></td>
        <td width="7%">
          <p><center><a href="#CPY">CPY</a></center></td>
        <td width="7%">
          <p><center><a href="#DEC">DEC</a></center></td>
        <td width="7%">
          <p><center><a href="#DEX">DEX</a></center></td>
        <td width="7%">
          <p><center><a href="#DEY">DEY</a></center></td>
        <td width="7%">
          <p><center><a href="#EOR">EOR</a></center></td>
        <td width="7%">
          <p><center><a href="#INC">INC</a></center></td>
        <td width="7%">
          <p><center><a href="#INX">INX</a></center></td>
        <td width="7%">
          <p><center><a href="#INY">INY</a></center></td>
        <td width="7%">
          <p><center><a href="#JMP">JMP</a></center></td>
      </tr>
      <tr>
        <td width="7%" height="25">
          <p><center><a href="#JSR">JSR</a></center></td>
        <td width="7%">
          <p><center><a href="#LDA">LDA</a></center></td>
        <td width="7%">
          <p><center><a href="#LDX">LDX</a></center></td>
        <td width="7%">
          <p><center><a href="#LDY">LDY</a></center></td>
        <td width="7%">
          <p><center><a href="#LSR">LSR</a></center></td>
        <td width="7%">
          <p><center><a href="#NOP">NOP</a></center></td>
        <td width="7%">
          <p><center><a href="#ORA">ORA</a></center></td>
        <td width="7%">
          <p><center><a href="#PHA">PHA</a></center></td>
        <td width="7%">
          <p><center><a href="#PHP">PHP</a></center></td>
        <td width="7%">
          <p><center><a href="#PLA">PLA</a></center></td>
        <td width="7%">
          <p><center><a href="#PLP">PLP</a></center></td>
        <td width="7%">
          <p><center><a href="#ROL">ROL</a></center></td>
        <td width="7%">
          <p><center><a href="#ROR">ROR</a></center></td>
        <td width="7%">
          <p><center><a href="#RTI">RTI</a></center></td>
      </tr>
      <tr>
        <td width="7%" height="25">
          <p><center><a href="#RTS">RTS</a></center></td>
        <td width="7%">
          <p><center><a href="#SBC">SBC</a></center></td>
        <td width="7%">
          <p><center><a href="#SEC">SEC</a></center></td>
        <td width="7%">
          <p><center><a href="#SED">SED</a></center></td>
        <td width="7%">
          <p><center><a href="#SEI">SEI</a></center></td>
        <td width="7%">
          <p><center><a href="#STA">STA</a></center></td>
        <td width="7%">
          <p><center><a href="#STX">STX</a></center></td>
        <td width="7%">
          <p><center><a href="#STY">STY</a></center></td>
        <td width="7%">
          <p><center><a href="#TAX">TAX</a></center></td>
        <td width="7%">
          <p><center><a href="#TAY">TAY</a></center></td>
        <td width="7%">
          <p><center><a href="#TSX">TSX</a></center></td>
        <td width="7%">
          <p><center><a href="#TXA">TXA</a></center></td>
        <td width="7%">
          <p><center><a href="#TXS">TXS</a></center></td>
        <td width="7%">
          <p><center><a href="#TYA">TYA</a></center></td>
      </tr>
    </table>

    <h3><a name="ADC"></a>ADC - Add with Carry</h3>

    <p>A,Z,C,N = A+M+C</p>

    <p>This instruction adds the contents of a memory location to
    the accumulator together with the carry bit. If overflow occurs
    the carry bit is set, this enables multiple byte addition to be
    performed.</p>

    <p>Processor Status after use:</p>

    <p><table height="171" cellspacing="0" cellpadding="0" width="450" border="1">
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#C">C</a></td>
        <td width="45%"><a href="registers.html#C">Carry Flag</a></td>
        <td width="50%">Set if overflow in bit 7</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#Z">Z</a></td>
        <td width="45%"><a href="registers.html#Z">Zero Flag</a></td>
        <td width="50%">Set if A = 0</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#I">I</a></td>
        <td width="45%"><a href="registers.html#I">Interrupt Disable</a></td>
        <td width="50%">Not affected</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#D">D</a></td>
        <td width="45%"><a href="registers.html#D">Decimal Mode Flag</a></td>
        <td width="50%">Not affected</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#B">B</a></td>
        <td width="45%"><a href="registers.html#B">Break Command</a></td>
        <td width="50%">Not affected</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#V">V</a></td>
        <td width="45%"><a href="registers.html#V">Overflow Flag</a></td>
        <td width="50%">Set if sign bit is incorrect</td>
      </tr>
      <tr>
        <td align="MIDDLE" width="5%" height="24"><a href="registers.html#N">N</a></td>
        <td width="45%"><a href="registers.html#N">Negative Flag</a></td>
        <td width="50%">Set if bit 7 set</td>
      </tr>
    </table>

    <p><table height="228" cellspacing="0" cellpadding="0" width="450" border="1">
      <tr>
        <td width="30%" height="25"><b>Addressing Mode</b></td>
        <td width="15%" height="25">
          <p><center><b>Opcode</b></center></td>
        <td width="15%" height="25">
          <p><center><b>Bytes</b></center></td>
        <td width="34%" height="25"><b>Cycles</b></td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#IMM">Immediate</a></td>
        <td width="15%" height="25">
          <p><center>$69</center></td>
        <td width="15%" height="25">
          <p><center>2</center></td>
        <td height="25">2</td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#ZPG">Zero Page</a></td>
        <td width="15%" height="25">
          <p><center>$65</center></td>
        <td width="15%" height="25">
          <p><center>2</center></td>
        <td height="25">3</td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#ZPX">Zero Page,X</a></td>
        <td width="15%" height="25">
          <p><center>$75</center></td>
        <td width="15%" height="25">
          <p><center>2</center></td>
        <td height="25">4</td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#ABS">Absolute</a></td>
        <td width="15%" height="25">
          <p><center>$6D</center></td>
        <td width="15%" height="25">
          <p><center>3</center></td>
        <td height="25">4</td>
      </tr>
      <tr>
        <td height="24"><a href="addressing.html#ABX">Absolute,X</a></td>
        <td width="15%" height="24">
          <p><center>$7D</center></td>
        <td width="15%" height="24">
          <p><center>3</center></td>
        <td height="24">4 (+1 if page crossed)</td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#ABY">Absolute,Y</a></td>
        <td width="15%" height="25">
          <p><center>$79</center></td>
        <td width="15%" height="25">
          <p><center>3</center></td>
        <td height="25">4 (+1 if page crossed)</td>
      </tr>
      <tr>
        <td height="25"><a href="addressing.html#IDX">(Indirect,X)</a></td>
        <td width="15%" height="25">
          <p><center>$61</center></td>
        <td width="15%" height="25">
          <p><center>2</center></td>
        <td height="25">6</td>
      </tr>
      <tr>
        <td height="26"><a href="addressing.html#IDY">(Indirect),Y</a></td>
        <td width="15%" height="26">
          <p><center>$71</center></td>
        <td width="15%" height="26">
          <p><center>2</center></td>
        <td height="26">5 (+1 if page crossed)</td>
      </tr>
    </table>
    <hr align="LEFT">This page was last updated on 17th February, 2008
    </body>
    </html>
    <!--
         FILE ARCHIVED ON 04:46:47 Apr 28, 2021 AND RETRIEVED FROM THE
         INTERNET ARCHIVE ON 19:10:50 Sep 23, 2021.
         JAVASCRIPT APPENDED BY WAYBACK MACHINE, COPYRIGHT INTERNET ARCHIVE.

         ALL OTHER CONTENT MAY ALSO BE PROTECTED BY COPYRIGHT (17 U.S.C.
         SECTION 108(a)(3)).
    -->

    """
    selector = parsel.Selector(text=html)
    expected_output = [
        {
            "opcode": "0x69",
            "name": "ADC",
            "bytes": "2",
            "cycles": "2",
            "addressing_mode": "Immediate",
            "group": "Official",
        },
        {
            "opcode": "0x65",
            "name": "ADC",
            "bytes": "2",
            "cycles": "3",
            "addressing_mode": "ZeroPage",
            "group": "Official",
        },
        {
            "opcode": "0x75",
            "name": "ADC",
            "bytes": "2",
            "cycles": "4",
            "addressing_mode": "ZeroPage_X",
            "group": "Official",
        },
        {
            "opcode": "0x6D",
            "name": "ADC",
            "bytes": "3",
            "cycles": "4",
            "addressing_mode": "Absolute",
            "group": "Official",
        },
        {
            "opcode": "0x7D",
            "name": "ADC",
            "bytes": "3",
            "cycles": "4",
            "addressing_mode": "Absolute_X",
            "group": "Official",
        },
        {
            "opcode": "0x79",
            "name": "ADC",
            "bytes": "3",
            "cycles": "4",
            "addressing_mode": "Absolute_Y",
            "group": "Official",
        },
        {
            "opcode": "0x61",
            "name": "ADC",
            "bytes": "2",
            "cycles": "6",
            "addressing_mode": "Indirect_X",
            "group": "Official",
        },
        {
            "opcode": "0x71",
            "name": "ADC",
            "bytes": "2",
            "cycles": "5",
            "addressing_mode": "Indirect_Y",
            "group": "Official",
        },
    ]

    item = extract_line(selector)
    assert item == expected_output
