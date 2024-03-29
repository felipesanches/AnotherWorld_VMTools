from releases.common_data import LABELED_CINEMATIC_ENTRIES

resource_ids = {
    "bytecode": [0x15, 0x18, 0x1b, 0x1e, 0x21, 0x24, 0x27, 0x2a, 0x7e],
    "screen": [0x13, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x53, 0x90, 0x91],
    "music": [0x07, 0x89, 0x8a],
    "sample": [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x09,
        0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x2c,
        0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x35,
        0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d,
        0x3e, 0x3f, 0x40, 0x41, 0x42, 0x4a, 0x4b, 0x4c,
        0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x54, 0x55,
        0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d,
        0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65,
        0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d,
        0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75,
        0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x80,
        0x81, 0x82, 0x83, 0x84, 0x88, 0x8b, 0x8c, 0x8d,
        0x8e
    ],
    "palette": [
        0x14, 0x17, 0x1a, 0x1d, 0x20, 0x23, 0x26, 0x29, 0x7d
    ],
    "cinematic": [
        0x16, 0x19, 0x1c, 0x1f, 0x22, 0x25, 0x28, 0x2b, 0x7f
    ],
    "video2": [0x11]
}

def generate_romset(input_dir, output_dir):
    from releases.common_data.banks2resources import Resources
    from releases.common_data.resources2romset import ROMSet
    import os
    import sys

    memlist_filename = f"{input_dir}/memlist.bin"
    if not os.path.exists(memlist_filename):
        print (f"Memlist file was not found at: {memlist_filename}")
        sys.exit(-1)

    memlist = open(memlist_filename, "rb")

    # TODO: validate checksums of the original files
    resources = Resources(input_dir, output_dir+"/msdos", memlist)
    resources.generate(uppercase=False)
    resources_dir = f"{output_dir}/msdos/resources"

    romset = ROMSet(resources_dir, output_dir+"/msdos", resource_ids)
    romset.generate()
    
    # TODO: validate checksums of generated ROM set
    #       according to the checksums listed below


MD5_CHECKSUMS = {
    'bytecode.rom': '0b0fd58ff5c8eb89dd0b619c3cace61b',
    'cinematic.rom': '1c36dc9286aa7843be6060a71b1fda6c',
    'palettes.rom': '81ad94d572990cba3a9e163c5fab4591',
    'str_data.rom': 'BAD_DUMP 6e4f0bcfc98b1e956686553d67011859', # copied from Fabien Sanglard's engine
    'str_index.rom': 'BAD_DUMP 254a3e2c0a84fde07a600618b3e32744', # copied from Fabien Sanglard's engine
    'video2.rom': 'ef35800c0c9effd2dd8388d140370d61'
}

