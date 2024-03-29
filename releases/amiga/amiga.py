resource_ids = {
    "bytecode": [0x15, 0x18, 0x1b, 0x1e, 0x21, 0x24, 0x27, 0x2a, 0x7e],
    "screen": [], # TODO
    "music": [], # TODO
    "sample": [], # TODO
    "palette": [0x14, 0x17, 0x1a, 0x1d, 0x20, 0x23, 0x26, 0x29, 0x7d],
    "cinematic": [0x16, 0x19, 0x1c, 0x1f, 0x22, 0x25, 0x28, 0x7f],
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

STAGE_TITLES = [
  "Code-wheel screen",                 # bank 0 Default entry-point.
  "Intro Sequence",                    # bank 1 Runs cleanly.
  "Arrival at the Lake & Beast Chase", # bank 2 Requires [0xBC]=0x0010; [0xC6]=0x0080; [0xDC]=0x0021; [0xF2]=0x1770
  "Prison Escape",                     # bank 3 Requires [0xBC]=0x0010; [0xF2]=0x1770
  "Gas tunnels, Caves and Pool",       # bank 4 Requires [0xBC]=0x0010; [0xC6]=0x0080; [0xDC]=0x0021; [0xF2]=0x1770
  "Tank in the Battle Arena",          # bank 5 Runs cleanly.
  "Capsule Lands at the Bath",         # bank 6 Requires [0xF2]=0x1770
  "Game Ending Sequence",              # bank 7 Requires [0xF2]=0x1770
  "Secret Code Entry Screen",          # bank 8 Requires [0xBC]=0x0010; [0xC6]=0x0080; [0xDC]=0x0021; [0xF2]=0x1770
]

# At bank 2 "Arrival", there's a beetle walking left. I was never able to make Lester interact with it in any manner, even though I've seen graphical assets of animations of it walking to the right, flipping upside down, opening its wings, and perhaps even flying. There's also some routines in the bytecode, but I was not yet able to fully understand that portion of the code. This seems to be an unfinished feature. On the MSDOS version it does not appear at all, but I was once able to glitch the code and saw an animation of it flying. I should inspect it more carefully.

# At bank 3 "Prison Escape", the jail swings more easily/quickly in the Amiga version, if compared to the MSDOS one.
# Another difference that I noticed: the MSDOS version adds a guard to the lowest floor of the elevator.

# At bank 4 "Gas tunnels", on the Amiga version the gas jets are not poisonous as they are in the MSDOS version, and are also positioned in different places.

MD5_CHECKSUMS = {
    'bytecode.rom': '?',
    'cinematic.rom': '?',
    'palettes.rom': '?',
    'str_data.rom': 'BAD_DUMP 6e4f0bcfc98b1e956686553d67011859', # copied from Fabien Sanglard's engine
    'str_index.rom': 'BAD_DUMP 254a3e2c0a84fde07a600618b3e32744', # copied from Fabien Sanglard's engine
    'video2.rom': '?'
}

POSSIBLY_UNUSED_CODEBLOCKS = {
  0: [], # TODO
  1: [], # TODO
  2: [0x004B, 0x014D, 0x018F, 0x0196, 0x0439, 0x055D, 0x0590, 0x085B,
      0x08E3, 0x0C27, 0x0CB9, 0x163F, 0x2CFC, 0x2DC5, 0x341A, 0x3710,
      0x384B, 0x4116, 0x413F, 0x429C, 0x42C4, 0x4641, 0x4898, 0x4AA9,
      0x4B55],
  3: [], # TODO
  4: [], # TODO
  5: [], # TODO
  6: [], # TODO
  7: [], # TODO
}

KNOWN_LABELS = {
  1: {
    0x01A5: "DNA_ANIMATION",
  },
  2: {
    0x00BC: "SOME_VIDEO_BUFFER_MAGIC", # TODO: review this
    0x0090: "KILL_CHANNEL_ROUTINE", # I am puzzled by this routine containing a single killChannel instruction.
    0x0091: "LIKELY_A_COPY_PROTECTION_MECHANISM",
    0x00B3: "SHUTDOWN_VM",
    # 0x0???: "WEIRD_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x0142: "ANOTHER_UNCLEAR_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x0155: "UNCLEAR_VIDEO_BUFFER_MANIPULATION", # TODO: review this
    0x017A: "INIT_VIDEO_BUFFERS",
    0x0F2C: "THE_BEAST_IS_KILLED_BY_A_LASER_SHOT",
    0x1366: "LESTER_RAISES_A_HAND",
    0x15C5: "REED_PLANT_ANIMATION",
    0x1676: "SNEAKY_TENTACLE_FROM_THE_POOL",
    0x1694: "SNEAKY_TENTACLE_GOING_UP",
    0x1830: "SNEAKY_TENTACLE_GIVES_UP_FOR_NOW",
    0x1838: "SNEAKY_TENTACLE_GOING_DOWN",
    0x1916: "VINE_SCREEN",
    0x1957: "OUTSIDE_POOL_SCREEN",
    0x19B2: "FIRST_SCREEN_TO_THE_RIGHT",
    0x1A89: "SECOND_SCREEN_TO_THE_RIGHT",
    0x1AEF: "THIRD_SCREEN_TO_THE_RIGHT",
    0x1CF2: "THE_BEAST_KILLS_LESTER",
    0x1DE3: "A_VIDEO_ROUTINE_OF_SOME_SORT", # TODO: review this
    0x1EC5: "LESTER_GRABS_A_VINE_AND_SWINGS",
    0x20E4: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_0",
    0x210C: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_1",
    0x2131: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_2",
    0x214B: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_3",
    0x2169: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_4",
    0x21AE: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_5",
    0x220E: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_6",
    0x226A: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_7",
    0x22BD: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_8",
    0x2350: "GETTING_OUT_OF_THE_POOL__ANIMATION_PART_9",
    0x2359: "THE_BEAST_APPEARS_FOR_THE_FIRST_TIME_IN_THE_BACKGROUND",
    0x24AF: "THE_BEAST_WANDERS_ON_THE_FIRST_SCREEN_TO_THE_RIGHT",
    0x2591: "THE_BEAST_WANDERS_ON_THE_SECOND_SCREEN_TO_THE_RIGHT",
    0x2647: "CHECK_IF_THE_BEAST_HAS_ALREADY_REACHED_LESTER",
    0x2654: "THE_BEAST_IS_CLOSE_ENOUGH",
    0x266B: "THE_BEAST_IS_STILL_AT_A_DISTANCE",
    0x2A05: "THE_BEAST_SURPRISES_LESTER",
    0x2B5E: "GOO_DRIPPING_FROM_SLUGS_CLAW_ANIMATION",
    0x3295: "SLUG_ANIMATION",
    0x33D5: "SLUG_FLIP_WALKS",
    0x3423: "BEETLE_WALKING_RIGHT",
    0x3518: "BEETLE_WALKING_LEFT",
    0x38D1: "DRAW_SCENARIO_OF_VINE_SCREEN",
    0x38EB: "DRAW_SCENARIO_OF_SECOND_SCREEN_TO_THE_RIGHT",
    0x3915: "DRAW_SCENARIO_OF_THIRD_SCREEN_TO_THE_RIGHT",
    0x3943: "DRAW_SCENARIO_OF_FIRST_SCREEN_TO_THE_RIGHT",
    0x3979: "DRAW_OUTSIDE_POOL_SCENARIO",
    0x39C3: "DRAW_INSIDE_ALIEN_POOL_SCENARIO",
    0x3A1B: "A_CALM_ALIEN_POOL_BEFORE_LESTERS_ARRIVAL", # <-- The game starts here!
    0x3A69: "THE_LAB_CONSOLE_SUDDENLY_APPEARS_INSIDE_THE_ALIEN_POOL",
    0x3B6C: "SETUP_TENTACLE_ANIMATIONS",
    0x3BA6: "MAIN_TENTACLE_INSIDE_POOL_ANIMATION",
    0x3BCD: "OTHER_TENTACLES_INSIDE_POOL_ANIMATION",
    0x3CD8: "UPDATE_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3D11: "SET_INITIAL_POSITIONS_OF_TENTACLES_INSIDE_THE_POOL",
    0x3D93: "POOL_SURFACE_WAVES_ANIMATION",
    0x3DBA: "POOL_WATER_WAVY_GLARE_ANIMATION",
    0x3DEF: "UPDATE_POSITION_OF_SECOND_WAVY_GLARE",
    0x3E15: "UPDATE_POSITION_OF_FIRST_WAVY_GLARE",
    0x3E2F: "SWIMMING_UP_TORSO_ANIMATION",
    0x3E4B: "BUBBLES_A_ANIMATION",
    0x3E83: "BUBBLES_B_ANIMATION",
    0x3EA5: "SWIMMING_UP_LEGS_ANIMATION",
    0x3ED9: "LAB_CONSOLE_SINKING_ANIMATION",
    0x3F1F: "SINKING_AT_CONSOLE",
    # 0x????: "SWIMMING_UP",
    0x3F54: "UPDATE_POSITION_OF_BUBBLES",
  },
  3: {
    0x0080: "LIKELY_A_COPY_PROTECTION_MECHANISM",
  },
  4: {
    0x0081: "LIKELY_A_COPY_PROTECTION_MECHANISM",
  }
}

LABELED_CINEMATIC_ENTRIES = {
  1: {
    0x0F72: "WALKING_FEET_ARRIVING_0",
    0x0F7E: "WALKING_FEET_ARRIVING_1",
    0x0FA2: "WALKING_FEET_ARRIVING_2",
    0x0FCA: "WALKING_FEET_ARRIVING_3",
    0x0FF2: "WALKING_FEET_ARRIVING_4",
    0x101A: "WALKING_FEET_ARRIVING_5",
    0x1048: "WALKING_FEET_ARRIVING_6",
    0x105E: "WALKING_FEET_ARRIVING_7",
    0x1082: "WALKING_FEET_ARRIVING_8",
    0x10B6: "WALKING_FEET_ARRIVING_9",
    0x10E2: "WALKING_FEET_ARRIVING_10",
    0x110A: "WALKING_FEET_ARRIVING_11",
    0x113E: "WALKING_FEET_ARRIVING_12",
    0x1158: "WALKING_FEET_ARRIVING_13",
    0x117E: "WALKING_FEET_ARRIVING_14",
    0x11AC: "WALKING_FEET_ARRIVING_15",
    0x11C6: "WALKING_FEET_ARRIVING_16",
    0x1200: "WALKING_FEET_ARRIVING_17",
    0x122A: "WALKING_FEET_ARRIVING_18",
    0x1278: "WALKING_FEET_ARRIVING_19",
    0x1292: "WALKING_FEET_ARRIVING_20",
    0x12AC: "WALKING_FEET_ARRIVING_21",
    0x12F2: "WALKING_FEET_ARRIVING_22",
    0x130C: "WALKING_FEET_ARRIVING_23",
    0x1326: "WALKING_FEET_ARRIVING_24",
    0x1340: "WALKING_FEET_ARRIVING_25",
    0x135A: "WALKING_FEET_ARRIVING_26",
    0x1374: "WALKING_FEET_ARRIVING_27",
    0x13EA: "WALKING_FEET_ARRIVING_28",
    0x1404: "WALKING_FEET_ARRIVING_29",
    0x141E: "WALKING_FEET_ARRIVING_30",
    0x143C: "WALKING_FEET_ARRIVING_31",
    0x145A: "WALKING_FEET_ARRIVING_32",
    0x1478: "WALKING_FEET_ARRIVING_33",
    0x14F6: "WALKING_FEET_ARRIVING_34",
    0x1514: "WALKING_FEET_ENTERING_0",
    0x1532: "WALKING_FEET_ENTERING_1",
    0x1584: "WALKING_FEET_ENTERING_2",
    0x15B6: "WALKING_FEET_ENTERING_3",
    0x15D0: "WALKING_FEET_ENTERING_4",
    0x15EA: "WALKING_FEET_ENTERING_5",
    0x1658: "WALKING_FEET_ENTERING_6",
    0x166A: "WALKING_FEET_ENTERING_7",
    0x16A2: "WALKING_FEET_ENTERING_8",
    0x16EA: "WALKING_FEET_ENTERING_9",
    0x1712: "WALKING_FEET_ENTERING_10",
    0xF760: "CARKEY",
    0xF866: "DNA_0",
    0xF878: "DNA_1",
    0xF88A: "DNA_2",
    0xF89E: "DNA_3",
    0xF8B0: "DNA_4",
    0xF8C2: "DNA_5",
    0xF8D6: "DNA_6",
    0xF8EA: "DNA_7",
    0xF8FE: "DNA_8",
    0xF90E: "DNA_9",
    0xF922: "DNA_10",
    0xF936: "DNA_11",
    0xF94A: "DNA_12",
    0xF95E: "DNA_13",
    0xF970: "DNA_14",
    0xF982: "DNA_15",
    0xF994: "DNA_16",
  },
  2: {
    0x105C: "GETTING_OUT_OF_THE_POOL_F_0",
    0x10F8: "GETTING_OUT_OF_THE_POOL_F_1",
    0x1D2E: "RIGHT_KICK_1",
    0x1D52: "RIGHT_CROUCHING_2",
    0x1DB6: "RIGHT_CROUCHING_1",
    0x1E1A: "RIGHT_CROUCHING_0",
    0x1E6A: "RIGHT_CROUCH_KICK_0",
    0x2008: "LEFT_KICK_1",
    0x202C: "LEFT_CROUCHING_2",
    0x2050: "LEFT_CROUCHING_1",
    0x2074: "LEFT_CROUCHING_0",
    0x2098: "LEFT_KICK_0",
    0x20BC: "INSIDE_ALIEN_POOL_SCENARIO_0",
    0x2116: "INSIDE_ALIEN_POOL_SCENARIO_1",
    0x216A: "INSIDE_ALIEN_POOL_SCENARIO_2",
    0x229C: "INSIDE_ALIEN_POOL_SCENARIO_3",
    0x23F2: "INSIDE_ALIEN_POOL_SCENARIO_4",
    0x2524: "INSIDE_ALIEN_POOL_SCENARIO_5",
    0x2610: "INSIDE_ALIEN_POOL_SCENARIO_6",
    0x2718: "INSIDE_ALIEN_POOL_SCENARIO_7",
    0x2766: "POOL_WATER_WAVY_GLARE_EFFECT",
    0x2796: "POOL_SURFACE_WAVES_0",
    0x284E: "POOL_SURFACE_WAVES_1",
    0x28B6: "POOL_SURFACE_WAVES_2",
    0x2922: "POOL_SURFACE_WAVES_3",
    0x2976: "POOL_SURFACE_WAVES_4",
    0x29CA: "POOL_SURFACE_WAVES_5",
    0x2A12: "POOL_SURFACE_WAVES_6",
    0x2A56: "INSIDE_ALIEN_POOL_SURFACE_0",
    0x2AA0: "INSIDE_ALIEN_POOL_SURFACE_1",
    0x2B60: "FLOATING_LAB_CONSOLE",
    0x2C14: "SWIMMING_UP_LEGS_7",
    0x2C88: "SWIMMING_UP_LEGS_0",
    0x2CD4: "SWIMMING_UP_LEGS_1",
    0x2D20: "SWIMMING_UP_LEGS_2",
    0x2D6C: "SWIMMING_UP_LEGS_3",
    0x2DB8: "SWIMMING_UP_LEGS_4",
    0x2E04: "SWIMMING_UP_LEGS_5",
    0x2E50: "SWIMMING_UP_LEGS_6",
    0x2E9C: "SWIMMING_UP_TORSO_0",
    0x300A: "BUBBLES_A_0",
    0x3042: "BUBBLES_B_0",
    0x3066: "BUBBLES_B_1",
    0x3082: "BUBBLES_B_2",
    0x309A: "BUBBLES_B_3",
    0x30B6: "BUBBLES_B_4",
    0x30C6: "BUBBLES_A_1",
    0x30F2: "BUBBLES_B_5",
    0x3116: "BUBBLES_A_2",
    0x3142: "CONSOLE_UNDERWATER_EXPLOSION_0",
    0x317A: "CONSOLE_UNDERWATER_EXPLOSION_1",
    0x31B6: "SITTING_AT_SINKING_CONSOLE",
    0x3212: "SWIM_OUT_OF_CONSOLE_0",
    0x327A: "SWIM_OUT_OF_CONSOLE_1",
    0x32E2: "SWIM_OUT_OF_CONSOLE_2",
    0x335C: "SWIMMING_UP_TORSO_1",
    0x33A0: "SWIMMING_UP_TORSO_2",
    0x33E4: "OUTSIDE_POOL_SCENARIO_1",
    0x3438: "OUTSIDE_POOL_SCENARIO_5",
    0x34CC: "OUTSIDE_POOL_SCENARIO_6",
    0x3650: "OUTSIDE_POOL_SCENARIO_7",
    0x37B0: "OUTSIDE_POOL_SCENARIO_8",
    0x3804: "OUTSIDE_POOL_SCENARIO_9",
    0x38F4: "OUTSIDE_POOL_SCENARIO_4",
    0x3A78: "OUTSIDE_POOL_SCENARIO_3",
    0x3B1C: "OUTSIDE_POOL_SCENARIO_10",
    0x3D44: "OUTSIDE_POOL_SCENARIO_11",
    0x3EC8: "OUTSIDE_POOL_SCENARIO_12",
    0x3F3C: "OUTSIDE_POOL_SCENARIO_13",
    0x3FD4: "OUTSIDE_POOL_SCENARIO_14",
    0x40A0: "OUTSIDE_POOL_SCENARIO_15",
    0x4130: "OUTSIDE_POOL_SCENARIO_16",
    0x42B8: "OUTSIDE_POOL_SCENARIO_0",
    0x437C: "OUTSIDE_POOL_SCENARIO_2",
    0x4400: "BACKGROUND_LANDSCAPE_TERRAIN",
    0x4498: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_0",
    0x445C: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_1",
    0x4464: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_5",
    0x4538: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_0",
    0x4600: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_0",
    0x46DE: "BACKGROUND_SUN_RAYS",
    0x481E: "RIGHT_CROUCH_KICK_1",
    0x4856: "RIGHT_KICK_0",
    0x48E2: "LEFT_CROUCH_KICK_1",
    0x4906: "LEFT_CROUCH_KICK_0",
    0x49B2: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_1",
    0x49F0: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_2",
    0x4A80: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_3",
    0x4B22: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_4",
    0x4C5E: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_5",
    0x4CDE: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_2",
    0x4D2E: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_3",
    0x4F2E: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_1",
    0x509E: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_6",
    0x5106: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_6",
    0x5176: "SCENARIO_VINE_SCREEN_1",
    0x5272: "SCENARIO_VINE_SCREEN_0",
    0x5312: "SCENARIO_VINE_SCREEN_2",
    0x5624: "SCENARIO_VINE_SCREEN_3",
    0x56DC: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_4",
    0x588C: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_2",
    0x597A: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_3",
    0x5A2A: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_4",
    0x5AD2: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_5",
    0x5D72: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_7",
    0x5DCE: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_8",
    0x60B2: "SLUG_FLIP_WALKING_0",
    0x60CE: "SLUG_FLIP_WALKING_1",
    0x60EE: "SLUG_FLIP_WALKING_2",
    0x610E: "SLUG_FLIP_WALKING_3",
    0x612E: "SLUG_FLIP_WALKING_4",
    0x614E: "SLUG_FLIP_WALKING_5",
    0x616A: "BEETLE_WALKING_LEFT_0",
    0x61D6: "BEETLE_WALKING_LEFT_1",
    0x61F2: "BEETLE_WALKING_LEFT_2",
    0x620E: "BEETLE_WALKING_LEFT_3",
    0x622E: "BEETLE_WALKING_LEFT_4",
    0x624A: "BEETLE_WALKING_LEFT_5",
    0x6266: "BEETLE_WALKING_LEFT_6",
    0x73F6: "BACKGROUND_BEAST_BODY",
    0x7416: "BACKGROUND_BEAST_HEAD_TURNING_0",
    0x7436: "BACKGROUND_BEAST_HEAD_TURNING_1",
    0x745A: "BACKGROUND_BEAST_HEAD_TURNING_2",
    0x747E: "BACKGROUND_BEAST_HEAD_TURNING_3",
    0x74DE: "BEAST_SURPRISE_BODY_0",
    0x7592: "BEAST_SURPRISE_BODY_1",
    0x766E: "BEAST_SURPRISE_BODY_2",
    0x774A: "BEAST_SURPRISE_BODY_3",
    0x77A2: "BEAST_SURPRISE_BODY_4",
    0x787A: "BEAST_SURPRISE_BODY_5",
    0x79AE: "BEAST_SURPRISE_BODY_6",
    0x7A86: "BEAST_SURPRISE_BODY_7",
    0x7B60: "BEAST_SURPRISE_BODY_8",
    0x7C34: "BEAST_SURPRISE_BODY_9",
    0x7D08: "BEAST_SURPRISE_EYES_AND_TEETH_0",
    0x7D2C: "BEAST_SURPRISE_EYES_AND_TEETH_1",
    0x7D38: "BEAST_SURPRISE_EYES_AND_TEETH_2",
    0x7D44: "BEAST_SURPRISE_EYES_AND_TEETH_3",
    0x7D50: "BEAST_SURPRISE_EYES_AND_TEETH_4",
    0x7D5C: "BEAST_SURPRISE_EYES_AND_TEETH_5",
    0x7D68: "BEAST_SURPRISE_EYES_AND_TEETH_6",
    0x7DD0: "BEAST_SURPRISE_EYES_AND_TEETH_7",
    0x7E08: "BEAST_SURPRISE_EYES_AND_TEETH_8",
    0x7DEC: "BEAST_SURPRISE_EYES_AND_TEETH_9",
    0x7DB4: "BEAST_SURPRISE_EYES_AND_TEETH_10",
    0x7E24: "TENTACLE_WAVING_0",
    0x7E58: "TENTACLE_WAVING_1",
    0x7E84: "TENTACLE_WAVING_2",
    0x7EB0: "TENTACLE_WAVING_3",
    0x7EE0: "GETTING_OUT_OF_THE_POOL_D_0",
    0x7F40: "GETTING_OUT_OF_THE_POOL_D_1",
    0x7F94: "GETTING_OUT_OF_THE_POOL_D_2",
    0x7FE8: "GETTING_OUT_OF_THE_POOL_D_3",
    0x8040: "GETTING_OUT_OF_THE_POOL_D_4",
    0x8098: "GETTING_OUT_OF_THE_POOL_D_5",
    0x80EC: "GETTING_OUT_OF_THE_POOL_D_6",
    0x8134: "GETTING_OUT_OF_THE_POOL_D_7",
    0x818C: "GETTING_OUT_OF_THE_POOL_D_8",
    0x81D8: "GETTING_OUT_OF_THE_POOL_D_9",
    0x8224: "GETTING_OUT_OF_THE_POOL_D_10",
    0x8270: "GETTING_OUT_OF_THE_POOL_D_11",
    0x82BC: "GETTING_OUT_OF_THE_POOL_D_12",
    0x82FA: "GETTING_OUT_OF_THE_POOL_D_13",
    0x834A: "GETTING_OUT_OF_THE_POOL_D_14",
    0x83BE: "GETTING_OUT_OF_THE_POOL_D_16",
    0x843E: "GETTING_OUT_OF_THE_POOL_D_18",
    0x84BE: "GETTING_OUT_OF_THE_POOL_D_20",
    0x8542: "GETTING_OUT_OF_THE_POOL_E_0",
    0x860E: "GETTING_OUT_OF_THE_POOL_E_1",
    0x868A: "GETTING_OUT_OF_THE_POOL_E_2",
    0x870A: "GETTING_OUT_OF_THE_POOL_E_3",
    0x8786: "GETTING_OUT_OF_THE_POOL_E_4",
    0x8802: "GETTING_OUT_OF_THE_POOL_E_5",
    0x887E: "GETTING_OUT_OF_THE_POOL_E_6",
    0x88FE: "GETTING_OUT_OF_THE_POOL_E_7",
    0x897A: "GETTING_OUT_OF_THE_POOL_E_8",
    0x89F6: "GETTING_OUT_OF_THE_POOL_E_9",
    0x8A72: "GETTING_OUT_OF_THE_POOL_E_10",
    0x9002: "SLUG_ATTACK__LESTERS_LEG",
    0x9016: "SLUG_ATTACKING_LEG_0",
    0x9036: "SLUG_ATTACKING_LEG_1",
    0x905A: "SLUG_ATTACKING_LEG_2",
    0x9082: "SLUG_ATTACKING_LEG_3",
    0x90AA: "SLUG_ATTACKING_LEG_4",
    0x90E6: "SLUG_ATTACKING_LEG_5",
    0x9136: "SLUG_ATTACKING_LEG_6",
    0xD284: "SLUG_ATTACKING_LEG_7",
    0xD5D2: "SLUG_ATTACKING_LEG_8",
    0x915E: "SLUG_ATTACK__CLOSED_CLAW",
    0x919A: "SLUG_ATTACK__OPEN_CLAW",
    0x9612: "JUMPING_TOWARDS_VINE_0",
    0x96B6: "JUMPING_TOWARDS_VINE_1",
    0x9752: "JUMPING_TOWARDS_VINE_2",
    0x97F2: "JUMPING_TOWARDS_VINE_3",
    0x9892: "JUMPING_TOWARDS_VINE_4",
    0x9932: "LESTER_GRABBING_VINE_0",
    0x99D6: "LESTER_GRABBING_VINE_1",
    0x9B1E: "LESTER_GRABBING_VINE_2",
    0x9A7A: "HANGING_ON_THE_VINE_0",
    0x9BC2: "HANGING_ON_THE_VINE_1",
    0x9C66: "HANGING_ON_THE_VINE_2",
    0x9D0A: "HANGING_ON_THE_VINE_3",
    0x9DAE: "STATIC_VINE_ABOUT_TO_SNAP_0",
    0x9DFE: "STATIC_VINE_ABOUT_TO_SNAP_1",
    0x9F8E: "SWINGING_AFTER_VINE_SNAPS_0",
    0x9FF6: "SWINGING_AFTER_VINE_SNAPS_1",
    0xA06A: "SWINGING_AFTER_VINE_SNAPS_2",
    0xA0DE: "SWINGING_AFTER_VINE_SNAPS_3",
    0xA152: "SWINGING_AFTER_VINE_SNAPS_4",
    0xA1A6: "SWINGING_AFTER_VINE_SNAPS_5",
    0xA1FA: "SWINGING_AFTER_VINE_SNAPS_6",
    0xA23A: "SWINGING_AFTER_VINE_SNAPS_7",
    0xA276: "SWINGING_AFTER_VINE_SNAPS_8",
    0xA29A: "SWINGING_AFTER_VINE_SNAPS_9",
    0xA2BE: "SWINGING_AFTER_VINE_SNAPS_10",
    0xA2E2: "SWINGING_AFTER_VINE_SNAPS_11",
    0xA306: "SWINGING_AFTER_VINE_SNAPS_12",
    0xA32A: "SWINGING_AFTER_VINE_SNAPS_13",
    0xA34E: "SWINGING_AFTER_VINE_SNAPS_14",
    0xA372: "SWINGING_AFTER_VINE_SNAPS_15",
    0xA39A: "SWINGING_AFTER_VINE_SNAPS_16",
    0xA3C2: "SWINGING_AFTER_VINE_SNAPS_17",
    0xA3EE: "SWINGING_AFTER_VINE_SNAPS_18",
    0xA422: "SWINGING_AFTER_VINE_SNAPS_19",
    0xA456: "SWINGING_AFTER_VINE_SNAPS_20",
    0xA48A: "SWINGING_AFTER_VINE_SNAPS_21",
    0xA496: "SWINGING_AFTER_VINE_SNAPS_22",
    0xA4CA: "SWINGING_AFTER_VINE_SNAPS_23",
    0xA5AE: "LANDING_AFTER_SWING_0",
    0xA5D2: "LANDING_AFTER_SWING_1",
    0xA5F6: "LANDING_AFTER_SWING_2",
    0xA61A: "LANDING_AFTER_SWING_3",
    0xA63E: "LANDING_AFTER_SWING_4",
    0xA662: "LANDING_AFTER_SWING_5",
    0xA686: "LANDING_AFTER_SWING_6",
    0xA6AA: "LANDING_AFTER_SWING_7",
    0xA6CE: "LANDING_AFTER_SWING_8",
    0xA6FA: "LANDING_AFTER_SWING_9",
    0xA726: "LANDING_AFTER_SWING_10",
    0xA756: "LANDING_AFTER_SWING_11",
    0xA786: "LANDING_AFTER_SWING_12",
    0xA7A2: "BEAST_KILLING_LESTER_0",
    0xA89A: "BEAST_KILLING_LESTER_1",
    0xA974: "BEAST_KILLING_LESTER_2",
    0xAA4C: "BEAST_KILLING_LESTER_3",
    0xAB28: "BEAST_KILLING_LESTER_4",
    0xAC40: "BEAST_KILLING_LESTER_5",
    0xAD54: "BEAST_KILLING_LESTER_6",
    0xAEEC: "SLUG_0",
    0xAF14: "SLUG_1",
    0xAF40: "SLUG_2",
    0xAF70: "SLUG_3",
    0xAFA0: "SLUG_4",
    0xAFD0: "SLUG_5",
    0xB000: "SLUG_6",
    0xB030: "SLUG_7",
    0xB060: "SLUG_8",
    0xB090: "SLUG_9",
    0xB0C0: "SLUG_10",
    0xB0F0: "SLUG_11",
    0xB120: "SLUG_12",
    0xB14C: "SLUG_13",
    0xB3CC: "BEETLE_WALKING_RIGHT_0",
    0xB438: "BEETLE_WALKING_RIGHT_1",
    0xB454: "BEETLE_WALKING_RIGHT_2",
    0xB470: "BEETLE_WALKING_RIGHT_3",
    0xB490: "BEETLE_WALKING_RIGHT_4",
    0xB4AC: "BEETLE_WALKING_RIGHT_5",
    0xB4C8: "BEETLE_WALKING_RIGHT_6",
    0xBCDC: "BEAST_SURPRISE_SCENARIO_BACKGROUND",
    0xBE40: "GETTING_OUT_OF_THE_POOL_D_15",
    0xBE6C: "GETTING_OUT_OF_THE_POOL_D_17",
    0xBE96: "GETTING_OUT_OF_THE_POOL_D_19",
    0xBEC6: "GETTING_OUT_OF_THE_POOL_D_21",
    0xBF12: "GETTING_OUT_OF_THE_POOL_A_0",
    0xBF2A: "GETTING_OUT_OF_THE_POOL_A_1",
    0xBF36: "GETTING_OUT_OF_THE_POOL_A_2",
    0xBF46: "GETTING_OUT_OF_THE_POOL_A_3",
    0xBF56: "GETTING_OUT_OF_THE_POOL_A_4",
    0xBF62: "GETTING_OUT_OF_THE_POOL_A_5",
    0xBF96: "GETTING_OUT_OF_THE_POOL_A_6",
    0xBFB2: "GETTING_OUT_OF_THE_POOL_A_7",
    0xBFD2: "GETTING_OUT_OF_THE_POOL_A_8",
    0xBFEE: "GETTING_OUT_OF_THE_POOL_A_9",
    0xC016: "BEAST_KILLING_LESTER_7",
    0xC01E: "BEAST_KILLING_LESTER_8",
    0xC032: "BEAST_KILLING_LESTER_9",
    0xC05C: "BEAST_KILLING_LESTER_10",
    0xC0F2: "LESTER_ATTACK_SCENE_BACKGROUND",
    0xC5BE: "SNEAKY_TENTACLE_0",
    0xC5DA: "SNEAKY_TENTACLE_1",
    0xC5FA: "SNEAKY_TENTACLE_2",
    0xC616: "SNEAKY_TENTACLE_3",
    0xC632: "SNEAKY_TENTACLE_4",
    0xC652: "SNEAKY_TENTACLE_5",
    0xCEA8: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_9",
    0xCED0: "SCENARIO_2ND_SCREEN_TO_THE_RIGHT_7",
    0xCF20: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_6",
    0xCF6C: "SCENARIO_VINE_SCREEN_4",
    0xCFA0: "OUTSIDE_POOL_SCENARIO_17",
    0xD292: "GOO_DRIPPING_FROM_SLUGS_CLAW_0",
    0xD2D6: "GOO_DRIPPING_FROM_SLUGS_CLAW_1",
    0xD32E: "GOO_DRIPPING_FROM_SLUGS_CLAW_2",
    0xD38E: "GOO_DRIPPING_FROM_SLUGS_CLAW_3",
    0xD3F2: "GOO_DRIPPING_FROM_SLUGS_CLAW_4",
    0xD496: "GOO_DRIPPING_FROM_SLUGS_CLAW_5",
    0xD4CA: "GOO_DRIPPING_FROM_SLUGS_CLAW_6",
    0xD4FE: "GOO_DRIPPING_FROM_SLUGS_CLAW_7",
    0xD536: "GOO_DRIPPING_FROM_SLUGS_CLAW_8",
    0xEB98: "SCENARIO_VINE_SCREEN_FOREGROUND",
    0xEBC4: "SCENARIO_3RD_SCREEN_TO_THE_RIGHT_7",
    0xEC30: "REED_PLANT_0",
    0xEC48: "REED_PLANT_1",
    0xEC60: "REED_PLANT_2",
    0xEC78: "REED_PLANT_3",
    0xEC90: "REED_PLANT_4",
    0xECA8: "GETTING_OUT_OF_THE_POOL_B_0",
    0xED0C: "GETTING_OUT_OF_THE_POOL_B_1",
    0xED60: "GETTING_OUT_OF_THE_POOL_B_2",
    0xED98: "GETTING_OUT_OF_THE_POOL_B_3",
    0xEDCC: "GETTING_OUT_OF_THE_POOL_C_0",
    0xEDF0: "GETTING_OUT_OF_THE_POOL_C_1",
    0xEE0C: "GETTING_OUT_OF_THE_POOL_C_2",
    0xEECA: "GETTING_OUT_OF_THE_POOL_B_4",
    0xEEF2: "GETTING_OUT_OF_THE_POOL_B_5",
    0xEEAC: "GETTING_OUT_OF_THE_POOL_B_6",
    0xEE28: "GETTING_OUT_OF_THE_POOL_C_3",
    0xEE50: "GETTING_OUT_OF_THE_POOL_C_4",
    0xEE6C: "GETTING_OUT_OF_THE_POOL_C_5",
    0xF04A: "VIDEO_MASK_FOR_SNEAKY_TENTACLE_ANIMATION",
    0xF1C2: "SCENARIO_1ST_SCREEN_TO_THE_RIGHT_10",
  },
}
