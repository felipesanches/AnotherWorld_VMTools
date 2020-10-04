# AnotherWorld_VMTools
- Toolchain for software development targeting the virtual machine originally designed for Eric Chahi's Another World game.
- The scripts in this repo are Licensed under the GPL version 2 or later
- All scripts require Python 3.

### banks2resources
- Reads compressed game data files such as "msdos_release/bank*"
- Outputs raw binary files called "resource-0x*.bin"
- These are the files used by the MAME driver at https://github.com/felipesanches/mame/commit/1b943d2264b8d536b1815d8d21ef6234bb586b13

### resources2romset
- Packs together resource files into ROM files.
- This is also useful for https://github.com/felipesanches/AnotherWorld_FPGA

### AWVM_trace.py
- Disassembles ROM files and generates an assembly source code tree.
- Includes data files extracted from the resource files.

### AWVM_assembler.py
- Assembles a source tree into bytecode binaries (with embedded data).
- It must generate binary output absolutely identical to the inputs of AWVM_trace.py

### build_and_run.sh
- Helper shell script that builds an assembly source tree, copies the generated ROM files to the MAME rompath and executes the emulator.
- This script needs to be tweaked to use the directory paths of your project files, MAME executable, etc...
