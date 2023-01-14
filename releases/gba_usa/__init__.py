from releases import common_data

def generate_romset(input_dir, output_dir):
    from releases.gba_usa.gba_usa2romset import GBAUSAROMSet

    # TODO: validate checksums of the original files

    romset = GBAUSAROMSet(input_dir + "/Another World (Prototype) # GBA.GBA", output_dir)
    romset.generate()
    
    # TODO: validate checksums of generated ROM set
    #       according to the checksums listed below


MD5_CHECKSUMS = {
    # TODO
    #'bytecode.rom': '',
    #'cinematic.rom': '',
    #'palettes.rom': '',
    #'str_data.rom': '',
    #'str_index.rom': '',
    #'video2.rom': ''
}

POSSIBLY_UNUSED_CODEBLOCKS = {
    # TODO
}

KNOWN_LABELS = {
    # TODO
}

LABELED_CINEMATIC_ENTRIES = {
  0: common_data.LABELED_CINEMATIC_ENTRIES.get(1, []),
  1: common_data.LABELED_CINEMATIC_ENTRIES.get(2, []),
  2: common_data.LABELED_CINEMATIC_ENTRIES.get(3, []),
  3: common_data.LABELED_CINEMATIC_ENTRIES.get(4, []),
  4: common_data.LABELED_CINEMATIC_ENTRIES.get(5, []),
  5: common_data.LABELED_CINEMATIC_ENTRIES.get(6, []),
  6: common_data.LABELED_CINEMATIC_ENTRIES.get(7, []),
  7: common_data.LABELED_CINEMATIC_ENTRIES.get(8, []),
}
