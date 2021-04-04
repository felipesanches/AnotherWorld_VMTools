RELEASE=msdos_release/WORLD
OUTPUT_DIR=output

echo "EXTRACTING RESOURCES..."
./banks2resources.py $RELEASE/memlist.bin resources

echo "GENERATING ROMSET..."
./resources2romset.py resources romset

echo "TRACING BYTECODE..."
./AWVM_trace.py romset/ disasm/

echo "Source-code available at this directory: disasm/"
