#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 2 or later

import os
from cairo import SVGSurface, Context, Matrix
COLOR_BLACK = 0xFF
DEFAULT_ZOOM = 0x40
MAX_POINTS = 50


def makedir(path):
    if not os.path.exists(path):
        os.mkdir(path)


class PolygonDecoder():
    def __init__(self, romset_dir, output_dir):
        self.romset_dir = romset_dir
        self.output_dir = output_dir
        self.pdata_offset = None
        self.polygon_data = None
        makedir(output_dir)

    def set_color_from_palette(self, ctx, palNum, color):
        p = self.game_level << 11 | 32*palNum + 2*(color % 16)
        c1 = self.palette_data[p]
        c2 = self.palette_data[p+1]
        r = ((c1 & 0x0F) << 2) | ((c1 & 0x0F) >> 2)
        g = ((c2 & 0xF0) >> 2) | ((c2 & 0xF0) >> 6)
        b = ((c2 & 0x0F) >> 2) | ((c2 & 0x0F) << 2)
        ctx.set_source_rgb(r/64.0, g/64.0, b/64.0)


    def fetch_polygon_data(self):
        try:
            value = self.polygon_data[self.game_level << 16 | self.pdata_offset]
        except:
            print(f'ERROR: self.pdata_offset={hex(self.pdata_offset)}')
            import sys
            sys.exit(-1)

        # visited_pdata(self.pdata_offset)
        self.pdata_offset += 1
        return value


    def fillPolygon(self, ctx, palette_number, color, zoom, cx, cy):
        #print("    <{}>".format(hex(self.pdata_offset)))
        bbox_w = self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM;
        bbox_h = self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM;
        numPoints = self.fetch_polygon_data()
        #print("        -> {} points polygon".format(numPoints))

        if not ((numPoints & 1) == 0 and numPoints < MAX_POINTS):
            print (f"ERROR: numPoints = {numPoints}")
            import sys
            sys.exit(-1)

        #Read all points, directly from bytecode segment
        for i in range(numPoints):
            x = self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM
            y = self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM
            #print ("        {}   x:{} y:{}".format(hex(self.pdata_offset), x, y))
            if i==0:
                ctx.move_to(cx - bbox_w/2 + x, cy - bbox_h/2 + y)
            else:
                ctx.line_to(cx - bbox_w/2 + x, cy - bbox_h/2 + y)

        ctx.close_path()

        ctx.stroke_preserve()
        ctx.save()
        self.set_color_from_palette(ctx, palette_number, color)
        ctx.fill()

        ctx.restore()
        self.set_color_from_palette(ctx, palette_number, color)
        ctx.set_line_width(2)
        ctx.stroke()


    def readAndDrawPolygon(self, address, ctx, palette_number, color, zoom, x, y):
        '''A shape can be given in two different ways:

	   - A list of screenspace vertices.
	   - A list of objectspace vertices, based on a delta from the first vertex.

	   This is a recursive function.
        '''
        self.pdata_offset = address
        value = self.fetch_polygon_data()
    
        if value >= 0xC0:
            if color & 0x80:
                color = value & 0x3F
    
            backup = self.pdata_offset
            self.fillPolygon(ctx, palette_number, color, zoom, x, y)
            self.pdata_offset = backup
        else:
            value &= 0x3F
            if value == 2:
                self.readAndDrawPolygonHierarchy(ctx, palette_number, zoom, x, y)
            else:
                print("ERROR: readAndDrawPolygon() (value != 2)\n")
                import sys
                sys.exit(-1)
    

    def readAndDrawPolygonHierarchy(self, ctx, palette_number, zoom, pgc_x, pgc_y):
        pt_x = pgc_x - (self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
        pt_y = pgc_y - (self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
        num_children = self.fetch_polygon_data() + 1

        # print (f"  hierarchy with {num_children} children.")
        for child in range(num_children):

            offset = self.fetch_polygon_data()
            offset = offset << 8 | self.fetch_polygon_data()

            po_x = pt_x + (self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
            po_y = pt_y + (self.fetch_polygon_data() * float(zoom) / DEFAULT_ZOOM)
            # print(f"child #{child}: offset={hex((2*offset) & 0xFFFF)} ({hex(offset)}) po_x={po_x} po_y={po_y}")

            color = 0xFF
            if offset & 0x8000:
                color = self.fetch_polygon_data() & 0x7F
                self.fetch_polygon_data() #and waste a byte...

            backup = self.pdata_offset
            self.readAndDrawPolygon((offset & 0x7FFF) * 2, ctx, palette_number, color, zoom, po_x, po_y);
            self.pdata_offset = backup


    def extract_polygon_data(self, game_level, entries, cinematic):
        if cinematic:
            self.game_level = game_level
        else:
            self.game_level = 0

        try:
            self.palette_data = open(f"{self.romset_dir}/palettes.rom", "rb").read()
        except:
            print("ERROR! Did not find a palettes.rom file...")
            return

        if cinematic:
            try:
                self.polygon_data = open(f"{self.romset_dir}/cinematic.rom", "rb").read()
            except:
                print("ERROR! Did not find a cinematic.rom file...")
                return
            level_path = f"{self.output_dir}/level_{game_level}"
            dirpath = f"{level_path}/cinematic/"
            makedir(level_path)
        else:
            try:
                self.polygon_data = open(f"{self.romset_dir}/video2.rom", "rb").read()
            except:
                print("ERROR! Did not find a video2.rom file...")
                return
    
            dirpath = f"{self.output_dir}/common_video/"

        makedir(dirpath)

        for addr in entries.keys():
            entry = entries[addr]
            s = SVGSurface(f"{dirpath}/{entry['label']}.svg", 320, 200)
            ctx = Context(s)
            zoom = entry["zoom"]
            x = entry["x"]
            y = entry["y"]
            palette_number = entry["palette_number"]

            if not isinstance(zoom, int):
                zoom = 0x40 #HACK!

            if not isinstance(x, int):
                x = 160 #HACK!

            if not isinstance(y, int):
                y = 100 #HACK!

            #print ("\ndecoding polygons at {}: {}".format(hex(addr), entry))
            self.readAndDrawPolygon(addr, ctx, palette_number, COLOR_BLACK, zoom, x, y)
            s.finish()


if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <romset_dir> <output_dir>")
        sys.exit(-1)

    romset_dir = sys.argv[1]
    output_dir = sys.argv[2]

    # example usage:
    game_level = 0
    entries = {
        0x031E: {'palette_number': 3, 'x': 160, 'y': 100, 'zoom': 0x100, 'label': "test"}
    }
    pd = PolygonDecoder(romset_dir,
                        output_dir)
    pd.extract_polygon_data(game_level, entries, cinematic=True)

    entries = {
        addr: {'palette_number': 1, 'x': 160, 'y': 100, 'zoom': 0x100, 'label': f"test_{addr}"}
        for addr in [0x20, 0xb4, 0x128, 0x1b0, 0x234, 0x2ac, 0x358, 0x37c]
    }
    pd.extract_polygon_data(game_level, entries, cinematic=False)

