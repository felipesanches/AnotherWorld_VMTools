#!/usr/bin/env python

verbose = False
debug = False
render_graph = False

video_entries = []
def log_video(x, y, zoom, address):
  if verbose:
    print ("VIDEO at 0x{} x={} y={} zoom={}\n".format(hex(address), x, y, zoom))

  video_entries.append({
    'address': address,
    'x': x,
    'y': y,
    'zoom': zoom
  })

def getVariableName(value):
  if value == 0x3c: return "RANDOM_SEED"
  elif value == 0x54: return "HACK_VAR_54"
  elif value == 0x67: return "HACK_VAR_67"
  elif value == 0xda: return "LAST_KEYCHAR"
  elif value == 0xdc: return "HACK_VAR_DC"
  elif value == 0xe5: return "HERO_POS_UP_DOWN"
  elif value == 0xf4: return "MUS_MARK"
  elif value == 0xf7: return "HACK_VAR_F7"
  elif value == 0xf9: return "SCROLL_Y"
  elif value == 0xfa: return "HERO_ACTION"
  elif value == 0xfb: return "HERO_POS_JUMP_DOWN"
  elif value == 0xfc: return "HERO_POS_LEFT_RIGHT"
  elif value == 0xfd: return "HERO_POS_MASK"
  elif value == 0xfe: return "HERO_ACTION_POS_MASK"
  elif value == 0xff: return "PAUSE_SLICES"
  else:
    return "0x%02X" % value


class CodeBlock():
  ''' A code block represents an address range in
      program memory. The range is specified by
      the self.start and self.end values.

      If a code block ends with a ret (return) instruction,
      then self.next_block will remain an empty list.

      Otherwise, it may have a single-element corresponding
      to a JMP instruction or a couple of values for each of
      the possible execution paths for a conditional branching
      instruction.
  '''

  def __init__(self, start, end, next_block=[]):
    self.start = start
    self.end = end
    self.subroutines = {}
    self.next_block = next_block

  def add_subroutine_call(self, instr_address, routine_address):
    self.subroutines[instr_address] = routine_address


