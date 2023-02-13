DRIVER=aw_amipk
RELEASE=amiga

SRC=output/$RELEASE/disasm
MAME=./awvm
FULLSET=~/ROM_DUMPS/FSanches/another_world_ROMs
ROMS=~/built_roms
TARGET=$ROMS/${DRIVER}
TOOLCHAIN_DIR=`pwd`
mkdir -p $ROMS
cp $FULLSET/$DRIVER $ROMS -rf

$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_0/${RELEASE}_level-0.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_1/${RELEASE}_level-1.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_2/${RELEASE}_level-2.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_3/${RELEASE}_level-3.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_4/${RELEASE}_level-4.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_5/${RELEASE}_level-5.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_6/${RELEASE}_level-6.asm
$TOOLCHAIN_DIR/awvm-asm.py $SRC/level_7/${RELEASE}_level-7.asm
# $TOOLCHAIN_DIR/awvm-asm.py $SRC/level_8/${RELEASE}_level-8.asm

cp $SRC/level_0/${RELEASE}_level-0.bin $TARGET/resource-0x15.bin
cp $SRC/level_1/${RELEASE}_level-1.bin $TARGET/resource-0x18.bin
cp $SRC/level_2/${RELEASE}_level-2.bin $TARGET/resource-0x1b.bin
cp $SRC/level_3/${RELEASE}_level-3.bin $TARGET/resource-0x1e.bin
cp $SRC/level_4/${RELEASE}_level-4.bin $TARGET/resource-0x21.bin
cp $SRC/level_5/${RELEASE}_level-5.bin $TARGET/resource-0x24.bin
cp $SRC/level_6/${RELEASE}_level-6.bin $TARGET/resource-0x27.bin
cp $SRC/level_7/${RELEASE}_level-7.bin $TARGET/resource-0x2a.bin
# cp $SRC/level_8/${RELEASE}_level-8.bin $TARGET/resource-0x7e.bin

cd ~/mame
$MAME -rp $ROMS ${DRIVER} -window -ui_active
# -debug
