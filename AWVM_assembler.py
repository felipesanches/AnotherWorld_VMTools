#!/usr/bin/env python3
symbols = {}
rom = False
address = 0
second_pass = False
VIDEO2=0 # shared polygon resource
CINEMATIC=1 # level-specific bank of polygonal data

def parse_value(v_str):
    try:
        if v_str.startswith("0x"):
            return int(v_str.split("0x")[1], 16)
        else:
            return int(v_str)
    except:
#    print (f"not a number! ({v_str})")
        return v_str

def parse_operand(operand_str):
    operand_str = operand_str.strip()
    key = None
    if "=" in operand_str:
        key, operand_str = operand_str.split("=")

    if '[' in operand_str and ']' in operand_str:
        value = operand_str.split('[')[1].strip()
        value = value.split(']')[0].strip()
        return {"type": "var",
                "key": key,
                "value": parse_value(value)}
    else:
        return {"type": "value",
                "key": key,
                "value": parse_value(operand_str)}

def parse_label(line_str):
    return line_str.split(":")[0]

def parse_common(name, line):
    retval = {"name": name}
    line = line.split(name)[1].strip()
    tokens = line.split(",")
    retval["operands"] = [parse_operand(t) for t in tokens]
    return retval

def byte(v):
    global address
    rom.seek(address)
    address += 1
    if isinstance(v, str):
        if v in symbols:
            v = symbols[v]
        else:
            v = 0
    rom.write(bytes([v]))

def word(v, negative=False):
    if v in symbols:
        v = symbols[v]

    # we need to do it here, right after
    # resolving symbolic names above.
    if negative:
        v = 0x10000 - v

    try:
        byte(v >> 8)
        byte(v & 0xFF)
    except:
        word(0x0000)
        if second_pass:
            print (f"Symbol '{v}' could not be found.")

def encode(instr):
    if instr["name"] == "db":
        for data_byte in instr["operands"]:
            byte(data_byte["value"])

    if instr["name"] == "mov":
        dest, data = instr["operands"]
        if data["type"] == "var":
            byte(0x01)
            byte(dest["value"])
            byte(data["value"])
        else: #value
            byte(0x00)

            byte(dest["value"])
            word(data["value"])

    elif instr["name"] == "add":
        dest, data = instr["operands"]
        if data["type"] == "var":
            byte(0x02)
            byte(dest["value"])
            byte(data["value"])
        else: #value
            byte(0x03)
            byte(dest["value"])
            word(data["value"])

    elif instr["name"] == "sub":
        dest, src = instr["operands"]
        if src["type"] == "var":
            byte(0x13)
            byte(dest["value"])
            byte(src["value"])
        else:
            # here we actually emit an add instruction with the
            # second operand multiplied by -1 and represented in
            # two's complement notation
            byte(0x03)
            byte(dest["value"])
            word(src["value"], negative=True)

    elif instr["name"] == "and":
        dest, src = instr["operands"]
        if src["type"] != "value":
            print ("ERROR: 'AND' instruction second operand must be an immediate 16bit value.")
        byte(0x14)
        byte(dest["value"])
        word(src["value"])

    elif instr["name"] == "or":
        dest, src = instr["operands"]
        if src["type"] != "value":
            print ("ERROR: 'OR' instruction second operand must be an immediate 16bit value.")
        byte(0x15)
        byte(dest["value"])
        word(src["value"])

    elif instr["name"] == "shl":
        dest, src = instr["operands"]
        if src["type"] != "value":
            print ("ERROR: 'SHL' instruction second operand must be an immediate 16bit value.")
        byte(0x16)
        byte(dest["value"])
        word(src["value"])

    elif instr["name"] == "shr":
        dest, src = instr["operands"]
        if src["type"] != "value":
            print ("ERROR: 'SHR' instruction second operand must be an immediate 16bit value.")
        byte(0x17)
        byte(dest["value"])
        word(src["value"])

    elif instr["name"] == "jmp":
        addr = instr["operands"][0]
        byte(0x07)
        word(addr["value"])

    elif instr["name"] == "call":
        addr = instr["operands"][0]
        byte(0x04)
        word(addr["value"])

    elif instr["name"] == "ret":
        byte(0x05)

    elif instr["name"] == "killChannel":
        byte(0x11)

    elif instr["name"] == "break":
        byte(0x06)

    elif instr["name"] == "text":
        ops = keyword_operands(instr)
        byte(0x12)
        word(ops["id"]["value"])
        byte(ops["x"]["value"])
        byte(ops["y"]["value"])
        byte(ops["color"]["value"])

    elif instr["name"] == "play":
        ops = keyword_operands(instr)
        byte(0x18)
        word(ops["id"]["value"])
        byte(ops["freq"]["value"])
        byte(ops["vol"]["value"])
        byte(ops["channel"]["value"])

    elif instr["name"] == "song":
        ops = keyword_operands(instr)
        byte(0x1A)
        word(ops["id"]["value"])
        word(ops["delay"]["value"])
        byte(ops["pos"]["value"])

    elif instr["name"] == "freezeChannels":
        first, last = instr["operands"]
        byte(0x0C)
        byte(first["value"])
        byte(last["value"])
        byte(0x00)

    elif instr["name"] == "unfreezeChannels":
        first, last = instr["operands"]
        byte(0x0C)
        byte(first["value"])
        byte(last["value"])
        byte(0x01)

    elif instr["name"] == "deleteChannels":
        first, last = instr["operands"]
        byte(0x0C)
        byte(first["value"])
        byte(last["value"])
        byte(0x02)

    elif instr["name"] == "djnz":
        var, address = instr["operands"]
        byte(0x09)
        byte(var["value"])
        word(address["value"])

    elif instr["name"] in ["je", "jne", "jg", "jge", "jl", "jle"]:
        b, c, addr = instr["operands"]
        subopcode = ["je", "jne", "jg", "jge", "jl", "jle"].index(instr["name"])
        if c["type"] == "var":
            subopcode |= 0x80
        elif c["value"] > 0xFF:
            subopcode |= 0x40

        byte(0x0A)
        byte(subopcode)
        byte(b["value"])
        if c["type"] == "value" and c["value"] > 0xFF:
            word(c["value"])
        else:
            byte(c["value"])
        word(addr["value"])

    elif instr["name"] == "setPalette":
        paletteIndex = instr["operands"][0]
        byte(0x0B)
        word(paletteIndex["value"] << 8 | 0xFF)

    elif instr["name"] == "load":
        ops = keyword_operands(instr)
        byte(0x19)
        word(ops["id"]["value"])

    elif instr["name"] == "selectVideoPage":
        pageId = instr["operands"][0]
        byte(0x0D)
        byte(pageId["value"])

    elif instr["name"] == "copyVideoPage":
        ops = keyword_operands(instr)
        byte(0x0F)
        byte(ops["src"]["value"])
        byte(ops["dst"]["value"])

    elif instr["name"] == "blitFramebuffer":
        pageId = instr["operands"][0]
        byte(0x10)
        byte(pageId["value"])

    elif instr["name"] == "fill":
        ops = keyword_operands(instr)
        byte(0x0E)
        byte(ops["page"]["value"])
        byte(ops["color"]["value"])

    elif instr["name"] == "setup":
        ops = keyword_operands(instr)
        byte(0x08)
        byte(ops["channel"]["value"])
        word(ops["address"]["value"])

    elif instr["name"] == "video":
        ops = keyword_operands(instr)

        offs = ops["offset"]["value"]
        if offs in symbols:
            offs = symbols[offs]
        else:
            offs = 0x0000

        x = ops["x"]
        y = ops["y"]
        if "zoom" not in ops:
            word(0x8000 | ((offs >> 1) & 0x7FFF))
            byte(x["value"])
            byte(y["value"])
            # TODO: error if type is VIDEO2
        else:
            zoom = ops["zoom"]
            opcode = 0x40
            if ops["type"]["value"] == VIDEO2:
                opcode |= 0x03
            operand_bytes = [
              offs >> 9,
              (offs >> 1) & 0xFF
            ]
            if x["type"] == "var":
                operand_bytes.append(x["value"])
                opcode |= 0x10
            else:
                if x["value"] <= 0x1FF:
                    opcode |= 0x20
                    operand_bytes.append(x["value"] & 0xFF)
                    if x["value"] > 0xFF:
                        opcode |= 0x10
                else:
                    operand_bytes.append(x["value"] >> 8)
                    operand_bytes.append(x["value"] & 0xFF)

            if y["type"] == "var":
                operand_bytes.append(y["value"])
                opcode |= 0x04
            else:
                if y["value"] <= 0xFF:
                    opcode |= 0x08
                    operand_bytes.append(y["value"])
                else:
                    operand_bytes.append(y["value"] >> 8)
                    operand_bytes.append(y["value"] & 0xFF)

            if zoom["type"] == "var":
                operand_bytes.append(zoom["value"])
                opcode |= 0x01
            else:
                if zoom["value"] != 0x40:
                    print ("ERROR! Zoom can't be a constant other than 0x40!")

            byte(opcode)
            for b in operand_bytes:
                byte(b)

