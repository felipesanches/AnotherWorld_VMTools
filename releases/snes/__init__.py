from releases import common_data

def generate_romset(input_dir, output_dir):
    from releases.snes.snes2romset import SNESROMSet
    input_rom_file = f"{input_dir}/Out of This World (USA).sfc"

    # TODO: validate original file checksum
    #       md5sum = ?
    #       sha1sum = ?

    romset = SNESROMSet(input_rom_file, output_dir)
    romset.generate()
    
    # TODO: validate checksums of generated ROM set
    #       according to the checksums listed below


MD5_CHECKSUMS = {
    'bytecode.rom': '?',
    'cinematic.rom': '?',
    'palettes.rom': 'BAD_DUMP 81ad94d572990cba3a9e163c5fab4591', # copied from msdos release
    'str_data.rom': 'BAD_DUMP 6e4f0bcfc98b1e956686553d67011859', # copied from Fabien Sanglard's engine / msdos release
    'str_index.rom': 'BAD_DUMP 254a3e2c0a84fde07a600618b3e32744', # copied from Fabien Sanglard's engine / msdos release
    'video2.rom': '?',
}
 
LABELED_CINEMATIC_ENTRIES = {
  0: common_data.LABELED_CINEMATIC_ENTRIES.get(1, []),
}


POSSIBLY_UNUSED_CODEBLOCKS = {
}


KNOWN_LABELS = {
}

