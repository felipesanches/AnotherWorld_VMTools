#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 3 or later

import os
import sys
from exectrace import ExecTrace
from releases.common_data.decode_polygons import PolygonDecoder


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
    if not str_data:
        str_data = open(f"{romset_dir}/str_data.rom", "rb").read()

    str_index = open(f"{romset_dir}/str_index.rom", "rb")
    str_index.seek((str_id)*2)
    index = ord(str_index.read(1))
    index = index | (ord(str_index.read(1)) << 8)
    str_index.close()

    if index == 0:
        the_string = f"string_{str_id:04X}"
    else:
        the_string = ""
        while str_data[index] != 0x00:
            c = chr(str_data[index])
            if c == '\n':
                c = "\\n"
            the_string += c
            index += 1

    # Note: I think that the string with index 0x01FA, referenced by msdos level #2,
    #       fails to load because it was an asset of the demo version.
    #       Once we implement extracting strings from ANOTHER.EXE, this should be sorted out.

    return the_string


def register_cinematic_entry(x, y, palette_number, zoom, address):
    global cinematic_counter
    if address in cinematic_entries.keys():
        return cinematic_entries[address]["label"]

    if game_level in LABELED_CINEMATIC_ENTRIES.keys() and address in LABELED_CINEMATIC_ENTRIES[game_level]:
        label = "CINEMATIC_%s" % (LABELED_CINEMATIC_ENTRIES[game_level][address])
    else:
        label = "CINEMATIC_%03d" % (cinematic_counter)
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


# TODO:
#       Add to ExecTrace a mechanism for declaring variable names
#       For most CPUs it would mean giving labels for RAM addresses.
#       For the Another World VM it would be a way to give names to
#       the VM vars, which includes the special purpose ones listed
#       below, but would also include level-specific variables.
#
#       For instance, on level #2 (the game's first stage),
#       variable 0x2A seems to be used to keep track of the sequencing
#       of events/scenes throughout the stage. So that the beast only
#       up once in the background of the first screen. And then only
#       on the second screen, and so on...
#       For that reason, var 0x2A on stage 2 could be called
#       something like "CURRENT_SCENE".
#
#       Also, a good name for var 0x66 on stage 2 seems to be
#       "CURRENTLY_CACHED_RENDERING_OF_SCENARIO_BACKGROUND".
#
#       var 0x01 = LESTER_X_COORDINATE (min: 0, max:320)
#       And I guess var 0x02 is LESTER_Y_COORDINATE, but I still need
#       to double check that interpretation, as well as the min/max values.
#
#       Other variables, though, seem to be reused, having different
#       meanings across the bytecode of a single stage.
#       That would be a bit trickier to document for a nice dissembly,
#       because it would require some mechanism for defining variable
#       scopes. 
#
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
    def getLabelName(self, addr):
        self.register_label(addr)
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
            return "call %s" % self.getLabelName(address)

        elif opcode == 0x05: # ret
            self.return_from_subroutine()
            return "ret"

        elif opcode == 0x06: # break
            return "break"

        elif opcode == 0x07: # jmp
            address = self.fetch()
            address = (address << 8) | self.fetch()
            self.unconditional_jump(address)
            return "jmp %s" % self.getLabelName(address)

        elif opcode == 0x08: # setVec
            threadId = self.fetch();
            pcOffsetRequested = self.fetch()
            pcOffsetRequested = (pcOffsetRequested << 8) | self.fetch()
            self.schedule_entry_point(pcOffsetRequested, needs_label=True)
            return "setup channel=0x%02X, address=%s" % (threadId, self.getLabelName(pcOffsetRequested))

        elif opcode == 0x09: # djnz = Decrement and Jump if Not Zero
            var = self.fetch();
            offset = self.fetch()
            offset = (offset << 8) | self.fetch()
            varName = getVariableName(var)
            self.conditional_branch(offset)
            return "djnz [%s], %s" % (varName, self.getLabelName(offset))

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
            line += ", %s, %s" % (midterm, self.getLabelName(offset))
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
            if (immediate > 0x100) and ((immediate & 0xf) < len(STAGE_TITLES)):
                if immediate & 0xfff0 != 0x3E80:
                    print(f"WARN: Found an instance of the load instruction indicating"
                          f" a bankSwitch but with an uncommon value of {immediate:04X}"
                          f" in its operands.\n"
                          f"Expected to see {0x3E80 | (immediate & 0xf):04X} instead.")
                return "bankSwitch %d;  %s" % (immediate & 0xf, STAGE_TITLES[immediate & 0xf])
            else:
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


