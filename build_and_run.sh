MAME=~/mame/awvm64
ROMS=~/ROM_DUMPS/FSanches/another_world_ROMs
TARGET=$ROMS/aw_msdos
SRC=disasm

./AWVM_assembler.py $SRC/level_0/level-0.asm
./AWVM_assembler.py $SRC/level_1/level-1.asm
./AWVM_assembler.py $SRC/level_2/level-2.asm
./AWVM_assembler.py $SRC/level_3/level-3.asm
./AWVM_assembler.py $SRC/level_4/level-4.asm
./AWVM_assembler.py $SRC/level_5/level-5.asm
./AWVM_assembler.py $SRC/level_6/level-6.asm
./AWVM_assembler.py $SRC/level_7/level-7.asm
./AWVM_assembler.py $SRC/level_8/level-8.asm

cp $SRC/level_0/level-0.bin $TARGET/resource-0x15.bin
cp $SRC/level_1/level-1.bin $TARGET/resource-0x18.bin
cp $SRC/level_2/level-2.bin $TARGET/resource-0x1b.bin
cp $SRC/level_3/level-3.bin $TARGET/resource-0x1e.bin
cp $SRC/level_4/level-4.bin $TARGET/resource-0x21.bin
cp $SRC/level_5/level-5.bin $TARGET/resource-0x24.bin
cp $SRC/level_6/level-6.bin $TARGET/resource-0x27.bin
cp $SRC/level_7/level-7.bin $TARGET/resource-0x2a.bin
cp $SRC/level_8/level-8.bin $TARGET/resource-0x7e.bin

$MAME -rp $ROMS aw_msdos -window ;-debug
