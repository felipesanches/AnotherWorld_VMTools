from releases import common_data

def generate_romset(input_dir, output_dir):
    from .genesis2romset import GenesisEuropeROMSet
    input_rom_file = f"{input_dir}/Another World (Europe).md"

    # TODO: validate original file checksum
    #       md5sum = 8cc928edf09159401618e273028216ea
    #       sha1sum = 9d98d6817b3e3651837bb2692f7a2a60a608c055

    romset = GenesisEuropeROMSet(input_rom_file, output_dir)
    romset.generate()
    
    # TODO: validate checksums of generated ROM set
    #       according to the checksums listed below


MD5_CHECKSUMS = {
    'bytecode.rom': '63e69335329c578a41d55d3843871fb3',
    'cinematic.rom': '8c6d3ee16b0d62927df6093b5c641e31',
    'palettes.rom': 'BAD_DUMP 81ad94d572990cba3a9e163c5fab4591', # copied from msdos release
    'str_data.rom': 'BAD_DUMP 6e4f0bcfc98b1e956686553d67011859', # copied from Fabien Sanglard's engine / msdos release
    'str_index.rom': 'BAD_DUMP 254a3e2c0a84fde07a600618b3e32744', # copied from Fabien Sanglard's engine / msdos release
    'video2.rom': 'ef35800c0c9effd2dd8388d140370d61',
}

# Note:
# The SEGA Genesis game does not include the game intro sequence and the codewheel screen
# which are typically the first two sets of bytecode/cinematic entries
STAGE_TITLES = [
  "Arrival at the Lake & Beast Chase",  # 0
  "Prison",                             # 1
  "TODO - Name this stage (bank number #2)",
  "TODO - Name this stage (bank number #3)",
  "TODO - Name this stage (bank number #4)",
  "TODO - Name this stage (bank number #5)",
  "Secret Code Entry Screen",  # 6
]

 
LABELED_CINEMATIC_ENTRIES = {
  0: common_data.LABELED_CINEMATIC_ENTRIES.get(2, []),
  1: common_data.LABELED_CINEMATIC_ENTRIES.get(3, []),
  2: common_data.LABELED_CINEMATIC_ENTRIES.get(4, []),
  3: common_data.LABELED_CINEMATIC_ENTRIES.get(5, []),
  4: common_data.LABELED_CINEMATIC_ENTRIES.get(6, []),
  5: common_data.LABELED_CINEMATIC_ENTRIES.get(7, []),
  6: common_data.LABELED_CINEMATIC_ENTRIES.get(8, []),
}