POSSIBLY_UNUSED_CODEBLOCKS = {
  0: [0x007B, 0x0D1E, 0x109E],

  1: [0x00D2, 0x1527, 0x154E, 0x1BD6],

  2: [0x0054, 0x011A, 0x017E, 0x01D1, 0x01D8, 0x047F, 0x05A3, 0x05D6,
      0x08A1, 0x0929, 0x0C6D, 0x0CFF, 0x169F, 0x2E46, 0x2F0F, 0x3564,
      0x385A, 0x3995, 0x42AA, 0x42CA, 0x4460, 0x4480, 0x4BA2, 0x4F37],

  3: [0x0043, 0x016F, 0x0190, 0x0691, 0x06BC, 0x070B, 0x0712, 0x0849,
      0x0AEB, 0x0B19, 0x1027, 0x10FA, 0x129B, 0x1963, 0x2668, 0x26D8,
      0x282D, 0x330F, 0x3325, 0x3B69, 0x4210, 0x4219, 0x470F, 0x47D1,
      0x4A6A, 0x5746, 0x5754, 0x5AE2, 0x62CF, 0x652C, 0x6566, 0x6595,
      0x65A4, 0x65D0, 0x669D, 0x6778, 0x67BB, 0x67DD, 0x68D1, 0x68E0,
      0x7453, 0x78C8, 0x7E74, 0x8323, 0x8893, 0x88DD, 0x8973, 0x89FE,
      0x8A0C, 0x8F43, 0x8F63, 0x90DA, 0x90FA],

  4: [0x015D, 0x106F, 0x1456, 0x1518, 0x1D87, 0x2250, 0x25B7, 0x2617,
      0x27E8, 0x297E, 0x2DF4, 0x30CD, 0x314D, 0x3A81, 0x3F75, 0x4080,
      0x40BA, 0x42EE, 0x42F5, 0x4799, 0x4933, 0x49E6, 0x4AD1, 0x4B35,
      0x4B36, 0x4CF5, 0x4DC9, 0x4F73, 0x50BA, 0x5282, 0x5BC7, 0x5E56,
      0x5ED1, 0x5FCA, 0x60D7, 0x60E4, 0x6A6F, 0x71A8, 0x71B1, 0x7786,
      0x7848, 0x8765, 0x877F, 0x878D, 0x8A97, 0x94E8, 0x9522, 0x9531,
      0x9551, 0x9560, 0x958C, 0x9734, 0x9768, 0x9777, 0x9799, 0x987C,
      0x988B, 0xA105, 0xA135, 0xA267, 0xAD4B, 0xC238, 0xC279, 0xC306,
      0xD898, 0xD8F5, 0xDD31, 0xE615, 0xE7B9, 0xE957, 0xE977, 0xEAEB,
      0xEB0B, 0xEF44, 0xF2E0, 0xF309, 0xF321],

  5: [0x0405, 0x0A27, 0x0DCC, 0x13D9, 0x1868, 0x1903, 0x1DE7, 0x201A],

  6: [0x01DA, 0x084C, 0x0BD2, 0x0C48, 0x1460, 0x1A6A, 0x1BDC, 0x1E60,
      0x2000, 0x2371, 0x269C, 0x275F, 0x2803, 0x2828, 0x28EF, 0x2DF6,
      0x2E17, 0x2E3B, 0x2E6A, 0x2E7C, 0x2E82, 0x2FAC, 0x3006, 0x3032,
      0x3039, 0x319A, 0x31BF, 0x31C8, 0x33E9, 0x3579, 0x3ACF, 0x3BAB,
      0x3C97, 0x3CD3, 0x4159, 0x422D, 0x43E1, 0x4532, 0x46FA, 0x4D82,
      0x50AB, 0x536F, 0x53EA, 0x5441, 0x5463, 0x5476, 0x54B3, 0x5572,
      0x5584, 0x5688, 0x5695, 0x5B54, 0x6050, 0x678D, 0x6796, 0x6D8E,
      0x6E50, 0x717B, 0x7D16, 0x7DA4, 0x7DB2, 0x80C8, 0x8913, 0x8B66,
      0x8BCD, 0x8C07, 0x8C36, 0x8C45, 0x8C71, 0x8E19, 0x8E4D, 0x8E5C,
      0x8E7E, 0x8F61, 0x8F70, 0x9712, 0x9742, 0x9C30, 0x9C6A, 0x9C73,
      0x9DD7, 0x9DEF, 0x9E13, 0x9E53, 0x9E79, 0x9EEB, 0xA02A, 0xA07A,
      0xA13F, 0xA2D4, 0xA600, 0xA6D6, 0xA736, 0xAA8F, 0xAB67, 0xB26E,
      0xB27B, 0xB283, 0xB5F2, 0xB60D, 0xB7F0, 0xB994, 0xBB3E, 0xBB5E,
      0xBCD2, 0xBCF2, 0xC113, 0xC49D],
      # level_6: 0x36FC does not look like valid code.

  7: [0x0939, 0x0AD7],

  8: [0x014F, 0x029D, 0x03D4, 0x03E1, 0x064E, 0x06C1, 0x0733, 0x1078,
      0x1088]
}

STAGE_TITLES = [
  "Code-wheel screen",                 # bank 0 Default entry-point.
  "Intro Sequence",                    # bank 1 Runs cleanly.
  "Arrival at the Lake & Beast Chase", # bank 2 Requires [0xBC]=0x0010; [0xDC]=0x0021; [0xF2]=0x0FA0
  "Prison Escape",                     # bank 3 Runs cleanly.
  "Gas tunnels, Caves and Pool",       # bank 4	Requires [0xBC]=0x0010; [0xC6]=0x0080; [0xDC], 0x0021; [0xF2]=0x0FA0 (and also [E4] != 14)
  "Tank in the Battle Arena",          # bank 5 Runs cleanly.
  "Capsule Lands at the Bath",         # bank 6 Requires [0xF2]=0x0FA0
  "Game Ending Sequence",              # bank 7 Requires [0xF2]=0x0FA0
  "Secret Code Entry Screen",          # bank 8	Requires [0xBC]=0x0010; [0xC6]=0x0080; [0xDC]=0x0021; [0xF2]=0x0FA0
]

