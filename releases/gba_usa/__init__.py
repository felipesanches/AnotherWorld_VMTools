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
  1: [0x00B1],
}

KNOWN_LABELS = {
  1: {
    0x00AF: "KILL_CHANNEL_ROUTINE",
    0x00B0: "LIKELY_A_COPY_PROTECTION_MECHANISM",
    0x00D0: "SHUTDOWN_VM",
    0x00D9: "SOME_VIDEO_BUFFER_MAGIC",
    0x0193: "WEIRD_VIDEO_BUFFER_MANIPULATION",
    0x019E: "ANOTHER_UNCLEAR_VIDEO_BUFFER_MANIPULATION",
    0x01B9: "UNCLEAR_VIDEO_BUFFER_MANIPULATION",
    0x01EF: "INIT_VIDEO_BUFFERS",
    0x1689: "REED_PLANT_ANIMATION",
    0x173A: "SNEAKY_TENTACLE_FROM_THE_POOL",
    0x1758: "SNEAKY_TENTACLE_GOING_UP",
    0x18F4: "SNEAKY_TENTACLE_GIVES_UP_FOR_NOW",
    0x18FC: "SNEAKY_TENTACLE_GOING_DOWN",
    0x19DA: "VINE_SCREEN",
    0x1A1F: "OUTSIDE_POOL_SCREEN",
    0x1A7E: "FIRST_SCREEN_TO_THE_RIGHT",
    0x1B59: "SECOND_SCREEN_TO_THE_RIGHT",
    0x1BC3: "THIRD_SCREEN_TO_THE_RIGHT",
    0x1E3D: "THE_BEAST_KILLS_LESTER",
    0x1F52: "A_VIDEO_ROUTINE_OF_SOME_SORT",
    0x2038: "LESTER_GRABS_A_VINE_AND_SWINGS",
    0x2257: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_0",
    0x227F: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_1",
    0x22A4: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_2",
    0x22BE: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_3",
    0x22DC: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_4",
    0x2327: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_5",
    0x2387: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_6",
    0x23E3: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_7",
    0x2436: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_8",
    0x24C9: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_9",
    0x24D2: "THE_BEAST_APPEARS_FOR_THE_FIRST_TIME_IN_THE_BACKGROUND",
    0x2628: "THE_BEAST_WANDERS_ON_THE_FIRST_SCREEN_TO_THE_RIGHT",
    0x270A: "THE_BEAST_WANDERS_ON_THE_SECOND_SCREEN_TO_THE_RIGHT",
    0x27C0: "CHECK_IF_THE_BEAST_HAS_ALREADY_REACHED_LESTER",
    0x27CD: "THE_BEAST_IS_CLOSE_ENOUGH",
    0x27E8: "THE_BEAST_IS_STILL_AT_A_DISTANCE",
    0x2BBF: "THE_BEAST_SURPRISES_LESTER",
    0x2D28: "GOO_DRIPPING_FROM_SLUGS_CLAW_ANIMATION",
    0x34A6: "SLUG_ANIMATION",
    0x35E6: "SLUG_FLIP_WALKS",
    0x3634: "BEETLE_WALKING_RIGHT",
    0x3729: "BEETLE_WALKING_LEFT",
    0x3AE2: "DRAW_SCENARIO_OF_VINE_SCREEN",
    0x3AFC: "DRAW_SCENARIO_OF_SECOND_SCREEN_TO_THE_RIGHT",
    0x3B26: "DRAW_SCENARIO_OF_THIRD_SCREEN_TO_THE_RIGHT",
    0x3B54: "DRAW_SCENARIO_OF_FIRST_SCREEN_TO_THE_RIGHT",
    0x3B8A: "DRAW_OUTSIDE_POOL_SCENARIO",
    0x3BD4: "DRAW_INSIDE_ALIEN_POOL_SCENARIO",
    0x3C2C: "A_CALM_ALIEN_POOL_BEFORE_LESTERS_ARRIVAL",
    0x3C7A: "THE_LAB_CONSOLE_SUDDENLY_APPEARS_INSIDE_THE_ALIEN_POOL",
    0x3D89: "SETUP_TENTACLE_ANIMATIONS",
    0x3DCD: "MAIN_TENTACLE_INSIDE_POOL_ANIMATION",
    0x3DF4: "OTHER_TENTACLES_INSIDE_POOL_ANIMATION",
    0x3EFF: "UPDATE_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3F38: "SET_INITIAL_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3FBA: "POOL_SURFACE_WAVES_ANIMATION",
    0x3FE1: "POOL_WATER_WAVY_GLARE_ANIMATION",
    0x4016: "UPDATE_POSITION_OF_SECOND_WAVY_GLARE",
    0x403C: "UPDATE_POSITION_OF_FIRST_WAVY_GLARE",
    0x4056: "SWIMMING_UP_TORSO_ANIMATION",
    0x4072: "BUBBLES_A_ANIMATION",
    0x40AA: "BUBBLES_B_ANIMATION",
    0x40CC: "SWIMMING_UP_LEGS_ANIMATION",
    0x4100: "LAB_CONSOLE_SINKING_ANIMATION",
    0x4146: "SINKING_AT_CONSOLE",
    0x4159: "SWIMMING_UP",
    0x4182: "UPDATE_POSITION_OF_BUBBLES",
  }
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
