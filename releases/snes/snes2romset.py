#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later
#
# This program extracts game assets from the SNES version
# of Another World which has the following checksums:
#
# md5sum = TODO
# sha1sum = TODO
# crc32 = TODO

class SNESROMSet():
    def __init__(self, input_dir, output_dir):
        self.input_dir = input_dir
        self.output_dir = f"{output_dir}/snes/romset"

        input_rom_file = f"{self.input_dir}"
        self.raw = open(input_rom_file, "rb").read()

    def generate(self):
        import os
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)

        self.generate_bytecode_rom()
        #self.generate_cinematic_rom()
        #self.generate_video2_rom()

    def generate_bytecode_rom(self):
        import sys
        print("Extracting BYTECODE...")
        chunks = [
            (0x74A4C, 0x10000), # Intro Sequence       # FIXME: wrong-length
            (0x81CB0, 0x51FD),  # Arrival + Beast run
        ]
        self.extract_resource(f"{self.output_dir}/bytecode.rom", chunks)


    def extract_resource(self, filename, chunks):
        resulting_rom = open(filename, "wb")
        for start, length in chunks:
            data=self.raw[start:start+length]
            for addr in range(0x10000):
                if addr < len(data):
                    resulting_rom.write(bytes([data[addr]]))
                else:
                    resulting_rom.write(bytes([0xFF]))


if __name__ == '__main__':
    import sys
    import os

    if len(sys.argv) != 3:
        sys.exit(f"usage: {sys.argv[0]} <input_dir> <output_dir>")

    input_dir = sys.argv[1]
    output_dir = sys.argv[2]
    romset = SNESROMSet(input_dir, output_dir)
    romset.generate()
