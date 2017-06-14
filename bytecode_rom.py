#!/usr/bin/env python

target_folder = "data/aw_msdos"
bytecode_resource_ids = [0x15, 0x18, 0x1b, 0x1e, 0x21, 0x24, 0x27, 0x2a, 0x7e]

bytecode_rom = open("%s/bytecode.rom" % (target_folder), "w")
for res in bytecode_resource_ids:
  data = open("%s/resource-0x%02x.bin" % (target_folder, res)).read()
  for i in xrange(0x10000):
    if i < len(data):
      bytecode_rom.write(data[i])
    else:
      bytecode_rom.write(chr(0xFF))
