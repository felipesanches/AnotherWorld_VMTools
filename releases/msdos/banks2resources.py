#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 2 or later

import os
import sys
from releases.msdos.Unpacker import Unpacker

class MSDOSResources():
    def __init__(self, input_dir, output_dir):
        self.input_dir = input_dir
        self.output_dir = f"{output_dir}/msdos/resources"
        self.memlist_filename = f"{input_dir}/memlist.bin"


    def generate(self):
        import os
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)

        directory = os.path.dirname(self.memlist_filename)
        entries = self.load_memlist()

        for resource_index, entry in enumerate(entries):
            if entry != None:
                print (f"resource:{hex(resource_index)}"
                       f"\tbankId:{entry['bankId']}"
                       f"\ttype:{self.get_type(entry['type'])}"
                       f"\toffset:{hex(entry['bankOffset'])}"
                       f"\tsize:{hex(entry['packedSize'])} / {hex(entry['size'])}"
                       f"\tnext:{hex(entry['bankOffset'] + entry['packedSize'])}")

            bank_file = os.path.join(directory, "bank%02x" % entry["bankId"])
            bank = open(bank_file, "rb")
            bank.seek(entry["bankOffset"])
            data = bank.read(entry["packedSize"])

            if entry["packedSize"] != entry["size"]:
                u = Unpacker()
                data = u.unpack(data)

            if data != None:
                bin_filename = os.path.join(self.output_dir,
                                            f"resource-0x{resource_index:02x}.bin")
                if entry['size'] != len(data):
                    print(f"SHOULD BE {entry['size']} ---- GOT {len(data)}")
                    sys.exit(-1)
                open(bin_filename, "wb").write(data)
            bank.close()

    def read_byte(self, m):
        return ord(m.read(1))

    def read_word(self, m):
        v = self.read_byte(m)
        v = v << 8 | self.read_byte(m)
        return v

    def read_quad(self, m):
        v = self.read_word(m)
        v = v << 16 | self.read_word(m)
        return v

    def read_mem_entry(self, memlist, n):
        memlist.seek(20*n)
        entry = {}
        entry["size"] = self.read_byte(memlist)
        entry["type"] = self.read_byte(memlist)
        entry["unknown_0x02"] = self.read_word(memlist) # unknown
        entry["unknown_0x04"] = self.read_word(memlist) # unknown
        entry["rankNum"] = self.read_byte(memlist)
        entry["bankId"] = self.read_byte(memlist)
        entry["bankOffset"] = self.read_quad(memlist)
        entry["unknown_0x0C"] = self.read_word(memlist) # unknown
        entry["packedSize"] = self.read_word(memlist)
        entry["unknown_0x10"] = self.read_word(memlist) # unknown
        entry["size"] = self.read_word(memlist)
        return entry


    def load_memlist(self):
        if not os.path.exists(self.memlist_filename):
            print (f"Memlist file was not found at: {self.memlist_filename}")
            sys.exit(-1)

        memlist = open(self.memlist_filename, "rb")
        mem_entries = []

        i = 0
        while True:
            entry = self.read_mem_entry(memlist, i)
            if entry["bankOffset"] == 0xFFFFFFFF:
                return mem_entries
            mem_entries.append(entry)
            i += 1


    def get_type(self, i):
        res_types = [
            "SOUND",
            "MUSIC",
            "POLY_ANIM",
            "PALETTE",
            "BYTECODE",
            "POLY_CINEM"] # poly_cinematic

        if i < len(res_types):
            return res_types[i]
        else:
            return f"UNKNOWN({i})"


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print (f"usage: {sys.argv[0]} <input_dir> <output_dir>\n")
        sys.exit(-1)

    input_dir = sys.argv[1]
    output_dir = sys.argv[2]

    resources = MSDOSResources(input_dir, output_dir)
    resources.generate()