def keyword_operands(instr):
    ops = {}
    for op in instr["operands"]:
        value = op["value"]
        key = op["key"]
        if key:
            ops[key] = op
    return ops

def assemble(input_filename):
    print (f"\nAssembling '{input_filename}' ...")
    global address, rom, symbols
    instructions = []
    label = None
    output_filename = input_filename.split(".asm")[0] + ".bin"
    rom = open(output_filename, "w+b")
    lines = open(input_filename).readlines()
    for line in lines:
        line = line.split(";")[0].strip() #remove comments
        if "EQU" in line:
            tokens = line.split("EQU")
            symbols[tokens[0].strip()] = parse_value(tokens[1].strip())

        instr = None
        if ":" in line.split(" ")[0]:
            label = parse_label(line)
            line = line.split(":")[1]

        for instruction_name in ["db", "mov", "add", "sub", "jmp",
                                 "call", "ret", "break", "setPalette",
                                 "selectVideoPage", "copyVideoPage",
                                 "blitFramebuffer", "video", "fill",
                                 "je", "jne", "jge", "jg", "jle", "jl",
                                 "load", "setup", "djnz", "freezeChannels",
                                 "unfreezeChannels", "deleteChannels",
                                 "killChannel", "text", "sub", "and", "or",
                                 "shl", "shr", "play", "song"]:
            if line.strip().startswith(instruction_name):
                instr = parse_common(instruction_name, line)
                instructions.append((label, instr))
                label = None
                break

#  print symbols
#  print "\n".join([str(instr) for instr in instructions])

    print ("First Pass.")
    address = 0
#  print "\n".join(map(str, instructions))
    for label, instruction in instructions:
        if label:
            symbols[label] = address
        encode(instruction)

#  print symbols

    print ("Second Pass.")
    second_pass = True
    address = 0
    for label, instruction in instructions:
        if label:
            symbols[label] = address
        encode(instruction)

#  print symbols

import sys
if len(sys.argv) != 2:
    print(f"usage: {sys.argv[0]} input.asm")
    sys.exit(-1)

assemble(sys.argv[1])
