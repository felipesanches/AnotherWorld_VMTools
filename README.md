# Another World VM Tools

- Toolchain for software development targeting the virtual machine originally designed for Eric Chahi's Another World game.
- The scripts in this repo are Licensed under the GPL version 2 or later
- All scripts require Python 3.

## Requirements

- Copy of Another World (Out Of This World) - MSDOS version
- Python 3

## Scripts Descriptions and Use

The simples way to use this to get bytecode disasm (assuming you have a copy of the game files for MSDOS saved on the `msdos_release` directory) is to perform the following commands:

 > `./banks2resources.py msdos_release/memlist.bin resources/`

 > `./resources2romset.py resources/ romset/`

 > `./AWVM_trace.py romset/ disasm/`

Below is a more detailed descriptioni of each individual script:

### banks2resources.py
- Reads compressed game data files such as "msdos_release/bank*"
- Outputs raw binary files called "resource-0x*.bin"
- These are the files used by the MAME driver at https://github.com/felipesanches/mame/commit/1b943d2264b8d536b1815d8d21ef6234bb586b13

### resources2romset.py
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

## Development

### Create and activate your virtual environment
- virtualenv venv -p python3
- source venv/bin/activate
- pip install -r requirements.txt