POSSIBLY_UNUSED_CODEBLOCKS = {
  0: [0x0064, 0x00B6, 0x0141, 0x01A5, 0x01F8, 0x01FF, 0x04C2, 0x05E6,
      0x0619, 0x08E4, 0x096C, 0x0CB0, 0x0D42, 0x1106, 0x16F1, 0x2EBF,
      0x2F88, 0x3607, 0x38FD, 0x3A38, 0x435D, 0x437D, 0x4513, 0x4533,
      0x4C55, 0x4FEA],

  1: [0x0095, 0x00FF, 0x0719, 0x0744, 0x0793, 0x079A, 0x08D1, 0x0B73,
      0x0BA1, 0x10AF, 0x1182, 0x132D, 0x19FF, 0x270A, 0x277A, 0x28CF,
      0x33F1, 0x3407, 0x3C63, 0x430A, 0x4313, 0x4809, 0x48CB, 0x4B64,
      0x5849, 0x5857, 0x5BE5, 0x63D2, 0x662F, 0x6669, 0x6698, 0x66A7,
      0x66D3, 0x67A0, 0x687B, 0x68BE, 0x68E0, 0x69D4, 0x69E3, 0x7556,
      0x79D1, 0x7ECF, 0x7FA3, 0x846B, 0x89F1, 0x8A3B, 0x8AD1, 0x8B5C,
      0x8B6A, 0x90A9, 0x90C9, 0x9240, 0x9260],

  2: [0x00CE, 0x0107, 0x01EA, 0x1004, 0x13F1, 0x14BD, 0x1D4C, 0x2217,
      0x257E, 0x25DE, 0x27AF, 0x2945, 0x2DC7, 0x30A0, 0x3A6B, 0x3F5F,
      0x406A, 0x40A4, 0x42D8, 0x42DF, 0x44A1, 0x4718, 0x47F0, 0x48A3,
      0x498E, 0x49F2, 0x49F3, 0x4BB2, 0x4C86, 0x4E3A, 0x4F81, 0x5185,
      0x5ACA, 0x5D5F, 0x5DE6, 0x5EDF, 0x5FEC, 0x5FF9, 0x699C, 0x70D5,
      0x70DE, 0x76E3, 0x77A5, 0x86DC, 0x86F6, 0x8704, 0x8A0E, 0x945F,
      0x9499, 0x94A8, 0x94C8, 0x94D7, 0x9503, 0x96AB, 0x96DF, 0x96EE,
      0x9710, 0x97F3, 0x9802, 0xA07C, 0xA0AC, 0xA1DE, 0xA585, 0xAD03,
      0xC261, 0xC2A2, 0xC32F, 0xD926, 0xD97B, 0xDDBA, 0xE69E, 0xE842,
      0xE9E0, 0xEA00, 0xEB74, 0xEB94, 0xEFCD, 0xF369, 0xF392, 0xF3AA],

  3: [0x0A26, 0x0D05, 0x0D2C, 0x0D68, 0x0D8F, 0x0DD7, 0x12ED, 0x177C,
      0x1817, 0x1C40, 0x1D10, 0x1F43],

  4: [0x0247, 0x0902, 0x0C91, 0x0D07, 0x156E, 0x1B78, 0x1CF0, 0x1F74,
      0x2114, 0x24C5, 0x2813, 0x28D6, 0x297A, 0x299F, 0x2A66, 0x2F73,
      0x2FB8, 0x2F94, 0x2FE7, 0x2FF9, 0x2FFF, 0x3129, 0x3183, 0x31AF,
      0x31B6, 0x3326, 0x334E, 0x3357, 0x3578, 0x3B73, 0x3C5F, 0x3C9B,
      0x4125, 0x41F9, 0x43BD, 0x450E, 0x46D6, 0x4D5E, 0x5087, 0x534B,
      0x53C6, 0x541D, 0x543F, 0x5452, 0x548F, 0x5562, 0x5574, 0x5678,
      0x5685, 0x5B44, 0x6059, 0x6796, 0x679F, 0x6DE6, 0x6EA8, 0x71E4,
      0x7D7F, 0x7E0D, 0x7E1B, 0x8131, 0x897C, 0x8BCF, 0x8C36, 0x8C70,
      0x8C9F, 0x8CAE, 0x8CDA, 0x8E82, 0x8EB6, 0x8EC5, 0x8EE7, 0x8FCA,
      0x8FD9, 0x977B, 0x97AB, 0x9CA1, 0x9CDB, 0x9E57, 0x9EB7, 0x9F2F,
      0xA06E, 0xA0BE, 0xA183, 0xA318, 0xA680, 0xA72A, 0xA78A, 0xAAF1,
      0xABCF, 0xB34F, 0xB35C, 0xB364, 0xB6D6, 0xB6F1, 0xB8D4, 0xBA78,
      0xBC22, 0xBC42, 0xBDB6, 0xBDD6, 0xC1F7, 0xC581],
      # level_4: 0x3765 does not look like valid code.

  5: [0x0026, 0x094D, 0x0AEB],

  6: [0x0150, 0x0153, 0x01E5, 0x01F2, 0x0292, 0x0382, 0x03F5, 0x0467,
      0x0BBA, 0x0BCA]
}


