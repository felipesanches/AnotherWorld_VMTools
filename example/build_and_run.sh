MAME=~/mame/awvm64
ROMS=~/ROM_DUMPS/FSanches/another_world_ROMs
TARGET=$ROMS/aw_msdos
SRC=disasm
EXAMPLE=bounce

../AWVM_assembler.py $EXAMPLE.asm && cp $EXAMPLE.bin $TARGET/resource-0x15.bin && $MAME -rp $ROMS aw_msdos -window #-debug
