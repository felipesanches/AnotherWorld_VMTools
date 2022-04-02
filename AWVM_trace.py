#!/usr/bin/env python3

from exec_trace import ExecTrace
output_dir = ""
game_level = None
cinematic_counter = 0
video2_counter = 0
cinematic_entries = {}
video2_entries = {}
romset_dir = None
str_data = None

VIDEO2=0 # shared polygon resource
CINEMATIC=1 # level-specific bank of polygonal data


def get_text_string(str_id):
    global str_data
    try:
        if not str_data:
            str_data = open(f"{romset_dir}/str_data.rom", "rb").read()

        str_index = open(f"{romset_dir}/str_index.rom", "rb")
        str_index.seek((str_id-1)*2)
        str_index.read(1)
        index = ord(str_index.read(1))
        index = index << 8 | ord(str_index.read(1))
        str_index.close()

        the_string = ""
        while str_data[index] != 0x00:
            c = chr(str_data[index])
            if c == '\n':
                c = "\\n"
            the_string += c
            index += 1
    except:
        the_string = f"string_{str_id}"
    return the_string


# March 30, 2022:
# I've manually checked all the level 2 addresses below
# and they're really never called! Looks like dead-code that
# ended up included in the build of the released bytecode.
# It is not a lot of code, though. So it is probably not any
# significantly big "unreleased" feature.

