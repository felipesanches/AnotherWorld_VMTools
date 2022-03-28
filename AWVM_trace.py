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
    0x9D3C: "SLUG_13"
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

# TODO: KNOWN LABELS
# level 1, 0x0219 = DNA_ANIMATION
def get_label(addr):
    return "LABEL_%04X" % addr


class AWVM_Trace(ExecTrace):
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
            return "call %s" % get_label(address)

        elif opcode == 0x05: # ret
            self.return_from_subroutine()
            return "ret"

        elif opcode == 0x06: # break
            return "break"

        elif opcode == 0x07: # jmp
            address = self.fetch()
            address = (address << 8) | self.fetch()
            self.unconditional_jump(address)
            return "jmp %s" % get_label(address)

        elif opcode == 0x08: # setVec
            threadId = self.fetch();
            pcOffsetRequested = self.fetch()
            pcOffsetRequested = (pcOffsetRequested << 8) | self.fetch()
            self.schedule_entry_point(pcOffsetRequested)
            return "setup channel=0x%02X, address=%s" % (threadId, get_label(pcOffsetRequested))

        elif opcode == 0x09: # djnz = Decrement and Jump if Not Zero
            var = self.fetch();
            offset = self.fetch()
            offset = (offset << 8) | self.fetch()
            varName = getVariableName(var)
            self.conditional_branch(offset)
            return "djnz [%s], %s" % (varName, get_label(offset))

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
            line += ", %s, %s" % (midterm, get_label(offset))
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
            self.return_from_subroutine()
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
            if immediate > 0x91:
                self.return_from_subroutine()
            return "load id=0x%04X" % immediate

        elif opcode == 0x1a: # song
            resNum = self.fetch()
            resNum = (resNum << 8) | self.fetch()
            delay = self.fetch()
            delay = (delay << 8) | self.fetch()
            pos = self.fetch()
            return "song id=0x%04X, delay=0x%04X, pos=0x%02X" % (resNum, delay, pos)

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


