from releases.common_data import LABELED_CINEMATIC_ENTRIES

resource_ids = {
    "bytecode": [0x15, 0x18, 0x1b, 0x1e, 0x21, 0x24, 0x27, 0x2a],
    "screen": [], # TODO
    "music": [], # TODO
    "sample": [], # TODO
    "palette": [0x14, 0x17, 0x1a, 0x1d, 0x20, 0x23, 0x26, 0x29],
    "cinematic": [0x16, 0x19, 0x1c, 0x1f, 0x22, 0x25, 0x28],
    "video2": [0x11]
}

def generate_romset(input_dir, output_dir):
    from releases.common_data.banks2resources import Resources
    from releases.common_data.resources2romset import ROMSet
    import os

    filename = f"{input_dir}/another"
    if not os.path.exists(filename):
        print (f"File was not found: {filename}")
        sys.exit(-1)

    another = open(filename, "rb")
    another.seek(0x5ec2)
    memlist = another.read(20*147)

    # TODO: validate checksums of the original files
    import io
    resources = Resources(input_dir, output_dir+"/amiga", io.BytesIO(memlist))
    resources.generate(uppercase=True)
    resources_dir = f"{output_dir}/amiga/resources"

    romset = ROMSet(resources_dir, output_dir+"/amiga", resource_ids)
    romset.generate()
    
    # TODO: validate checksums of generated ROM set
    #       according to the checksums listed below


MD5_CHECKSUMS = {
    'bytecode.rom': '?',
    'cinematic.rom': '?',
    'palettes.rom': '?',
    'str_data.rom': 'BAD_DUMP 6e4f0bcfc98b1e956686553d67011859', # copied from Fabien Sanglard's engine
    'str_index.rom': 'BAD_DUMP 254a3e2c0a84fde07a600618b3e32744', # copied from Fabien Sanglard's engine
    'video2.rom': '?'
}

POSSIBLY_UNUSED_CODEBLOCKS = {
}

KNOWN_LABELS = {
}

