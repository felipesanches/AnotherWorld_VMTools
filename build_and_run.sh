MAME=~/mame/awvm64
ROMS=~/ROM_DUMPS/FSanches/another_world_ROMs
TARGET=$ROMS/aw_msdos

./AWVM_assembler.py output/level_0/level-0.asm
./AWVM_assembler.py output/level_1/level-1.asm
./AWVM_assembler.py output/level_2/level-2.asm
./AWVM_assembler.py output/level_3/level-3.asm
./AWVM_assembler.py output/level_4/level-4.asm
./AWVM_assembler.py output/level_5/level-5.asm
./AWVM_assembler.py output/level_6/level-6.asm
./AWVM_assembler.py output/level_7/level-7.asm
./AWVM_assembler.py output/level_8/level-8.asm

cp output/level_0/level-0.bin $TARGET/resource-0x15.bin
cp output/level_1/level-1.bin $TARGET/resource-0x18.bin
cp output/level_2/level-2.bin $TARGET/resource-0x1b.bin
cp output/level_3/level-3.bin $TARGET/resource-0x1e.bin
cp output/level_4/level-4.bin $TARGET/resource-0x21.bin
cp output/level_5/level-5.bin $TARGET/resource-0x24.bin
cp output/level_6/level-6.bin $TARGET/resource-0x27.bin
cp output/level_7/level-7.bin $TARGET/resource-0x2a.bin
cp output/level_8/level-8.bin $TARGET/resource-0x7e.bin

$MAME -rp $ROMS aw_msdos -window -debug
