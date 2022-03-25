EXAMPLE=bounce
MAME=./awvm
ROMS=~/ROM_DUMPS/FSanches/another_world_ROMs
TARGET=$ROMS/aw_msdos
TOOLCHAIN_DIR=`pwd`/..
EXAMPLE_DIR=$TOOLCHAIN_DIR/example/

cd ~/mame
$TOOLCHAIN_DIR/AWVM_assembler.py $EXAMPLE_DIR/$EXAMPLE.asm && cp $EXAMPLE_DIR/$EXAMPLE.bin $TARGET/resource-0x15.bin && $MAME -rp $ROMS aw_msdos -window -ui_active #-debug