POSSIBLY_UNUSED_CODEBLOCKS = {
  0: [
      0x007B,
      0x0D1E,
      0x109E
     ],
  1: [
      0x00D2,
      0x1527,
      0x154E,
      0x1BD6
     ],
  2: [
      0x0054,
      0x011A,
      0x017E,
      0x01D1,
      0x01D8,
      0x0480,
      0x05A3,
      0x05D6,
      0x08A1,
      0x0929,
      0x0C6D,
      0x0CFF,
      0x169F,
      0x2F0F,
      0x3564,
      0x385A,
      0x42AA,
      0x42CA,
      0x4460,
      0x4480,
      0x4F37,
      0x4BA2
     ],
  3: [
      0x0043,
      0x016F,
      0x0190,
      0x0691,
      0x06BC,
      0x070B,
      0x0712,
      0x0849,
      0x0AEB,
      0x0B19,
      0x1027,
      0x10FA,
      0x129B,
      0x1963,
      0x2668,
      0x26D8,
      0x282D,
      0x330F,
      0x3325,
      0x3B69,
      0x4210,
      0x4219,
      0x470F,
      0x47D1,
      0x4A6A,
      0x5746,
      0x5754,
      0x5AE2,
      0x652C,
      0x6566,
      0x6595,
      0x65A4,
      0x65D0,
      0x6778,
      0x67BB,
      0x67DD,
      0x68D1,
      0x68E0,
      0x7453,
      0x78C8,
      0x8323,
      0x88DD,
      0x8973,
      0x89FE,
      0x8A0C,
      0x8F43,
      0x8F63,
      0x90DA,
      0x90FA
     ],
  4: [
      0x015D,
      0x106F,
      0x1456,
      0x1518,
      0x1D87,
      0x2250,
      0x25B7,
      0x2617,
      0x27E8,
      0x297E,
      0x2DF4,
      0x30CD,
      0x314D,
      0x3A81,
      0x3F75,
      0x4080,
      0x40BA,
      0x42EE,
      0x42F5,
      0x4799,
      0x4933,
      0x49E6,
      0x4AD1,
      0x4CF5,
      0x4DC9,
      0x4F73,
      0x50BA,
      0x5282,
      0x5BC7,
      0x5E56,
      0x5ED1,
      0x5FCA,
      0x60D7,
      0x60E4,
      0x6A6F,
      0x71A8,
      0x71B1,
      0x7786,
      0x7848,
      0x8765,
      0x877F,
      0x878D,
      0x8A97,
      0x94E8,
      0x9522,
      0x9531,
      0x9551,
      0x9560,
      0x958C,
      0x9734,
      0x9768,
      0x9777,
      0x9799,
      0x987C,
      0x988B,
      0xA105,
      0xA135,
      0xA267,
      0x4B36,
      0xAD4B,
      0xC238,
      0xC279,
      0xC306,
      0xD898,
      0xD8F5,
      0xDD31,
      0xE615,
      0xE7B9,
      0xE957,
      0xE977,
      0xEAEB,
      0xEB0B,
      0xEF44,
      0xF2E0,
      0xF309,
      0xF321
     ],
  5: [
      0x0405,
      0x0A27,
      0x0DCC,
      0x13D9,
      0x1868,
      0x1903,
      0x1DE7,
      0x201A
     ],
  6: [
      0x01DA,
      0x084C,
      0x0BD2,
      0x0C48,
      0x1460,
      0x1A6A,
      0x1BDC,
      0x1E60,
      0x2000,
      0x2371,
      0x269C,
      0x275F,
      0x2803,
      0x2828,
      0x28EF,
      0x2CA5,
      0x2D43,
      0x2DF6,
      0x2E17,
      0x2E3B,
      0x2E6A,
      0x2E7C,
      0x2E82,
      0x2FAC,
      0x3006,
      0x3032,
      0x3039,
      0x319A,
      0x31BF,
      0x31C8,
      0x33E9,
      0x3579,
      0x3ACF,
      0x3BAB,
      0x3C97,
      0x3CD3,
      0x4159,
      0x422D,
      0x43E1,
      0x4532,
      0x46FA,
      0x4D82,
      0x50AB,
      0x536F,
      0x53EA,
      0x5441,
      0x5463,
      0x5476,
      0x54B3,
      0x5572,
      0x5584,
      0x5688,
      0x5695,
      0x5B54,
      0x6050,
      0x678D,
      0x6796,
      0x6D8E,
      0x6E50,
      0x717B,
      0x7D16,
      0x7DA4,
      0x7DB2,
      0x80C8,
      0x8913,
      0x8B66,
      0x8BCD,
      0x8C07,
      0x8C36,
      0x8C45,
      0x8C71,
      0x8E19,
      0x8E4D,
      0x8E5C,
      0x8E7E,
      0x8F61,
      0x8F70,
      0x9712,
      0x9742,
      0x9C30,
      0x9C6A,
      0x9C73,
      0x9DD7,
      0x9DEF,
      0x9E13,
      0x9E53,
      0x9E79,
      0x9EEB,
      0xA02A,
      0xA07A,
      0xA13F,
      0xA2D4,
      0xA600,
      0xA6D6,
      0xA736,
      0xAA8F,
      0xAB67,
      0xB26E,
      0xB27B,
      0xB283,
      0xB5F2,
      0xB60D,
      0xB7F0,
      0xB994,
      0xBB3E,
      0xBB5E,
      0xBCD2,
      0xBCF2,
      0xC113,
      0xC49D
     ],
  7: [
      0x0939,
      0x0AD7
     ],
  8: [
      0x014F,
      0x029D,
      0x03D4,
      0x03E1,
      0x064E,
      0x06C1,
      0x0733,
      0x1078,
      0x1088
     ]
}


