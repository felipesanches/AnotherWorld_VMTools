#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later
#
# This program extracts game assets from the SEGA Genesis (Europe) Another World
# cartridge ROM which has the following checksums:
#
# md5sum = 8cc928edf09159401618e273028216ea
# sha1sum = 9d98d6817b3e3651837bb2692f7a2a60a608c055
import os

class GenesisEuropeROMSet():

    def __init__(self, input_rom_file, output_dir):
        self.output_dir = f"{output_dir}/genesis_europe/romset"
        self.data = open(input_rom_file, "rb").read()

        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)


    def generate(self):
        self.generate_text_string_roms()
        #self.generate_font_data_rom()
        self.generate_bytecode_rom()
        self.generate_cinematic_rom()
        self.generate_video2_rom()
        #self.generate_screens_rom()
        #self.generate_samples_rom()
        #self.generate_palettes_rom()


    def generate_text_string_roms(self):
        str_data_rom = open(f"{self.output_dir}/str_data.rom", "wb")
        str_data_rom.seek(0xfff)
        str_data_rom.write(bytes([0x00]))

        str_index_rom = open(f"{self.output_dir}/str_index.rom", "wb")
        str_index_rom.seek(0x7ff)
        str_index_rom.write(bytes([0x00]))

        addr = 0x382B
        strdata_addr = 0
        str_data_rom.seek(0)
        while addr <= 0x46fe:
            index = (self.data[addr] << 8) | self.data[addr+1]
            addr += 2
            str_index_rom.seek(index*2)
            str_index_rom.write(bytes([strdata_addr & 0xff]))
            str_index_rom.write(bytes([(strdata_addr >> 8) & 0xff]))
            while self.data[addr] != 0:
                str_data_rom.write(bytes([self.data[addr]]))
                addr += 1
                strdata_addr += 1
            str_data_rom.write(bytes([0x00]))
            strdata_addr += 1
            addr += 1


    def generate_bytecode_rom(self):
        bytecode_rom = open(f"{self.output_dir}/bytecode.rom", "wb")
        chunks = [(0x3f576, 0x51FD),  # Arrival + beast run    # FIXME: wrong-length
                  (0x5281a, 0x9c8c),      # TODO: verify
                  (0x693e8, 0x10000),     # FIXME: wrong-length
                  (0x88716, 0x10000),     # FIXME: wrong-length
                  (0x919a0, 0x10000),     # FIXME: wrong-length
                  (0xbcab8, 0x10000),     # FIXME: wrong-length
                  (0xada78, 0x10000),     # FIXME: wrong-length
                 ]

        for offset, length in chunks:
            for i in range(0x10000):
                addr = offset + i
                if offset != -1 and addr < len(self.data) and i < length:
                    bytecode_rom.write(bytes([self.data[addr]]))
                else:
                    bytecode_rom.write(bytes([0xFF]))


    def generate_cinematic_rom(self):
        cinematic_rom = open(f"{self.output_dir}/cinematic.rom", "wb")    
        cinematic_offsets = [0x44774, 0x5c4b4, 0x7894c, 0x8a69e, 0x9e0b4, 0xbd612, 0xae65c]

        for offset in cinematic_offsets:
            for i in range(0x10000):
                addr = offset + i
                if offset != -1 and addr < len(self.data):
                    cinematic_rom.write(bytes([self.data[addr]]))
                else:
                    cinematic_rom.write(bytes([0xFF]))


    def generate_video2_rom(self):
        video2_rom = open(f"{self.output_dir}/video2.rom", "wb")    
        video2_offset = 0xcc000
        video2_len = 0x6214
        for i in range(0x8000):
            addr = video2_offset + i
            if i < video2_len:
                video2_rom.write(bytes([self.data[addr]]))
            else:
                video2_rom.write(bytes([0xFF]))


if __name__ == '__main__':
    import sys
    import os

    if len(sys.argv) != 3:
        sys.exit(f"usage: {sys.argv[0]} <input_rom_file> <output_dir>")

    input_rom_file = sys.argv[1]
    output_dir = sys.argv[2]
    romset = GenesisEuropeROMSet(input_rom_file, output_dir)
    romset.generate()