def makedir(path):
    if not os.path.exists(path):
        os.mkdir(path)


if len(sys.argv) not in [2, 3]:
    print(f"usage: {sys.argv[0]} <input_dir> [release-name]")
else:
    if len(sys.argv) == 3:
        release_name = sys.argv[2]
        try:
            data = __import__(name=f"releases.{release_name}",
                              fromlist=["MD5_CHECKSUMS",
                                        "STAGE_TITLES",
                                        "LABELED_CINEMATIC_ENTRIES",
                                        "POSSIBLY_UNUSED_CODEBLOCKS",
                                        "KNOWN_LABELS",
                                        "generate_romset"])
            MD5_CHECKSUMS=data.MD5_CHECKSUMS
            STAGE_TITLES=data.STAGE_TITLES
            LABELED_CINEMATIC_ENTRIES = data.LABELED_CINEMATIC_ENTRIES
            KNOWN_LABELS = data.KNOWN_LABELS
            POSSIBLY_UNUSED_CODEBLOCKS = data.POSSIBLY_UNUSED_CODEBLOCKS
        except ImportError:
            sys.exit(f"ERROR: Unrecognized release name '{release_name}'")

        print(f"\n=== {release_name} ===")

        output_dir = f"{os.getcwd()}/output"
        disasm_dir = f"{output_dir}/{release_name}/disasm"
        romset_dir = f"{output_dir}/{release_name}/romset"
        data.generate_romset(input_dir=sys.argv[1],
                             output_dir=output_dir)
    else:
        romset_dir = sys.argv[1]
        disasm_dir = f"{romset_dir}/output"
        LABELED_CINEMATIC_ENTRIES = {}
        KNOWN_LABELS = {}
        POSSIBLY_UNUSED_CODEBLOCKS = {}

    gamerom = f"{romset_dir}/bytecode.rom"
    makedir(disasm_dir)

    pd = PolygonDecoder(romset_dir,
                        disasm_dir)

    num_levels = int(os.path.getsize(gamerom) / 0x10000)
    print(f"Num. levels = {num_levels}")
    for game_level in range(num_levels):
        print(f"disassembling level {game_level}...")
        RELOCATION_BLOCKS = (
            # physical,           logical, length 
            (0x10000*game_level,  0x00000, 0x10000),
        )
        trace = AWVM_Trace(gamerom,
                           relocation_blocks=RELOCATION_BLOCKS,
                           subroutines=POSSIBLY_UNUSED_CODEBLOCKS.get(game_level, []).copy(),
                           labels=KNOWN_LABELS.get(game_level, {}).copy(),
                           loglevel=0)
        trace.game_level = game_level # TODO: pass this to the constructor
        trace.current_palette_number = 0
        trace.run()
        #trace.print_ranges()
        #trace.print_grouped_ranges()
        #print_video_entries()

        level_path = f"{disasm_dir}/level_{game_level}"
        makedir(level_path)
        trace.save_disassembly_listing(f"{level_path}/{release_name}_level-{game_level}.asm")
        print(f"\t{len(cinematic_entries.keys())} cinematic entries.")

        # cinematic polygon data:
        pd.used_pdata = []
        pd.extract_polygon_data(game_level, cinematic_entries, cinematic=True)
        #pd.print_unused_polygon_data()

        cinematic_entries = {}
        cinematic_counter = 0

    # common polygon data:
    print (f"\t{len(video2_entries.keys())} video2 entries.")
    pd.used_pdata = []
    pd.extract_polygon_data(game_level, video2_entries, cinematic=False)
    #pd.print_unused_polygon_data()

