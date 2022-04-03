from releases import common_data
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
      0x0619, 0x08E4, 0x096C, 0x0CB0, 0x0D42, 0x1106, 0x16F1, 0x2F88,
      0x3607, 0x38FD, 0x435D, 0x437D, 0x4513, 0x4533, 0x4C55, 0x4FEA],

  1: [0x0095, 0x00FF, 0x0719, 0x0744, 0x0793, 0x079A, 0x08D1, 0x0B73,
      0x0BA1, 0x10AF, 0x1182, 0x132D, 0x19FF, 0x270A, 0x277A, 0x28CF,
      0x33F1, 0x3407, 0x3C63, 0x430A, 0x4313, 0x4809, 0x48CB, 0x4B64,
      0x5849, 0x5857, 0x5BE5, 0x63D2, 0x662F, 0x6669, 0x6698, 0x66A7,
      0x66D3, 0x67A0, 0x687B, 0x68BE, 0x68E0, 0x69D4, 0x69E3, 0x7556,
      0x79D1, 0x7ECF, 0x7FA3, 0x846B, 0x89F1, 0x8A3B, 0x8AD1, 0x8B6A,
      0x90A9, 0x90C9, 0x9240, 0x9260],

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
      0x2F94, 0x2FB8, 0x2FE7, 0x2FF9, 0x2FFF, 0x3129, 0x3183, 0x31AF,
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

  5: [0x0026, 0x094D, 0x0AEB],

  6: [0x0150, 0x0153, 0x01E5, 0x01F2, 0x0292, 0x0382, 0x03F5, 0x0467,
      0x0BBA, 0x0BCA]
}

KNOWN_LABELS = {
  0: {
  }
}

