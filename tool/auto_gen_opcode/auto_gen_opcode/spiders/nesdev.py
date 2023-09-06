import re

import scrapy

# debug
# from tool.auto_gen_opcode.auto_gen_opcode.spiders.items import AutoGenOpcodeItem
# command line
from auto_gen_opcode.spiders.items import AutoGenOpcodeItem


class NesdevSpider(scrapy.Spider):
    name = "nesdev"
    allowed_domains = ["www.nesdev.org"]
    start_urls = ["https://www.nesdev.org/obelisk-6502-guide/reference.html"]

    def parse(self, response):
        """
        スクレイピングのメイン処理を行うメソッド
        レスポンスからデータを抽出し、Itemに格納して返す
        """
        item = AutoGenOpcodeItem()
        names = response.xpath("//h3/text()").getall()
        cleaned_names = [name.split("-")[0].strip() for name in names]
        item["name"] = cleaned_names

        line = []
        table_index = 3  # mode,opcode,bytes,cyclesが格納されているtableの始まり
        for i, name in enumerate(cleaned_names):
            # Addressing Modeの抽出
            modes = response.xpath(f"//table[{i+table_index}]/tr/td/a/text()").getall()
            cleaned_modes = [self.clean_text(mode) for mode in modes]
            cleaned_modes = [self.normalize_spacing(mode) for mode in cleaned_modes]
            formatted_modes = self.correct_errate_list(cleaned_modes)

            # OpcodeとBytesの抽出
            opcode_bytes = response.xpath(
                f"//table[{i+table_index}]/tr/td/center/text()"
            ).getall()
            opcodes, bytes_counts = self.disassembly(opcode_bytes)
            formatted_opcodes = [f"0x{opcode[1:]}" for opcode in opcodes]

            # Cyclesの抽出
            cycles = response.xpath(f"//table[{i+table_index}]/tr/td/text()").getall()
            formatted_cycles = self.extract_cycles(cycles)

            # アイテムにデータを追加
            item["addressing_mode"] = formatted_modes
            item["opcode"] = formatted_opcodes
            item["bytes"] = bytes_counts
            item["cycles"] = formatted_cycles

            # アイテムを解析結果に追加
            for k in range(0, len(item["opcode"]), 1):
                item_line = {
                    "opcode": item["opcode"][k],
                    "name": name,
                    "bytes": item["bytes"][k],
                    "cycles": item["cycles"][k],
                    "addressing_mode": item["addressing_mode"][k],
                }
                line.append(item_line)
            table_index += 1

        item["line"] = line

        yield item

    def disassembly(self, base):
        first = []
        second = []

        # ORA->Absolute->Bytesだけ<p align="center">3</p>でありbaseの構造が違うため応急処理
        # 基本のbytesの構造
        # ex: ORA->Absolute->Bytesは<center>2</center>

        if base[0] == "$09":
            base.insert(7, 3)
        else:
            pass

        for i in range(0, len(base), 2):
            first_element = base[i]
            second_element = base[i + 1]

            first.append(first_element)
            second.append(second_element)

        return first, second

    def correct_errate_list(self, mode_list):
        AddressingMode = {
            "Zero Page": "ZeroPage",
            "Zero Page,X": "ZeroPage_X",
            "Zero Page,Y": "ZeroPage_Y",
            "Absolute,X": "Absolute_X",
            "Absolute,Y": "Absolute_Y",
            "(Indirect,X)": "Indirect_X",
            "(Indirect),Y": "Indirect_Y",
            "Implied": "Implicit",
        }
        corrected_list = [
            AddressingMode[mode] if mode in AddressingMode else mode
            for mode in mode_list
        ]

        return corrected_list

    def normalize_spacing(self, text):
        """
        連続した空白を1つの空白に置き換えるメソッド
        """
        return re.sub(r"\s+", " ", text)

    def extract_cycles(self, cycles):
        formatted_cycles = []
        for cycle in cycles:
            cleaned_cycle = self.clean_text(cycle)

            # 空文字なら飛ばす
            if "" == cleaned_cycle:
                continue

            # 最初の数字のみを抽出する
            splitted_cycle = self.extract_first_number(cleaned_cycle)

            # "+"がなければ抽出した数字を追加
            if not self.has_plus_sign(splitted_cycle):
                formatted_cycles.append(splitted_cycle)

        return formatted_cycles

    def clean_text(self, text):
        """
        文字列内の余分な空白と改行を取り除くメソッド
        """
        return text.replace("\r\n", "").strip()

    def extract_first_number(self, text):
        if " " in text:
            return text.split(" ")[0]
        else:
            return text

    def has_plus_sign(self, text):
        return re.match(r"\+", text)
