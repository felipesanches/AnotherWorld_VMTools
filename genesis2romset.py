#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 2 or later
#
# This program extracts game assets from the SEGA Genesis (Europe) Another World
# cartridge ROM which has the following checksums:
#
# md5sum = 8cc928edf09159401618e273028216ea
# sha1sum = 9d98d6817b3e3651837bb2692f7a2a60a608c055

input_rom_file = None
output_dir = None
data = None

def generate_bytecode_rom():
    bytecode_rom = open(f"{output_dir}/bytecode.rom", "wb")
    # offs:0x5281a len: 9c8c
    bytecode_offsets = [0x3f576, 0x5281a, 0x693e8, 0x88716, 0x919a0, 0xbcab8, 0xada78]

    for offset in bytecode_offsets:
        for i in range(0x10000):
            addr = offset + i
            if offset != -1 and addr < len(data):
                bytecode_rom.write(bytes([data[addr]]))
            else:
                bytecode_rom.write(bytes([0xFF]))


def generate_cinematic_rom():
    cinematic_rom = open(f"{output_dir}/cinematic.rom", "wb")    
    cinematic_offsets = [0x44774, 0x5c4b4, 0x7894c, 0x8a69e, 0x9e0b4, 0xbd612, 0xae65c]

    for offset in cinematic_offsets:
        for i in range(0x10000):
            addr = offset + i
            if offset != -1 and addr < len(data):
                cinematic_rom.write(bytes([data[addr]]))
            else:
                cinematic_rom.write(bytes([0xFF]))


def generate_video2_rom():
    video2_rom = open(f"{output_dir}/video2.rom", "wb")    
    video2_offset = 0xcc000
    video2_len = 0x6214
    for i in range(0x8000):
        addr = video2_offset + i
        if i < video2_len:
            video2_rom.write(bytes([data[addr]]))
        else:
            video2_rom.write(bytes([0xFF]))


import sys
import os

if len(sys.argv) != 3:
    sys.exit(f"usage: {sys.argv[0]} <input_rom_file> <output_dir>")

input_rom_file = sys.argv[1]
output_dir = sys.argv[2]
data = open(input_rom_file, "rb").read()

generate_bytecode_rom()
generate_cinematic_rom()
generate_video2_rom()
