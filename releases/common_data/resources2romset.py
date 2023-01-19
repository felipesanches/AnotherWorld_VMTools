#!/usr/bin/env python3
#
# (c) 2020 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later
#
# This module generates the ROM files needed for the FPGA project at
# https://github.com/felipesanches/AnotherWorld_FPGA


class ROMSet():
    def __init__(self, input_dir, output_dir, resource_ids):
        self.input_dir = input_dir
        self.output_dir = f"{output_dir}/romset"
        self.resource_ids = resource_ids


    def generate(self):
        import os
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)

        self.generate_text_string_roms()
        self.generate_font_data_rom()
        self.generate_bytecode_rom()
        self.generate_cinematic_rom()
        self.generate_video2_rom()
        #FIXME! self.generate_screens_rom()
        #TODO: self.generate_samples_rom()
        self.generate_palettes_rom()


    def generate_text_string_roms(self):
        '''
            FIXME: This should ideally be extracted from the original game files
            but we currently don't know how to do it, so we keep these hardcoded
            ROM files which where put together based on the hardcoded data found
            in the Adrien Sanglard's VM codebase:
            https://github.com/fabiensanglard/Another-World-Bytecode-Interpreter

            See also:
            https://github.com/felipesanches/AnotherWorld_VMTools/issues/15
        '''
        import shutil
        for filename in ['str_data.rom',
                         'str_index.rom']:
            shutil.copyfile(f'hardcoded_data/{filename}',
                            f'{self.output_dir}/{filename}')


    def generate_font_data_rom(self):
        # FIXME: This should ideally be extracted from the original game files
        import shutil
        filename = 'anotherworld_chargen.rom'
        shutil.copyfile(f'hardcoded_data/{filename}',
                        f'{self.output_dir}/{filename}')


    def generate_bytecode_rom(self):
        bytecode_rom = open(f"{self.output_dir}/bytecode.rom", "wb")
        for res in self.resource_ids["bytecode"]:
            data = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb").read()
            for i in range(0x10000):
                if i < len(data):
                    bytecode_rom.write(bytes([data[i]]))
                else:
                    bytecode_rom.write(bytes([0xFF]))


    def generate_screens_rom(self):
        screens_rom = open(f"{self.output_dir}/screens.rom", "wb")
        for res in self.resource_ids["screen"]:
            data = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb").read()
            offset = 0
            for h in range(200):
                for w in range(40):
                    p = [
                        data[offset + 8000 * 3],
                        data[offset + 8000 * 2],
                        data[offset + 8000 * 1],
                        data[offset + 8000 * 0]
                    ]
                    for j in range(4):
                        value = 0
                        for i in range(8):
                            value <<= 1
                            if p[i & 3] & 0x80:
                                value |= 1
                            p[i & 3] <<= 1
                        screens_rom.write(bytes([(value >> 4) & 0x0F]))
                        screens_rom.write(bytes([value & 0x0F]))
                    offset += 1
                for i in range(512-320):
                    screens_rom.write(bytes([0xFF]))
            for i in range(256-200):
                for j in range(512):
                    screens_rom.write(bytes([0xFF]))

        screens_rom.close()


    def generate_samples_rom(self):
        # TODO: Implement-me!
        samples = open(f"{self.output_dir}/samples.rom", "wb")
        for res in self.resource_ids["sample"]:
            s = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb")
            value = s.read(1)
            value += s.read(1) << 8
            print("0x%04X" % value)


    def generate_palettes_rom(self):
        palettes = open(f"{self.output_dir}/palettes.rom", "wb")
        for res in self.resource_ids["palette"]:
            pal = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb").read()
            palettes.write(pal)


    def generate_cinematic_rom(self):
        rom = open(f"{self.output_dir}/cinematic.rom", "wb")
        for res in self.resource_ids["cinematic"]:
            cinematic = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb").read()
            for i in range(0x10000):
                if i < len(cinematic):
                    rom.write(bytes([cinematic[i]]))
                else:
                    rom.write(bytes([0xFF]))
        rom.close()


    def generate_video2_rom(self):
        rom = open(f"{self.output_dir}/video2.rom", "wb")
        assert(len(self.resource_ids["video2"]) <= 1)
        res = self.resource_ids["video2"][0]
        v2data = open(f"{self.input_dir}/resource-0x{res:02x}.bin", "rb").read()
        for i in range(0x8000):
            if i < len(v2data):
                rom.write(bytes([v2data[i]]))
            else:
                rom.write(bytes([0xFF]))
        rom.close()