LABELED_CINEMATIC_ENTRIES = {
  1: {
    0x0F72: "WALKING_FEET_ARRIVING_0",
    0x0F7E: "WALKING_FEET_ARRIVING_1",
    0x0FA2: "WALKING_FEET_ARRIVING_2",
    0x0FCA: "WALKING_FEET_ARRIVING_3",
    0x0FF2: "WALKING_FEET_ARRIVING_4",
    0x101A: "WALKING_FEET_ARRIVING_5",
    0x1048: "WALKING_FEET_ARRIVING_6",
    0x105E: "WALKING_FEET_ARRIVING_7",
    0x1082: "WALKING_FEET_ARRIVING_8",
    0x10B6: "WALKING_FEET_ARRIVING_9",
    0x10E2: "WALKING_FEET_ARRIVING_10",
    0x10E2: "WALKING_FEET_ARRIVING_11",
    0x110A: "WALKING_FEET_ARRIVING_12",
    0x113E: "WALKING_FEET_ARRIVING_13",
    0x1158: "WALKING_FEET_ARRIVING_14", 
    0x117E: "WALKING_FEET_ARRIVING_15",
    0x11AC: "WALKING_FEET_ARRIVING_16",
    0x11C6: "WALKING_FEET_ARRIVING_17",
    0x1200: "WALKING_FEET_ARRIVING_18",
    0x122A: "WALKING_FEET_ARRIVING_19",
    0x1278: "WALKING_FEET_ARRIVING_20",
    0x1292: "WALKING_FEET_ARRIVING_21",
    0x12AC: "WALKING_FEET_ARRIVING_22",
    0x12F2: "WALKING_FEET_ARRIVING_23",
    0x130C: "WALKING_FEET_ARRIVING_24",
    0x1326: "WALKING_FEET_ARRIVING_25",
    0x1340: "WALKING_FEET_ARRIVING_26",
    0x135A: "WALKING_FEET_ARRIVING_27",
    0x1374: "WALKING_FEET_ARRIVING_28",
    0x13EA: "WALKING_FEET_ARRIVING_29",
    0x1404: "WALKING_FEET_ARRIVING_30",
    0x141E: "WALKING_FEET_ARRIVING_31",
    0x143C: "WALKING_FEET_ARRIVING_32",
    0x145A: "WALKING_FEET_ARRIVING_33",
    0x1478: "WALKING_FEET_ARRIVING_34",
    0x14F6: "WALKING_FEET_ARRIVING_35",
    0x1514: "WALKING_FEET_ENTERING_0",
    0x1532: "WALKING_FEET_ENTERING_1",
    0x1584: "WALKING_FEET_ENTERING_2",
    0x15B6: "WALKING_FEET_ENTERING_3",
    0x15D0: "WALKING_FEET_ENTERING_4",
    0x15EA: "WALKING_FEET_ENTERING_5",
    0x1658: "WALKING_FEET_ENTERING_6",
    0x166A: "WALKING_FEET_ENTERING_7",
    0x16A2: "WALKING_FEET_ENTERING_8",
    0x16EA: "WALKING_FEET_ENTERING_9",
    0x1712: "WALKING_FEET_ENTERING_10",
    0xF6D2: "CARKEY",
    0xF7D8: "DNA_0",
    0xF7EA: "DNA_1",
    0xF7FC: "DNA_2",
    0xF810: "DNA_3",
    0xF822: "DNA_4",
    0xF834: "DNA_5",
    0xF848: "DNA_6",
    0xF85C: "DNA_7",
    0xF870: "DNA_8",
    0xF880: "DNA_9",
    0xF894: "DNA_10",
    0xF8A8: "DNA_11",
    0xF8BC: "DNA_12",
    0xF8D0: "DNA_13",
    0xF8E2: "DNA_14",
    0xF8F4: "DNA_15",
    0xF906: "DNA_16",
  },
  2: {
    0x4D5A: "BEETLE_WALKING_LEFT_0",
    0x4DC6: "BEETLE_WALKING_LEFT_1",
    0x4DE2: "BEETLE_WALKING_LEFT_2",
    0x4DFE: "BEETLE_WALKING_LEFT_3",
    0x4E1E: "BEETLE_WALKING_LEFT_4",
    0x4E3A: "BEETLE_WALKING_LEFT_5",
    0x4E56: "BEETLE_WALKING_LEFT_6",
    0x9ADC: "SLUG_0",
    0x9B04: "SLUG_1",
    0x9B30: "SLUG_2",
    0x9B60: "SLUG_3",
    0x9B90: "SLUG_4",
    0x9BC0: "SLUG_5",
    0x9BF0: "SLUG_6",
    0x9C20: "SLUG_7",
    0x9C50: "SLUG_8",
    0x9C80: "SLUG_9",
    0x9CB0: "SLUG_10",
    0x9CE0: "SLUG_11",
    0x9D10: "SLUG_12",
    0x9D3C: "SLUG_13",
    0x9FBC: "BEETLE_WALKING_RIGHT_0",
    0xA028: "BEETLE_WALKING_RIGHT_1",
    0xA044: "BEETLE_WALKING_RIGHT_2",
    0xA060: "BEETLE_WALKING_RIGHT_3",
    0xA080: "BEETLE_WALKING_RIGHT_4",
    0xA09C: "BEETLE_WALKING_RIGHT_5",
    0xA0B8: "BEETLE_WALKING_RIGHT_6",
  },
}


