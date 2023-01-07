#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later
#
# This program extracts game assets from the Symbian OS Series 60 demo version
# of Another World which has the following checksums:
#
# md5sum = fe4742b67415eb16ef340548573538b8
# sha1sum = 4c4a908e3fadc029efdd7e69e0e35b042ba4e489
# crc32 = da3739cb

class SymbianDemoROMSet():
    def __init__(self, input_dir, output_dir):
        self.input_dir = input_dir
        self.output_dir = f"{output_dir}/symbian_demo/romset"

        input_rom_file = f"{self.input_dir}/locked_anotherworld.sis"
        import zlib
        data = open(input_rom_file, "rb").read()
        addr = 0xBBA
        length = 749540
        packed = data[addr:addr+length]
        self.raw = zlib.decompress(packed)


    def generate(self):
        import os
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)

        #self.generate_bytecode_rom()
        self.generate_cinematic_rom()
        #self.generate_video2_rom()


    def generate_cinematic_rom(self):
        import lzma
        cinematic_rom = open(f"{self.output_dir}/cinematic.rom", "wb")    
        cinematic_compressed_chunks = [(0x6551E, 8388608)]
        for offset, length in cinematic_compressed_chunks:
            decompressed = lzma.LZMADecompressor().decompress(data=self.raw[offset:offset+length])            
            for addr in range(0x10000):
                if addr < len(decompressed):
                    cinematic_rom.write(bytes([decompressed[addr]]))
                else:
                    cinematic_rom.write(bytes([0xFF]))


if __name__ == '__main__':
    import sys
    import os

    if len(sys.argv) != 3:
        sys.exit(f"usage: {sys.argv[0]} <input_dir> <output_dir>")

    input_dir = sys.argv[1]
    output_dir = sys.argv[2]
    romset = SymbianDemoROMSet(input_dir, output_dir)
    romset.generate()
