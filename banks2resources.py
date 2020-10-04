#!/usr/bin/env python3

import os
import sys
from Unpacker import Unpacker

def read_byte(m):
  return ord(m.read(1))

def read_word(m):
  v = read_byte(m)
  v = v << 8 | read_byte(m)
  return v

def read_quad(m):
  v = read_word(m)
  v = v << 16 | read_word(m)
  return v

def read_mem_entry(memlist, n):
  memlist.seek(20*n)
  entry = {}
  entry["size"] = read_byte(memlist)
  entry["type"] = read_byte(memlist)
  entry["unknown_0x02"] = read_word(memlist) # unknown
  entry["unknown_0x04"] = read_word(memlist) # unknown
  entry["rankNum"] = read_byte(memlist)
  entry["bankId"] = read_byte(memlist)
  entry["bankOffset"] = read_quad(memlist)
  entry["unknown_0x0C"] = read_word(memlist) # unknown
  entry["packedSize"] = read_word(memlist)
  entry["unknown_0x10"] = read_word(memlist) # unknown
  entry["size"] = read_word(memlist)
  return entry


def read_msdos_memlist(memlist_filename):
  if not os.path.exists(memlist_filename):
    print (f"Memlist file was not found at: {memlist_filename}")
    sys.exit(-1)

  memlist = open(memlist_filename, "rb")
  mem_entries = []

  i = 0
  while True:
    entry = read_mem_entry(memlist, i)
    if entry["bankOffset"] == 0xFFFFFFFF:
      return mem_entries
    i += 1
    if entry["packedSize"] != 0:
      mem_entries.append(entry)


def main():
  if len(sys.argv) != 3:
    print (f"usage: {sys.argv[0]} <memlist.bin> <output_dir>\n")
    sys.exit(-1)

  memlist_filename = sys.argv[1]
  output_dir = sys.argv[2]
  if not os.path.exists(output_dir):
    os.mkdir(output_dir)

  directory = os.path.dirname(memlist_filename)
  entries = read_msdos_memlist(memlist_filename)
  for resource_index, entry in enumerate(entries):
    if entry != None:
      print (f"id:{entry['bankId']}\ttype:{entry['type']}"
             f"\toffset:{hex(entry['bankOffset'])}"
             f"\tsize:{entry['packedSize']}/{entry['size']}")

    bank_file = os.path.join(directory, "bank%02x" % entry["bankId"])
    bank = open(bank_file, "rb")
    bank.seek(entry["bankOffset"])
    data = bank.read(entry["packedSize"])
    if entry["packedSize"] != entry["size"]:
      data = Unpacker(data).unpack(entry["packedSize"])

    if len(data) > 0:
      open(os.path.join(output_dir, "resource-0x%02.bin" % resource_index)).write(data)
    bank.close()


if __name__ == "__main__":
  main()