def register_cinematic_entry(x, y, palette_number, zoom, address):
    global cinematic_counter
    if address in cinematic_entries.keys():
        return cinematic_entries[address]["label"]

    if game_level in LABELED_CINEMATIC_ENTRIES.keys() and address in LABELED_CINEMATIC_ENTRIES[game_level]:
        label = "LEVEL_%d_CINEMATIC_%s" % (game_level, LABELED_CINEMATIC_ENTRIES[game_level][address])
    else:
        label = "LEVEL_%d_CINEMATIC_%03d" % (game_level, cinematic_counter)
    cinematic_counter += 1
    cinematic_entries[address] = {
        'palette_number': palette_number,
        'x': x,
        'y': y,
        'zoom': zoom,
        'label': label
    }
    return label


def register_video2_entry(x, y, palette_number, zoom, address):
    global video2_counter
    if address in video2_entries.keys():
        return video2_entries[address]["label"]

    label = "COMMON_VIDEO_%03d" % (video2_counter)
    video2_counter += 1
    video2_entries[address] = {
        'palette_number': palette_number,
        'x': x,
        'y': y,
        'zoom': zoom,
        'label': label
    }
    return label


def print_video_entries():
    for addr in sorted(cinematic_entries.keys()):
        v = cinematic_entries[addr]
        print (f"label:{v['label']}: CINEMATIC: 0x{addr:04X} x:{v['x']} y:{v['y']} zoom:{v['zoom']}")
    for addr in sorted(video2_entries.keys()):
        v = video2_entries[addr]
        print (f"label:{v['label']}: VIDEO2: 0x{addr:04X} x:{v['x']} y:{v['y']} zoom:{v['zoom']}")


KNOWN_LABELS = {
  1: {
    0x0219: "DNA_ANIMATION",
  }
}

# For now disable these, because they are specific to the msdos release:
LABELED_CINEMATIC_ENTRIES = {}
KNOWN_LABELS = {}
POSSIBLY_UNUSED_CODEBLOCKS = {}
# FIXME: Come up with a way of loading this kind of thing based on the target
#        release of assets being disassembled.


SPECIAL_PURPOSE_VARS = {
    0x3c: "RANDOM_SEED",
    0x54: "HACK_VAR_54",
    0x67: "HACK_VAR_67",
    0xda: "LAST_KEYCHAR",
    0xdc: "HACK_VAR_DC",
    0xe5: "HERO_POS_UP_DOWN",
    0xf4: "MUS_MARK",
    0xf7: "HACK_VAR_F7",
    0xf9: "SCROLL_Y",
    0xfa: "HERO_ACTION",
    0xfb: "HERO_POS_JUMP_DOWN",
    0xfc: "HERO_POS_LEFT_RIGHT",
    0xfd: "HERO_POS_MASK",
    0xfe: "HERO_ACTION_POS_MASK",
    0xff: "PAUSE_SLICES"
}

def getVariableName(value):
    if value in SPECIAL_PURPOSE_VARS.keys():
        return SPECIAL_PURPOSE_VARS[value]
    else:
        return "0x%02X" % value


