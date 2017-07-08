#!/usr/bin/env python

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


class ExecTrace():
  def __init__(self, romfile, verbose=True, debug=True, log_ranges=False):
    self.verbose = verbose
    self.debug = debug
    self.log_ranges = log_ranges
    self.rom = open(romfile).read()
    self.visited_ranges = []
    self.pending_entry_points = []
    self.current_entry_point = None
    self.PC = None

  def already_visited(self, address):
    if self.PC is not None:
      if address >= self.current_entry_point and address < self.PC:
        if self.debug:
          print ("RECENTLY: (PC={} address={})".format(hex(self.PC), hex(address)))
        return True

    for codeblock in self.visited_ranges:
      if address >= codeblock.start and address <= codeblock.end:
        if self.debug:
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
      if self.verbose:
        print("Restarting from: {}".format(hex(address)))

  def add_range(self, start, end, exit=None):
    if end < start:
      self.add_range(end, start, exit)
      return

    if self.debug:
      print("=== New Range: start: {}  end: {} ===".format(hex(start), hex(end)))
    block = CodeBlock(start, end, exit)
    self.visited_ranges.append(block)

  def print_status(self):
    print "Pending: {}".format(map(hex, self.pending_entry_points))
    if self.log_ranges:
      self.print_ranges()

  def schedule_entry_point(self, address):
    if self.already_visited(address):
      return

    if address not in self.pending_entry_points:
      self.pending_entry_points.append(address)
      if self.verbose:
        print "SCHEDULING: {}".format(hex(address))
        self.print_status()

  def subroutine(self, address):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[self.PC, address])
    self.schedule_entry_point(self.PC)
    self.schedule_entry_point(address)
    if self.verbose:
      print "{}: CALL SUBROUTINE ({})".format(hex(self.PC-2), hex(address))
      self.print_status()
    self.restart_from_another_entry_point()

  def return_from_subroutine(self):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[])
    if self.verbose:
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
    if self.verbose:
      print ("CONDITIONAL JUMP to {}".format(hex(address)))
      if self.log_ranges:
        self.print_ranges()
    self.restart_from_another_entry_point()

  def unconditional_jump(self, address):
    self.add_range(start=self.current_entry_point,
                   end=self.PC-1,
                   exit=[address])
    self.schedule_entry_point(address)
    if self.verbose:
      print ("JUMP to {}".format(hex(address)))
      if self.log_ranges:
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
      if self.verbose:
        print("ALREADY BEEN AT {}!".format(hex(self.PC)))
      if self.debug:
        print("pending_entry_points: {}".format(self.pending_entry_points))
      self.add_range(start=self.current_entry_point,
                     end=self.PC-1,
                     exit=[self.PC])
      self.restart_from_another_entry_point()
    else:
      self.PC += 1

  def fetch(self):
    value = ord(self.rom[self.PC])
    if self.debug:
      print (("Fetch at {}: {}").format(hex(self.PC), hex(value)))
    self.increment_PC()
    return value

  def run(self, entry_point=0x0000):
    self.current_entry_point = entry_point
    self.PC = entry_point
    while self.PC is not None:
      PC = self.PC
      line = self.disasm_instruction()
      # print("%04X: %s" % (PC, line))

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