KNOWN_LABELS = {
  0: {
    0x00A9: "KILL_CHANNEL_ROUTINE",
    0x00AA: "LIKELY_A_COPY_PROTECTION_MECHANISM",
    0x00CA: "SHUTDOWN_VM",
    0x00D3: "SOME_VIDEO_BUFFER_MAGIC",
    0x0187: "WEIRD_VIDEO_BUFFER_MANIPULATION",
    0x0192: "ANOTHER_UNCLEAR_VIDEO_BUFFER_MANIPULATION",
    0x01AD: "UNCLEAR_VIDEO_BUFFER_MANIPULATION",
    0x01E3: "INIT_VIDEO_BUFFERS",
    0x0FB5: "THE_BEAST_IS_KILLED_BY_A_LASER_SHOT",
    0x1407: "LESTER_RAISES_A_HAND",
    0x1677: "REED_PLANT_ANIMATION",
    0x1728: "SNEAKY_TENTACLE_FROM_THE_POOL",
    0x1746: "SNEAKY_TENTACLE_GOING_UP",
    0x18E2: "SNEAKY_TENTACLE_GIVES_UP_FOR_NOW",
    0x18EA: "SNEAKY_TENTACLE_GOING_DOWN",
    0x19C8: "VINE_SCREEN",
    0x1A0D: "OUTSIDE_POOL_SCREEN",
    0x1A6C: "FIRST_SCREEN_TO_THE_RIGHT",
    0x1B47: "SECOND_SCREEN_TO_THE_RIGHT",
    0x1BB1: "THIRD_SCREEN_TO_THE_RIGHT",
    0x1E2B: "THE_BEAST_KILLS_LESTER",
    0x1F40: "A_VIDEO_ROUTINE_OF_SOME_SORT",
    0x2026: "LESTER_GRABS_A_VINE_AND_SWINGS",
    0x2245: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_0",
    0x226D: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_1",
    0x2292: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_2",
    0x22AC: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_3",
    0x22CA: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_4",
    0x230F: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_5",
    0x236F: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_6",
    0x23CB: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_7",
    0x241E: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_8",
    0x24B1: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_9",
    0x24BA: "THE_BEAST_APPEARS_FOR_THE_FIRST_TIME_IN_THE_BACKGROUND",
    0x2610: "THE_BEAST_WANDERS_ON_THE_FIRST_SCREEN_TO_THE_RIGHT",
    0x26F2: "THE_BEAST_WANDERS_ON_THE_SECOND_SCREEN_TO_THE_RIGHT",
    0x27A8: "CHECK_IF_THE_BEAST_HAS_ALREADY_REACHED_LESTER",
    0x27B5: "THE_BEAST_IS_CLOSE_ENOUGH",
    0x27D0: "THE_BEAST_IS_STILL_AT_A_DISTANCE",
    0x2BA1: "THE_BEAST_SURPRISES_LESTER",
    0x2D04: "GOO_DRIPPING_FROM_SLUGS_CLAW_ANIMATION",
    0x3482: "SLUG_ANIMATION",
    0x35C2: "SLUG_FLIP_WALKS",
    0x3610: "BEETLE_WALKING_RIGHT",
    0x3705: "BEETLE_WALKING_LEFT",
    0x3ABE: "DRAW_SCENARIO_OF_VINE_SCREEN",
    0x3AD8: "DRAW_SCENARIO_OF_SECOND_SCREEN_TO_THE_RIGHT",
    0x3B02: "DRAW_SCENARIO_OF_THIRD_SCREEN_TO_THE_RIGHT",
    0x3B30: "DRAW_SCENARIO_OF_FIRST_SCREEN_TO_THE_RIGHT",
    0x3B66: "DRAW_OUTSIDE_POOL_SCENARIO",
    0x3BB0: "DRAW_INSIDE_ALIEN_POOL_SCENARIO",
    0x3C08: "A_CALM_ALIEN_POOL_BEFORE_LESTERS_ARRIVAL",
    0x3C56: "THE_LAB_CONSOLE_SUDDENLY_APPEARS_INSIDE_THE_ALIEN_POOL",
    0x3D5F: "SETUP_TENTACLE_ANIMATIONS",
    0x3DA3: "MAIN_TENTACLE_INSIDE_POOL_ANIMATION",
    0x3DCA: "OTHER_TENTACLES_INSIDE_POOL_ANIMATION",
    0x3ED5: "UPDATE_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3F0E: "SET_INITIAL_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3F90: "POOL_SURFACE_WAVES_ANIMATION",
    0x3FB7: "POOL_WATER_WAVY_GLARE_ANIMATION",
    0x3FEC: "UPDATE_POSITION_OF_SECOND_WAVY_GLARE",
    0x4012: "UPDATE_POSITION_OF_FIRST_WAVY_GLARE",
    0x402C: "SWIMMING_UP_TORSO_ANIMATION",
    0x4048: "BUBBLES_A_ANIMATION",
    0x4080: "BUBBLES_B_ANIMATION",
    0x40A2: "SWIMMING_UP_LEGS_ANIMATION",
    0x40D6: "LAB_CONSOLE_SINKING_ANIMATION",
    0x411C: "SINKING_AT_CONSOLE",
    0x412F: "SWIMMING_UP",
    0x4158: "UPDATE_POSITION_OF_BUBBLES",
  }
}