class AWVM_Trace(ExecTrace):
    def get_label(self, addr):
        if addr in KNOWN_LABELS.get(self.game_level, []):
            return KNOWN_LABELS[self.game_level][addr]
        elif addr in POSSIBLY_UNUSED_CODEBLOCKS.get(self.game_level, []):
            return "JUNK__%04X" % addr
        else:
            return "LABEL_%04X" % addr

    def output_disasm_headers(self):
        header = "; Generated by AnotherWorld_VMTools\n"
        for var in SPECIAL_PURPOSE_VARS.keys():
            name = SPECIAL_PURPOSE_VARS[var]
            header += "%s\t\tEQU 0x%02X\n" % (name, var)

        for addr in cinematic_entries.keys():
            v = cinematic_entries[addr]
            header += "%s\t\tEQU 0x%04X\n" % (v['label'], addr)

        for addr in video2_entries.keys():
            v = video2_entries[addr]
            header += "%s\t\tEQU 0x%04X\n" % (v['label'], addr)
        return header


    def disasm_instruction(self, opcode):
        if (opcode & 0x80) == 0x80:  # VIDEO
            offset = (((opcode & 0x7F) << 8) | self.fetch()) * 2
            x = self.fetch()
            y = self.fetch()
            #print("found video_entry {} at PC={}".format(hex(offset), hex(self.PC)))
            label = register_cinematic_entry(x, y, self.current_palette_number, "0x40", offset)
            return "video type=%d, offset=%s, x=%d, y=%d" % (CINEMATIC, label, x, y)

        elif (opcode & 0x40) == 0x40: # VIDEO
            offset = self.fetch()
            offset = (((offset & 0x7F) << 8) | self.fetch()) * 2

            x_str = ""
            x = self.fetch()
            if not (opcode & 0x20):
                if not (opcode & 0x10):
                    x = (x << 8) | self.fetch()
                    x_str = "%d" % x
                else:
                    x_str = "[0x%02x]" % x
            else:
                if opcode & 0x10:
                    x += 0x100
                x_str = "%d" % x

            y_str = ""
            if not (opcode & 8):
                if not (opcode & 4):
                    y = self.fetch()
                    y = (y << 8) | self.fetch()
                    y_str = "%d" % y
                else:
                    y_str = "[0x%02x]" % self.fetch()
            else:
                y_str = "%d" % self.fetch()

            zoom_str = ""
            if not (opcode & 2):
                if not (opcode & 1):
                    zoom_str = "0x40"
                else:
                    zoom_str = "[0x%02x]" % self.fetch()
            else:
                if opcode & 1:
                    zoom_str = "0x40"
                else:
                    zoom_str = "[0x%02x]" % self.fetch()

            if opcode & 3 == 3:
                label = register_video2_entry(x_str, y_str, self.current_palette_number, zoom_str, offset)
                return "video type=%d, offset=%s, x=%s, y=%s, zoom=%s" % (VIDEO2, label, x_str, y_str, zoom_str)
            else:
                label = register_cinematic_entry(x_str, y_str, self.current_palette_number, zoom_str, offset)
                return "video type=%d, offset=%s, x=%s, y=%s, zoom=%s" % (CINEMATIC, label, x_str, y_str, zoom_str)

        elif opcode == 0x00: # movConst
            dstVar = getVariableName(self.fetch())
            immediate = self.fetch()
            immediate = (immediate << 8) | self.fetch()
            return "mov [%s], 0x%04X" % (dstVar, immediate)

        elif opcode == 0x01: # mov
            dstVar = getVariableName(self.fetch())
            srcVar = getVariableName(self.fetch())
            return "mov [%s], [%s]" % (dstVar, srcVar)

        elif opcode == 0x02: # add
            dstVar = getVariableName(self.fetch())
            srcVar = getVariableName(self.fetch())
            return "add [%s], [%s]" % (dstVar, srcVar)

        elif opcode == 0x03: # addConst
            dstVar = getVariableName(self.fetch())
            immediate = self.fetch()
            immediate = (immediate << 8) | self.fetch()
            if immediate >= 0x8000:
                return "sub [%s], 0x%04X" % (dstVar, 0x10000 - immediate)
            else:
                return "add [%s], 0x%04X" % (dstVar, immediate)

        elif opcode == 0x04: # call
            address = self.fetch()
            address = (address << 8) | self.fetch()
            self.subroutine(address)
            return "call %s" % self.get_label(address)

        elif opcode == 0x05: # ret
            self.return_from_subroutine()
            return "ret"

        elif opcode == 0x06: # break
            return "break"

        elif opcode == 0x07: # jmp
            address = self.fetch()
            address = (address << 8) | self.fetch()
            self.unconditional_jump(address)
            return "jmp %s" % self.get_label(address)

        elif opcode == 0x08: # setVec
            threadId = self.fetch();
            pcOffsetRequested = self.fetch()
            pcOffsetRequested = (pcOffsetRequested << 8) | self.fetch()
            self.schedule_entry_point(pcOffsetRequested, needs_label=True)
            return "setup channel=0x%02X, address=%s" % (threadId, self.get_label(pcOffsetRequested))

        elif opcode == 0x09: # djnz = Decrement and Jump if Not Zero
            var = self.fetch();
            offset = self.fetch()
            offset = (offset << 8) | self.fetch()
            varName = getVariableName(var)
            self.conditional_branch(offset)
            return "djnz [%s], %s" % (varName, self.get_label(offset))

        elif opcode == 0x0a: # Conditional Jump instructions
            subopcode = self.fetch()
            b = self.fetch()
            c = self.fetch()
            var1Str = getVariableName(b)

            if subopcode & 0x80:
                var2Str = getVariableName(c)
                midterm = "[%s]" % var2Str
                offset = self.fetch()
                offset = (offset << 8) | self.fetch()
            elif subopcode & 0x40:
                midterm = "0x%04X" % ((c << 8) | self.fetch());
                offset = self.fetch()
                offset = (offset << 8) | self.fetch();
            else:
                midterm = "0x%02X" % c
                offset = self.fetch()
                offset = (offset << 8) | self.fetch();

            condition = subopcode & 7
            line = ""
            if condition == 0: # jz
                line = "je [%s]" % var1Str
            elif condition == 1: # jnz
                line = "jne [%s]" % var1Str
            elif condition == 2: # jg
                line = "jg [%s]" % var1Str
            elif condition == 3: # jge
                line = "jge [%s]" % var1Str
            elif condition == 4: # jl
                line = "jl [%s]" % var1Str
            elif condition == 5: # jle
                line = "jle [%s]" % var1Str
            else:
                return "; DISASM ERROR! Conditional JMP instruction with invalid condition (%d)" % condition

            self.conditional_branch(offset)
            line += ", %s, %s" % (midterm, self.get_label(offset))
            return line

        elif opcode == 0x0b: # setPalette
            paletteId = self.fetch()
            self.fetch() # waste a byte...
            self.current_palette_number = paletteId
            return "setPalette 0x%02X" % paletteId

        elif opcode == 0x0c: # freezeChannel
            first = self.fetch()
            last = self.fetch()
            type = self.fetch()
            operation_names = [
              "freezeChannels",
              "unfreezeChannels",
              "deleteChannels"
            ]

            if type > 2:
                return "< invalid operation type for resetThread opcode >"
            else:
                return "%s first=0x%02X, last=0x%02X" % (operation_names[type], first, last)

        elif opcode == 0x0d: # selectVideoPage
            frameBufferId = self.fetch()
            return "selectVideoPage 0x%02X" % frameBufferId

        elif opcode == 0x0e: # fillVideoPage
            pageId = self.fetch()
            color = self.fetch()
            return "fill page=0x%02X, color=0x%02X" % (pageId, color)

        elif opcode == 0x0f: # copyVideoPage
            srcPageId = self.fetch()
            dstPageId = self.fetch()
            return "copyVideoPage src=0x%02X, dst=0x%02X" % (srcPageId, dstPageId)

        elif opcode == 0x10: # blitFrameBuffer
            pageId = self.fetch()
            return "blitFramebuffer 0x%02X" % pageId

        elif opcode == 0x11: # killChannel
            return "killChannel"

        elif opcode == 0x12: # text
            stringId = self.fetch()
            stringId = (stringId << 8) | self.fetch()
            x = self.fetch()
            y = self.fetch()
            color = self.fetch()
            text_string = get_text_string(stringId)
            return "text id=0x%04X, x=%d, y=%d, color=0x%02X ; \"%s\"" % (stringId, x, y, color, text_string)

        elif opcode == 0x13: # sub
            var1Str = getVariableName(self.fetch())
            var2Str = getVariableName(self.fetch())
            return "sub [%s], [%s]" % (var1Str, var2Str)

        elif opcode == 0x14: # and
            dstVar = getVariableName(self.fetch())
            immediate = self.fetch()
            immediate = (immediate << 8) | self.fetch()
            return "and [%s], 0x%04X" % (dstVar, immediate)

        elif opcode == 0x15: # or
            dstVar = getVariableName(self.fetch())
            immediate = self.fetch()
            immediate = (immediate << 8) | self.fetch()
            return "or [%s], 0x%04X" % (dstVar, immediate)

        elif opcode == 0x16: # shift left
            variableId = self.fetch()
            leftShiftValue = self.fetch()
            leftShiftValue = (leftShiftValue << 8) | self.fetch()
            varStr = getVariableName(variableId)
            return "shl [%s], 0x%04X" % (varStr, leftShiftValue)

        elif opcode == 0x17: # shift right
            variableId = self.fetch()
            rightShiftValue = self.fetch()
            rightShiftValue = (rightShiftValue << 8) | self.fetch()
            varStr = getVariableName(variableId)
            return "shr [%s], 0x%04X" % (varStr, rightShiftValue)

        elif opcode == 0x18: # play
            resourceId = self.fetch()
            resourceId = (resourceId << 8) | self.fetch()
            freq = self.fetch()
            vol = self.fetch()
            channel = self.fetch()
            return "play id=0x%04X, freq=0x%02X, vol=0x%02X, channel=0x%02X" % (resourceId, freq, vol, channel)

        elif opcode == 0x19: # load
            immediate = self.fetch()
            immediate = (immediate << 8) | self.fetch()
            return "load id=0x%04X" % immediate

        elif opcode == 0x1a: # song
            resNum = self.fetch()
            resNum = (resNum << 8) | self.fetch()
            delay = self.fetch()
            delay = (delay << 8) | self.fetch()
            pos = self.fetch()
            return "song id=0x%04X, delay=0x%04X, pos=0x%02X" % (resNum, delay, pos)

        elif opcode == 0x1b: # gameover on SEGA Genesis
            return "GameOver"

        else:
            self.illegal_instruction(opcode)
            return "; DISASM ERROR! Illegal instruction (opcode = 0x%02X)" % opcode


