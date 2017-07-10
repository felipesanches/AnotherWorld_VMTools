#!/usr/bin/env python

from exec_trace import ExecTrace

video_entries = {}
def register_video_entry(x, y, zoom, address):
  video_entries[address] = {
    'x': x,
    'y': y,
    'zoom': zoom
  }

video2_entries = {}
def register_video2_entry(x, y, zoom, address):
  video2_entries[address] = {
    'x': x,
    'y': y,
    'zoom': zoom
  }

def print_video_entries():
  for addr in sorted(video_entries.keys()):
    v = video_entries[addr]
    print "0x%04X x:%s y:%s zoom:%s" % (addr, v['x'], v['y'], v['zoom'])

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

class AWVM_Trace(ExecTrace):
  def disasm_instruction(self, opcode):
    if (opcode & 0x80) == 0x80:  # VIDEO
      offset = (((opcode & 0x7F) << 8) | self.fetch()) * 2
      x = self.fetch()
      y = self.fetch()
      print("found video_entry {} at PC={}".format(hex(offset), hex(self.PC)))
      register_video_entry(x, y, "0x40" ,offset)
      return "video: off=0x%X x=%d y=%d" % (offset, x, y)

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
        register_video2_entry(x_str, y_str, zoom_str, offset)
      else:
        print("found video_entry {} at PC={}".format(hex(offset), hex(self.PC)))
        register_video_entry(x_str, y_str, zoom_str, offset)

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
      self.subroutine(address)
      return "call 0x%04X" % address

    elif opcode == 0x05: # ret
      self.return_from_subroutine()
      return "ret"

    elif opcode == 0x06: # break
      return "break"

    elif opcode == 0x07: # jmp
      address = self.fetch()
      address = (address << 8) | self.fetch()
      self.unconditional_jump(address)
      return "jmp 0x%04X" % address

    elif opcode == 0x08: # setVec
      threadId = self.fetch();
      pcOffsetRequested = self.fetch()
      pcOffsetRequested = (pcOffsetRequested << 8) | self.fetch()
      self.schedule_entry_point(pcOffsetRequested)
      return "setvec channel:0x%02X, address:0x%04X" % (threadId, pcOffsetRequested)

    elif opcode == 0x09: # djnz = Decrement and Jump if Not Zero
      var = self.fetch();
      offset = self.fetch()
      offset = (offset << 8) | self.fetch()
      varName = getVariableName(var)
      self.conditional_branch(offset)
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

      self.conditional_branch(offset)
      line += ", %s, 0x%04X" % (midterm, offset)
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
      self.illegal_instruction(opcode)
      return "illegal_instruction!"

game_level = None
polygon_data = None
pdata_offset = 0
def fetch_polygon_data():
  global pdata_offset
  value = ord(polygon_data[game_level << 16 | pdata_offset])
  pdata_offset += 1
  return value

DEFAULT_ZOOM = 0x40
MAX_POINTS = 50
def fillPolygon(c, zoom, color, cx, cy):
  print("    <{}>".format(hex(pdata_offset)))
  bbox_w = fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM;
  bbox_h = fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM;
  numPoints = fetch_polygon_data()
  print("        -> {} points polygon".format(numPoints))

  if not ((numPoints & 1) == 0 and numPoints < MAX_POINTS):
    print "error: numPoints = {}".format(numPoints)
    sys.exit(-1)

  #Read all points, directly from bytecode segment
  for i in range(numPoints):
    x = fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM
    y = fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM
    print ("        {}   x:{} y:{}".format(hex(pdata_offset), x, y))
    if i==0:
      c.move_to(cx - bbox_w/2 + x, cy - bbox_h/2 + y)
    else:
      c.line_to(cx - bbox_w/2 + x, cy - bbox_h/2 + y)

  c.close_path()
  c.set_source_rgb(0, 0, 0) # TODO: add color fill
  c.stroke()

def readAndDrawPolygon(c, color, zoom, x, y):
  global pdata_offset

  value = fetch_polygon_data()
    
  if value >= 0xC0:
    if color & 0x80:
      color = value & 0x3F

    backup = pdata_offset
    fillPolygon(c, zoom, color, x, y)
    pdata_offset = backup
  else:
    value &= 0x3F
    if value == 2:
      readAndDrawPolygonHierarchy(c, zoom, x, y)
    else:
      print("ERROR: readAndDrawPolygon() (value != 2)\n")
      sys.exit(-1)

def readAndDrawPolygonHierarchy(c, zoom, pgc_x, pgc_y):
  global pdata_offset
  pt_x = pgc_x - (fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
  pt_y = pgc_y - (fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
  num_children = fetch_polygon_data()

  print ("  hierarchy with {} children.".format(num_children))
  for child in range(num_children):

    offset = fetch_polygon_data()
    offset = offset << 8 | fetch_polygon_data()

    po_x = pt_x + (fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
    po_y = pt_y + (fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
    print "child #{}: offset={} ({}) po_x={} po_y={}".format(
      child, 
      hex((2*offset) & 0xFFFF), hex(offset),
      po_x, po_y)

    color = 0xFF
    if offset & 0x8000:
      color = fetch_polygon_data() & 0x7F
      fetch_polygon_data() #and waste a byte...

    backup = pdata_offset

    pdata_offset = (offset & 0x7FFF) * 2
    readAndDrawPolygon(c, color, zoom, po_x, po_y);

    pdata_offset = backup

COLOR_BLACK = 0xFF
def extract_polygon_data():
  global polygon_data, pdata_offset
  from cairo import SVGSurface, Context, Matrix
  polygon_data = open("anotherw/cinematic.rom").read()
  for addr in video_entries.keys():
    entry = video_entries[addr]
    if addr > 0xFFFF:
      print "skipping {}".format(hex(addr))
      continue
    import os
    dirpath = "level_%s/video/" % (game_level)
    if not os.path.exists(dirpath):
      os.mkdir("level_%s" % game_level)
      os.mkdir(dirpath)
    s = SVGSurface("level_%s/video/%s.svg" % (game_level, hex(addr)), 320, 200)
    c = Context(s)
    zoom = entry["zoom"]
    x = entry["x"]
    y = entry["y"]

    if not isinstance(zoom, int):
      zoom = 0x40 #HACK!

    if not isinstance(x, int):
      x = 160 #HACK!

    if not isinstance(y, int):
      y = 100 #HACK!

    print ("\ndecoding polygons at {}: {}".format(hex(addr), entry))
    pdata_offset = addr
    readAndDrawPolygon(c, COLOR_BLACK, zoom, x, y)
    s.finish()

import sys
if len(sys.argv) != 2:
  print("usage: {} input.rom".format(sys.argv[0]))
else:
  gamerom = sys.argv[1]
  for game_level in range(9):
    print "disassembling level {}...".format(game_level)
    trace = AWVM_Trace(gamerom, rombank=0x10000*game_level, loglevel=0)
    trace.run()
#    trace.print_ranges()
#    trace.print_grouped_ranges()
#    print_video_entries()
    extract_polygon_data()
    print "\t{} video entries.".format(len(video_entries.keys()))
    print "\t{} video2 entries.".format(len(video2_entries.keys()))
    video_entries = {}
    video2_entries = {}
    trace.save_disassembly_listing("level-{}.asm".format(game_level))

