SRC=output/msdos/disasm
MAME=./awvm
FULLSET=~/ROM_DUMPS/FSanches/another_world_ROMs
ROMS=~/built_roms
TARGET=$ROMS/aw_msdos
TOOLCHAIN_DIR=`pwd`
mkdir -p $ROMS
cp $FULLSET/aw_msdos $ROMS -rf

$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_0/msdos_level-0.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_1/msdos_level-1.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_2/msdos_level-2.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_3/msdos_level-3.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_4/msdos_level-4.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_5/msdos_level-5.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_6/msdos_level-6.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_7/msdos_level-7.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_8/msdos_level-8.asm

cp $SRC/level_0/msdos_level-0.bin $TARGET/resource-0x15.bin
cp $SRC/level_1/msdos_level-1.bin $TARGET/resource-0x18.bin
cp $SRC/level_2/msdos_level-2.bin $TARGET/resource-0x1b.bin
cp $SRC/level_3/msdos_level-3.bin $TARGET/resource-0x1e.bin
cp $SRC/level_4/msdos_level-4.bin $TARGET/resource-0x21.bin
cp $SRC/level_5/msdos_level-5.bin $TARGET/resource-0x24.bin
cp $SRC/level_6/msdos_level-6.bin $TARGET/resource-0x27.bin
cp $SRC/level_7/msdos_level-7.bin $TARGET/resource-0x2a.bin
cp $SRC/level_8/msdos_level-8.bin $TARGET/resource-0x7e.bin

cd ~/mame
$MAME -rp $ROMS aw_msdos -window -ui_active
