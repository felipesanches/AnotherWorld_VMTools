from releases.common_data import LABELED_CINEMATIC_ENTRIES

def generate_romset(input_dir, output_dir):
    from releases.symbian_demo.symbian2romset import SymbianDemoROMSet

    # TODO: validate checksums of the original files

    romset = SymbianDemoROMSet(input_dir, output_dir)
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