KNOWN_LABELS = {
  1: {
    0x0219: "DNA_ANIMATION",
  },
  2: {
    0x00C2: "SOME_VIDEO_BUFFER_MAGIC", # TODO: review this
    0x0099: "KILL_CHANNEL_ROUTINE", # I am puzzled by this routine containing a single killChannel instruction.
    0x009A: "LIKELY_A_COPY_PROTECTION_MECHANISM",
    0x00B9: "SHUTDOWN_VM",
    0x0160: "WEIRD_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x016B: "ANOTHER_UNCLEAR_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x0186: "UNCLEAR_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x01BC: "INIT_VIDEO_BUFFERS",
    0x0F72: "THE_BEAST_IS_KILLED_BY_A_LASER_SHOT",
    0x13BB: "LESTER_RAISES_A_HAND",
    0x1625: "REED_PLANT_ANIMATION",
    0x16D6: "SNEAKY_TENTACLE_FROM_THE_POOL",
    0x16F4: "SNEAKY_TENTACLE_GOING_UP",
    0x1890: "SNEAKY_TENTACLE_GIVES_UP_FOR_NOW",
    0x1898: "SNEAKY_TENTACLE_GOING_DOWN",
    0x1976: "VINE_SCREEN",
    0x19BB: "OUTSIDE_POOL_SCREEN",
    0x1A1A: "FIRST_SCREEN_TO_THE_RIGHT",
    0x1AF5: "SECOND_SCREEN_TO_THE_RIGHT",
    0x1B5F: "THIRD_SCREEN_TO_THE_RIGHT",
    0x1DD5: "THE_BEAST_KILLS_LESTER",
    0x1ED3: "A_VIDEO_ROUTINE_OF_SOME_SORT", # TODO: review this
    0x1FB9: "LESTER_GRABS_A_VINE_AND_SWINGS",
    0x21D8: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_0",
    0x2200: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_1",
    0x2225: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_2",
    0x223F: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_3",
    0x225D: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_4",
    0x22A2: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_5",
    0x2302: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_6",
    0x235E: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_7",
    0x23B1: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_8",
    0x2444: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_9",
    0x244D: "THE_BEAST_APPEARS_FOR_THE_FIRST_TIME_IN_THE_BACKGROUND",
    0x25A3: "THE_BEAST_WANDERS_ON_THE_FIRST_SCREEN_TO_THE_RIGHT",
    0x2685: "THE_BEAST_WANDERS_ON_THE_SECOND_SCREEN_TO_THE_RIGHT",
    0x273B: "CHECK_IF_THE_BEAST_HAS_ALREADY_REACHED_LESTER",
    0x2748: "THE_BEAST_IS_CLOSE_ENOUGH",
    0x2763: "THE_BEAST_IS_STILL_AT_A_DISTANCE",
    0x2B34: "THE_BEAST_SURPRISES_LESTER",
    0x2C91: "GOO_DRIPPING_FROM_SLUGS_CLAW_ANIMATION",
    0x33DF: "SLUG_ANIMATION",
    0x351F: "SLUG_FLIP_WALKS",
    0x356D: "BEETLE_WALKING_RIGHT",
    0x3662: "BEETLE_WALKING_LEFT",
    0x3A1B: "DRAW_SCENARIO_OF_VINE_SCREEN",
    0x3AC3: "DRAW_OUTSIDE_POOL_SCENARIO",
    0x3A8D: "DRAW_SCENARIO_OF_FIRST_SCREEN_TO_THE_RIGHT",
    0x3A35: "DRAW_SCENARIO_OF_SECOND_SCREEN_TO_THE_RIGHT",
    0x3A5F: "DRAW_SCENARIO_OF_THIRD_SCREEN_TO_THE_RIGHT",
    0x3B0D: "DRAW_INSIDE_ALIEN_POOL_SCENARIO",
    0x3B65: "A_CALM_ALIEN_POOL_BEFORE_LESTERS_ARRIVAL", # <-- The game starts here!
    0x3BB3: "THE_LAB_CONSOLE_SUDDENLY_APPEARS_INSIDE_THE_ALIEN_POOL",
    0x3CB6: "SETUP_TENTACLE_ANIMATIONS",
    0x3CF0: "MAIN_TENTACLE_INSIDE_POOL_ANIMATION",
    0x3D17: "OTHER_TENTACLES_INSIDE_POOL_ANIMATION",
    0x3E22: "UPDATE_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3E5B: "SET_INITIAL_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3EDD: "POOL_SURFACE_WAVES_ANIMATION",
    0x3F04: "POOL_WATER_WAVY_GLARE_ANIMATION",
    0x3F39: "UPDATE_POSITION_OF_SECOND_WAVY_GLARE",
    0x3F5F: "UPDATE_POSITION_OF_FIRST_WAVY_GLARE",
    0x3F79: "SWIMMING_UP_TORSO_ANIMATION",
    0x3F95: "BUBBLES_A_ANIMATION",
    0x3FCD: "BUBBLES_B_ANIMATION",
    0x3FEF: "SWIMMING_UP_LEGS_ANIMATION",
    0x4023: "LAB_CONSOLE_SINKING_ANIMATION",
    0x4069: "SINKING_AT_CONSOLE",
    0x407C: "SWIMMING_UP",
    0x40A5: "UPDATE_POSITION_OF_BUBBLES",
  },
  4: {
    0x0081: "LIKELY_A_COPY_PROTECTION_MECHANISM",
  },
  7: {
    0x0025: "LIKELY_A_COPY_PROTECTION_MECHANISM",
  },
  8: {
    0x0179: "LIKELY_A_COPY_PROTECTION_MECHANISM",
  }
}

