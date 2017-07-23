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

from io import BytesIO
class Unpacker():
  def __init__(self, buf):
    self.buffer = BytesIO(buf)

  def READ_BE_UINT32(self):
    self.buffer.seek(self.index)
    self.index -= 4
    v = ord(self.buffer.read(1)) << 24
    v |= ord(self.buffer.read(1)) << 16
    v |= ord(self.buffer.read(1)) << 8
    v |= ord(self.buffer.read(1))
    return v

  def unpack(self, packedSize):
    self.index = packedSize - 4
    self.size = 0
    self.datasize = self.READ_BE_UINT32()
    self.output_idx = self.datasize - 1
    self.crc = self.READ_BE_UINT32()
    self.chk = self.READ_BE_UINT32()
    self.crc ^= self.chk
    while True:
      if not self.nextChunk():
        self.size = 1
        if not self.nextChunk():
          self.decUnk1(3, 0)
        else:
          self.decUnk2(8)
      else:
        c = self.getCode(2)
        if c == 3:
          self.decUnk1(8, 8)
        else:
          if c < 2:
            self.size = c + 2
            self.decUnk2(c + 9)
          else:
            self.size = self.getCode(8)
            self.decUnk2(12)
      if self.datasize <= 0:
        break
    return self.crc == 0

  def decUnk1(self, numChunks, addCount):
    count = self.getCode(numChunks) + addCount + 1
    self.datasize -= count
    while count:
      count -= 1
      self.buffer.seek(self.output_idx)
      self.buffer.write(chr(self.getCode(8)))
      self.output_idx -= 1

  def decUnk2(self, numChunks):
    self.index = self.getCode(numChunks)
    count = self.size + 1
    self.datasize -= count;
    while count:
      count -= 1
      self.buffer.seek(self.output_idx + self.index)
      value = self.buffer.read(1)
      self.buffer.seek(self.output_idx)
      self.buffer.write(value)
      self.output_idx -= 1

  def getCode(self, numChunks):
    c = 0
    while numChunks:
      numChunks -= 1
      c <<= 1
      if self.nextChunk():
        c |= 1
    return c

  def nextChunk(self):
    CF = self.rcr(False)
    if self.chk == 0:
      self.chk = self.READ_BE_UINT32()
      self.crc ^= self.chk
      CF = self.rcr(True)
    return CF

  def rcr(self, CF):
    rCF = self.chk & 1
    self.chk >>= 1
    if (CF):
      self.chk |= 0x80000000
    return rCF


def main():
  if len(sys.argv) != 3:
    print ("usage: {} <memlist.bin> <output_dir>\n".format(sys.argv[0]))
    sys.exit(-1)

  memlist_filename = sys.argv[1]
  output_dir = sys.argv[2]
  if not os.path.exists(output_dir):
    os.mkdir(output_dir)

  directory = os.path.dirname(memlist_filename)
  entries = read_msdos_memlist(memlist_filename)
  for resource_index, entry in enumerate(entries):
    print ("id:{}\ttype:{}"
           "\toffset:{}\tsize:{}/{}").format(entry["bankId"],
                                           entry["type"],
                                           hex(entry["bankOffset"]),
                                           entry["packedSize"],
                                           entry["size"])
    bank = open(os.path.join(directory, "bank%02x" % entry["bankId"]))
    bank.seek(entry["bankOffset"])
    data = bank.read(entry["packedSize"])
    if entry["packedSize"] != entry["size"]:
      data = Unpacker(data).unpack(entry["packedSize"])

    if len(data) > 0:
      open(os.path.join(output_dir, "resource-0x%02.bin" % resource_index)).write(data)
    bank.close()


if __name__ == "__main__":
  main()