class AWDisasm():
  def __init__(self, romfile):
    self.rom = open(romfile).read()
    self.visited_ranges = []
    self.pending_entry_points = []
    self.current_entry_point = None
    self.PC = None
    self.level_bank = 0


  def already_visited(self, address):
    if self.PC is not None:
      if address >= self.current_entry_point and address < self.PC:
        if debug:
          print ("RECENTLY: (PC={} address={})".format(hex(self.PC), hex(address)))
        return True

    for codeblock in self.visited_ranges:
      if address >= codeblock.start and address <= codeblock.end:
        if debug:
          print ("ALREADY VISITED: {}".format(hex(address)))
        if address > codeblock.start:
          # split the block into two:
          new_block = CodeBlock(start=codeblock.start,
                                end=address-1,
                                next_block=[address])
          codeblock.start = address
          # and also split ownership of subroutine calls:
          for instr_addr, call_addr in codeblock.subroutines.iteritems():
            if instr_addr < address:
              new_block.add_subroutine_call(instr_addr, call_addr)
              del codeblock.subroutines[instr_addr]
          self.visited_ranges.append(new_block)
        return True

    # otherwise:
    return False


  def restart_from_another_entry_point(self):
    if len(self.pending_entry_points) == 0:
      self.PC = None  # This will finish the crawling
    else:
      address = self.pending_entry_points.pop()
      self.current_entry_point = address
      self.PC = address
      if verbose:
        print("Restarting from: {}".format(hex(address)))


  def add_range(self, start, end, exit=None):
    if end < start:
      self.add_range(end, start, exit)
      return

    if debug:
      print("=== New Range: start: {}  end: {} ===".format(hex(start), hex(end)))
    block = CodeBlock(start, end, exit)
    self.visited_ranges.append(block)


  def print_status(self):
    print "Pending: {}".format(map(hex, self.pending_entry_points))
    self.print_ranges()


  def schedule_entry_point(self, address):
    if self.already_visited(address):
      return

    if address not in self.pending_entry_points:
      self.pending_entry_points.append(address)
      if verbose:
        print "SCHEDULING: {}".format(hex(address))
        self.print_status()


  def subroutine(self, address):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[self.PC, address])
    self.schedule_entry_point(self.PC)
    self.schedule_entry_point(address)
    if verbose:
      print "{}: CALL SUBROUTINE ({})".format(hex(self.PC-2), hex(address))
      self.print_status()
    self.restart_from_another_entry_point()


  def return_from_subroutine(self):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[])
    if verbose:
      print("RETURN FROM SUBROUTINE")
      self.print_status()
    self.restart_from_another_entry_point()


  def conditional_branch(self, address):
    if address > self.current_entry_point and address < self.PC:
      self.add_range(start=self.current_entry_point,
                     end=address-1,
                     exit=[address])
      self.add_range(start=address,
                     end=self.PC-1,
                     exit=[self.PC, address])
      self.schedule_entry_point(self.PC)
    else:
      self.add_range(start=self.current_entry_point,
                     end=self.PC-1,
                     exit=[self.PC, address])
      self.schedule_entry_point(self.PC)
      self.schedule_entry_point(address)
    if verbose:
      print ("CONDITIONAL JUMP to {}".format(hex(address)))
      self.print_ranges()
    self.restart_from_another_entry_point()


  def unconditional_jump(self, address):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[address])
    self.schedule_entry_point(address)
    if verbose:
      print ("JUMP to {}".format(hex(address)))
      self.print_ranges()
    self.restart_from_another_entry_point()

  def illegal_instruction(self, opcode):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=["Illegal Opcode: {}".format(hex(opcode))])
    print("[{}] ILLEGAL: {}".format(hex(self.PC-1), hex(opcode)))
    self.restart_from_another_entry_point()


  def increment_PC(self):
    if self.already_visited(self.PC):
      if verbose:
        print("ALREADY BEEN AT {}!".format(hex(self.PC)))
      if debug:
        print("pending_entry_points: {}".format(self.pending_entry_points))
      self.add_range(start=self.current_entry_point,
                     end=self.PC-1,
                     exit=[self.PC])
      self.restart_from_another_entry_point()
    else:
      self.PC += 1


  def fetch(self):
    value = ord(self.rom[self.level_bank << 16 | self.PC])
    if debug:
      print (("Fetch at [{}] {}: {}").format(hex(self.level_bank),
                                             hex(self.PC),
                                             hex(value)))
    self.increment_PC()
    return value

  def disasm_instruction(self):
    opcode = self.fetch()

    if opcode & 0x80 == 0x80:  # VIDEO
      offset = ((opcode << 8) | self.fetch()) * 2
      x = self.fetch()
      y = self.fetch()
      log_video(x, y, "0x40" ,offset)
      return "video: off=0x%X x=%d y=%d" % (offset, x, y)

    elif opcode & 0x40 == 0x40: # VIDEO
      offset = self.fetch()
      offset = ((offset << 8) | self.fetch()) * 2

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

      log_video(x_str, y_str, zoom_str, offset)
      return "video: off=0x%X x=%s y=%s zoom:%s" % (offset, x_str, y_str, zoom_str)

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
      return "add [%s], 0x%04X" % (dstVar, immediate)

    elif opcode == 0x04: # call
      address = self.fetch()
      address = (address << 8) | self.fetch()
      self.subroutine(self.level_bank << 16 | address)
      return "call 0x%04X" % address

    elif opcode == 0x05: # ret
      self.return_from_subroutine()
      return "ret"

    elif opcode == 0x06: # break
      return "break"

    elif opcode == 0x07: # jmp
      address = self.fetch()
      address = (address << 8) | self.fetch()
      self.unconditional_jump(self.level_bank << 16 | address)
      return "jmp 0x%04X" % address

    elif opcode == 0x08: # setVec
      threadId = self.fetch();
      pcOffsetRequested = self.fetch()
      pcOffsetRequested = (pcOffsetRequested << 8) | self.fetch()
      self.schedule_entry_point(self.level_bank << 16 | pcOffsetRequested)
      return "setvec channel:0x%02X, address:0x%04X" % (threadId, pcOffsetRequested)

    elif opcode == 0x09: # djnz = Decrement and Jump if Not Zero
      var = self.fetch();
      offset = self.fetch()
      offset = (offset << 8) | self.fetch()
      varName = getVariableName(var)
      self.conditional_branch(self.level_bank << 16 | offset)
      return "djnz [%s], 0x%04X" % (varName, offset)

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
        return "< conditional jmp with invalid condition: %d >" % condition

      line += ", %s, 0x%04X" % (midterm, offset)
      self.conditional_branch(offset)
      return line

    elif opcode == 0x0b: # setPalette
      paletteId = self.fetch()
      paletteId = (paletteId << 8) | self.fetch()
      return "setPalette 0x%04X" % paletteId

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
        return "%s first:0x%02X, last:0x%02X" % (operation_names[type], first, last)

    elif opcode == 0x0d: # selectVideoPage
      frameBufferId = self.fetch()
      return "selectVideoPage 0x%02X" % frameBufferId

    elif opcode == 0x0e: # fillVideoPage
      pageId = self.fetch()
      color = self.fetch()
      return "fillVideoPage 0x%02X, color:0x%02X" % (pageId, color)

    elif opcode == 0x0f: # copyVideoPage
      srcPageId = self.fetch()
      dstPageId = self.fetch()
      return "copyVideoPage src:0x%02X, dst:0x%02X" % (srcPageId, dstPageId)

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
      return "text id:0x%04X, x:%d, y:%d, color:0x%02X" % (stringId, x, y, color)

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
      return "play id:0x%04X, freq:0x%02X, vol:0x%02X, channel:0x%02X" % (resourceId, freq, vol, channel)

    elif opcode == 0x19: # load
      immediate = self.fetch()
      immediate = (immediate << 8) | self.fetch()
      if immediate > 0x91:
        self.return_from_subroutine()
      return "load id:0x%04X" % immediate

    elif opcode == 0x1a: # song
      resNum = self.fetch()
      resNum = (resNum << 8) | self.fetch()
      delay = self.fetch()
      delay = (delay << 8) | self.fetch()
      pos = self.fetch()
      return "song id:0x%04X, delay:0x%04X, pos:0x%02X" % (resNum, delay, pos)

    else:
      return "ilegal_instruction!"

  def select_memory_bank(self, n):
    print("SELECT MEM-BANK {} !!! (PC={})".format(n, hex(self.PC)))


  def run(self, entry_point=0x0000):
    self.current_entry_point = entry_point
    self.PC = entry_point
    while self.PC is not None:
      PC = self.PC
      line = self.disasm_instruction()
      print("[%02X] %04X: %s" % (self.level_bank, PC, line))


  def print_ranges(self):
    results = []
    for codeblock in sorted(self.visited_ranges, key=lambda cb: cb.start):
      results.append("[start: {}, end: {}]".format(hex(codeblock.start),
                                                   hex(codeblock.end)))
    print ("ranges:\n  " + "\n  ".join(results) + "\n")

  def print_grouped_ranges(self):
    results = []
    current = None
    for codeblock in sorted(self.visited_ranges, key=lambda cb: cb.start):
      if current == None:
        current = [codeblock.start, codeblock.end]
        continue

      # FIX-ME: There's something bad going on here!!!
      if codeblock.start == current[1] or \
         codeblock.start == (current[1] + 1):
        current[1] = codeblock.end
        continue
