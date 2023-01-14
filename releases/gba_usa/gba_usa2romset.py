#!/usr/bin/env python3
#
# (c) 2023 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later
#
# This program extracts game assets from the GBA USA
# Another World Prototype cartridge ROM which has the following checksums:
#
# md5sum = 9cef2ca9fba8a4532629f8c7e7c9ddf8
# sha1sum = 41d39a0c34f72469dd3fbcc90190605b8ada93e6
import os

class GBAUSAROMSet():

    def __init__(self, input_rom_file, output_dir):
        self.output_dir = f"{output_dir}/gba_usa/romset"
        self.data = open(input_rom_file, "rb").read()

        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)


    def generate(self):
        #self.generate_text_string_roms()
        #self.generate_font_data_rom()
        self.generate_bytecode_rom()
        #self.generate_cinematic_rom()
        #self.generate_video2_rom()
        #self.generate_screens_rom()
        #self.generate_samples_rom()
        #self.generate_palettes_rom()


    def generate_bytecode_rom(self):
        bytecode_rom = open(f"{self.output_dir}/bytecode.rom", "wb")
        bytecode_offsets = [
            0x6ea74, # level 1
            0x813f8, # level 2
        ]

        for offset in bytecode_offsets:
            for i in range(0x10000):
                addr = offset + i
                if offset != -1 and addr < len(self.data):
                    bytecode_rom.write(bytes([self.data[addr]]))
                else:
                    bytecode_rom.write(bytes([0xFF]))


if __name__ == '__main__':
    import sys
    import os

    if len(sys.argv) != 3:
        sys.exit(f"usage: {sys.argv[0]} <input_rom_file> <output_dir>")

    input_rom_file = sys.argv[1]
    output_dir = sys.argv[2]
    romset = GBAUSAROMSet(input_rom_file, output_dir)
    romset.generate()
