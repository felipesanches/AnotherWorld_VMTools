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
#        print(f"0:__{self.datasize:08X}__")
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
                return self.crc == 0

    def decUnk1(self, numChunks, addCount):
#        print(f"1: {numChunks}_{addCount}")
        count = self.getCode(numChunks) + addCount + 1
        self.datasize -= count
        while count:
            count -= 1
            value = self.getCode(8)
#            print(f"1v: {value}")
            self.buffer.seek(self.output_idx)
            self.buffer.write(bytes([value]))
            self.output_idx -= 1

    def decUnk2(self, numChunks):
#        print(f"2: {numChunks}")
        i = self.getCode(numChunks)
        count = self.size + 1
        self.datasize -= count;
        while count:
            count -= 1
            self.buffer.seek(self.output_idx + i)
            value = self.buffer.read(1)
#            print(f"2v: {value[0]}")
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
