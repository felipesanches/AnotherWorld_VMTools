#!/usr/bin/env python
import os
import sys

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
    print ("Memlist file was not found at: '{}'".format(memlist_filename))
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
  if len(sys.argv) != 2:
    print ("usage: {} <memlist.bin>\n".format(sys.argv[0]))
    sys.exit(-1)

  memlist_filename = sys.argv[1]
  entries = read_msdos_memlist(memlist_filename)
  for entry in entries:
    print ("id:{}\ttype:{}"
           "\toffset:{}\tsize:{}/{}").format(entry["bankId"],
                                           entry["type"],
                                           hex(entry["bankOffset"]),
                                           entry["packedSize"],
                                           entry["size"])

if __name__ == "__main__":
  main()