#      print (">>> codeblock.start: {} current[1]: {}\n".format(hex(codeblock.start),
#                                                               hex(current[1])))

      results.append("[start: {}, end: {}]".format(hex(current[0]),
                                                   hex(current[1])))
      current = [codeblock.start, codeblock.end]

    print ("ranges:\n  " + "\n  ".join(results) + "\n")


def generate_graph():
  def block_name(block):
    return "{}-{}".format(hex(block.start), hex(block.end))

  import pydotplus
  graph = pydotplus.graphviz.Graph(graph_name='AWVM trace',
			   graph_type='digraph',
			   strict=False,
			   suppress_disconnected=False)
  graph_dict = {}
  for block in awdis.visited_ranges:
    node = pydotplus.graphviz.Node(block_name(block))
    graph.add_node(node)
    graph_dict[block.start] = node

  for block in awdis.visited_ranges:
    for nb in block.next_block:
      if nb is str:
	print nb  # this must be an illegal instruction
      else:
	if nb in graph_dict.keys():
	  edge = pydotplus.graphviz.Edge(graph_dict[block.start], graph_dict[nb])
	  graph.add_edge(edge)
	else:
	  print "Missing codeblock: {}".format(hex(nb))

  open("output.gv", "w").write(graph.to_string())

  #from graphviz import Digraph
  #dot = Digraph(comment='The Round Table')
  #dot.render('test-output/round-table.gv', view=True)


import sys
if len(sys.argv) != 2:
  print("usage: {} input.rom".format(sys.argv[0]))
else:
  gamerom = sys.argv[1]
  awdis = AWDisasm(gamerom)
  awdis.run()
#  awdis.print_ranges()

  if render_graph:
    generate_graph()

  awdis.print_grouped_ranges()