used_pdata = []
def visited_pdata(addr):
    """ Keeps track of all polygon data byte addresses that are
        read during extraction of the artwork data."""
    global used_pdata
    if addr not in used_pdata:
        used_pdata.append(addr)


def print_unused_polygon_data():
    """Prints which ranges of the polygon data have
       not been ever used in the bytecode."""
    max_addr = sorted(used_pdata)[-1]
    state = 0
    for addr in xrange(max_addr+1):
        if addr not in used_pdata:
            if state == 0:
                start = addr
                state = 1
            else:
                end = addr
        else:
            if state == 1:
                print (f"{hex(start)}-{hex(end)} ({(end-start+1)})")
                state = 0
    if state == 1:
        end = max_addr
        print (f"{hex(start)}-{hex(end)} ({(end-start+1)})")


def makedir(path):
    import os
    if not os.path.exists(path):
        os.mkdir(path)


import sys
if len(sys.argv) != 3:
    print(f"usage: {sys.argv[0]} <romset_dir> <disasm_output_dir>")
else:
    romset_dir = sys.argv[1]
    output_dir = sys.argv[2]
    gamerom = f"{romset_dir}/bytecode.rom"
    makedir(output_dir)
    
    from decode_polygons import PolygonDecoder
    pd = PolygonDecoder(romset_dir,
                        output_dir)
    
    for game_level in range(9):
        print (f"disassembling level {game_level}...")
        trace = AWVM_Trace(gamerom, rombank=0x10000*game_level, loglevel=0)
        trace.game_level = game_level # TODO: pass this to the constructor
        trace.pending_labeled_entry_points = POSSIBLY_UNUSED_CODEBLOCKS.get(game_level, []).copy()
        trace.current_palette_number = 0
        trace.run()
#        trace.print_ranges()
#        trace.print_grouped_ranges()
#        print_video_entries()

        level_path = f"{output_dir}/level_{game_level}"
        makedir(level_path)
        trace.save_disassembly_listing(f"{level_path}/level-{game_level}.asm")
        print (f"\t{len(cinematic_entries.keys())} cinematic entries.")
        # cinematic polygon data:
        used_pdata = []
        pd.extract_polygon_data(game_level, cinematic_entries, cinematic=True)
        cinematic_entries = {}
        cinematic_counter = 0
        # print_unused_polygon_data()

    # common polygon data:
    print (f"\t{len(video2_entries.keys())} video2 entries.")
    pd.extract_polygon_data(game_level, video2_entries, cinematic=False)


