#!/usr/bin/env python3
#
# (c) 2020 Felipe Correa da Silva Sanches
# Licensed under GPL version 2 or later
#
# This program generates the ROM files needed for the FPGA project at
# https://github.com/felipesanches/AnotherWorld_FPGA

target_folder = "data/aw_msdos"
output_folder = "anotherw/"
bytecode_resource_ids = [0x15, 0x18, 0x1b, 0x1e, 0x21, 0x24, 0x27, 0x2a, 0x7e]
screen_resource_ids = [0x13, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x53, 0x90, 0x91]
music_resource_ids = [
    0x07, 0x89, 0x8a
]
sample_resource_ids = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x09,
    0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x2c,
    0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x35,
    0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d,
    0x3e, 0x3f, 0x40, 0x41, 0x42, 0x4a, 0x4b, 0x4c,
    0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d,
    0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65,
    0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d,
    0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x80,
    0x81, 0x82, 0x83, 0x84, 0x88, 0x8b, 0x8c, 0x8d,
    0x8e
]
palette_resource_ids = [
    0x14, 0x17, 0x1a, 0x1d, 0x20, 0x23, 0x26, 0x29, 0x7d
]
cinematic_resource_ids = [
    0x16, 0x19, 0x1c, 0x1f, 0x22, 0x25, 0x28, 0x2b, 0x7f
]
video2_resource_id = 0x11


def generate_bytecode_rom():
    bytecode_rom = open(f"{output_folder}/bytecode.rom", "wb")
    for res in bytecode_resource_ids:
        data = open(f"{target_folder}/resource-0x{res:02x}.bin", "rb").read()
        for i in range(0x10000):
            if i < len(data):
                bytecode_rom.write(bytes([data[i]]))
            else:
                bytecode_rom.write(bytes([0xFF]))


def generate_screens_rom():
    screens_rom = open(f"{output_folder}/screens.rom", "wb")
    for res in screen_resource_ids:
        data = open(f"{target_folder}/resource-0x{res:02x}.bin", "rb").read()
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


def generate_samples_rom():
    # TODO: Implement-me!
    samples = open(f"{output_folder}/samples.rom", "wb")
    for res in sample_resource_ids:
        s = open(f"{target_folder}/resource-0x{res:02x}.bin", "rb")
        value = s.read(1)
        value += s.read(1) << 8
        print("0x%04X" % value)


def generate_palettes_rom():
    palettes = open(f"{output_folder}/palettes.rom", "wb")
    for res in palette_resource_ids:
        pal = open(f"{target_folder}/resource-0x{res:02x}.bin", "rb").read()
        palettes.write(pal)


def generate_cinematic_rom():
    rom = open(f"{output_folder}/cinematic.rom", "wb")
    for res in cinematic_resource_ids:
        cinematic = open(f"{target_folder}/resource-0x{res:02x}.bin", "rb").read()
        for i in range(0x10000):
            if i < len(cinematic):
                rom.write(bytes([cinematic[i]]))
            else:
                rom.write(bytes([0xFF]))
    rom.close()


def generate_video2_rom():
    rom = open(f"{output_folder}/video2.rom", "wb")
    v2data = open(f"{target_folder}/resource-0x{video2_resource_id:02x}.bin", "rb").read()
    for i in range(0x8000):
        if i < len(v2data):
            rom.write(bytes([v2data[i]]))
        else:
            rom.write(bytes([0xFF]))
    rom.close()


# create the output_folder if it does not yet exist:
import os
if not os.path.exists(output_folder):
    os.makedirs(output_folder)

generate_bytecode_rom()
generate_screens_rom()
#TODO: generate_samples_rom()
generate_palettes_rom()
generate_cinematic_rom()
generate_video2_rom()

