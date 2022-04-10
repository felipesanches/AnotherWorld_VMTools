#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 2 or later

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
        self.raw_data_size = self.READ_BE_UINT32()
        self.output_idx = self.raw_data_size - 1
        self.crc = self.READ_BE_UINT32()
        self.chk = self.READ_BE_UINT32()
        self.crc ^= self.chk
        while True:
            if self.nextChunk():
                c = self.getCode(2)
                if c == 0:
                    # 3 bytes from up to 511 memory positions away
                    # encoded in 12 bits (compression = 12/24 = 50%)
                    self.copy_data(count = 3,
                                   offset = self.getCode(9))
                elif c == 1:
                    # 4 bytes from up to 1023 memory positions away
                    # encoded in 13 bits (compression = 13/32 = approx. 40.6%)
                    self.copy_data(count = 4,
                                   offset = self.getCode(10))
                elif c == 2:
                    # Not useful for less than 5 bytes.
                    #
                    # min: 1 + 4   = 5 bytes
                    # max: 1 + 255 = 256 bytes
                    # encoded in 23 bits
                    # (max-compression = 23/2048 = approx. 1.12%)
                    # (min-compression = 23/40 = approx. 57.5%)
                    self.copy_data(count = 1 + self.getCode(8),
                                   offset = self.getCode(12))
                elif c == 3:
                    # min: 9 + 0   = 9 raw byte
                    # max: 9 + 255 = 264 raw bytes
                    # overhead of 11 bits
                    self.raw_bytes(count = 9 + self.getCode(8))
            else:
                if self.nextChunk():
                    # 2 bytes from up to 255 memory positions away
                    # encoded in 10 bits (compresison = 10/16 = approx. 62.5%)
                    self.copy_data(count = 2,
                                   offset = self.getCode(8))
                else:
                    # min: 1 + 0 = 1 raw byte
                    # max: 1 + 7 = 8 raw bytes
                    # overhead of 4 bits
                    self.raw_bytes(count = 1 + self.getCode(3))

            if self.raw_data_size <= 0:
                return self.crc == 0


    def raw_bytes(self, count):
        self.raw_data_size -= count
        for _ in range(count):
            value = self.getCode(8)
            self.buffer.seek(self.output_idx)
            self.buffer.write(bytes([value]))
            self.output_idx -= 1


    def copy_data(self, count, offset):
        self.raw_data_size -= count;
        for _ in range(count):
            self.buffer.seek(self.output_idx + offset)
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
