from releases import common_data, genesis_europe

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
  0: common_data.LABELED_CINEMATIC_ENTRIES.get(0, []),
  1: common_data.LABELED_CINEMATIC_ENTRIES.get(2, []),
#  2: common_data.LABELED_CINEMATIC_ENTRIES.get(3, []),
#  3: common_data.LABELED_CINEMATIC_ENTRIES.get(4, []),
#  4: common_data.LABELED_CINEMATIC_ENTRIES.get(5, []),
#  5: common_data.LABELED_CINEMATIC_ENTRIES.get(6, []),
#  6: common_data.LABELED_CINEMATIC_ENTRIES.get(7, []),
}


# The SEGA Genesis game does not include the game intro sequence and the codewheel screen
POSSIBLY_UNUSED_CODEBLOCKS = {
  1: genesis_europe.POSSIBLY_UNUSED_CODEBLOCKS.get(0, []), # Arrival + beast run
}


KNOWN_LABELS = {
  1: genesis_europe.KNOWN_LABELS.get(0, {}), # Arrival + beast run
}

