#!/usr/bin/env python3
#
# (c) 2022 Felipe Correa da Silva Sanches <juca@members.fsf.org>
# Licensed under GPL version 2 or later

class PolygonEncoder():
    def __init__(self, input_svg, output):
        self.input_svg = input_svg
        self.output_filename = output_filename
        self.output = open(output_filename, "r+b")


    def byte(self, v):
        self.output.write(chr(v))


    def word(self, v):
        self.byte(v >> 8)
        self.byte(v & 0xFF)


    #TODO: detect correct base_address
    #      for storing data at the output binary file!

    def output_polygon(self, points, color):
        # make sure we have an even number of points
        if len(points) % 2:
            points.append(points[-1])

        bbox_w, bbox_h = self.compute_bbox(points)

        self.byte(0xC0) # we don't declare color info here at the moment...
        self.byte(bbox_w)
        self.byte(bbox_h)
        self.byte(len(points))
        for x, y in points:
            self.byte(x + bbox_w/2)
            self.byte(y + bbox_h/2)

        print("\t-> {} {}".format(len(points), points))
        return 4 + len(points)*2


    def compute_polygon_origin(self, points):
        # TODO: do something better here?
        return (0, 0)


    def compute_bbox(self, points):
        xmin = False
        xmax = False
        ymin = False
        ymax = False
        for x, y in points:
            if xmin:
                xmin = min(xmin, x)
            else:
                xmin = x

            if ymin:
                ymin = min(ymin, y)
            else:
                ymin = y

            if xmax:
                xmax = max(xmax, x)
            else:
                xmax = x

            if ymax:
                ymax = max(ymax, y)
            else:
                ymax = y
        w = xmax - xmin
        h = ymax - ymin
        return (w, h)


    def encode(self):
        import svgpathtools

        base_address = 0
        cur_color = 0x30
        K = 1.0/10
        paths, attributes = svgpathtools.svg2paths(self.input_svg)
        polygons = []
        for path in paths:
            points = []
            print ("\n\n" + path.d())
            for element in path:
                x1 = int(element.start.real * K)
                y1 = int(element.start.imag * K)
                x2 = int(element.end.real * K)
                y2 = int(element.end.imag * K)
                if len(points) == 0:
                    points.append((x1, y1))

                if points[-1] != (x1, y1):
                    cur_color += 1
                    polygons.append((points, cur_color))
                    points = [(x1, y1), (x2, y2)]
                else:
                    points.append((x2, y2))
            cur_color += 1
            polygons.append((points, cur_color))


        self.byte(0x02) # PolyHierarchy
        self.byte(0x00) # reference-x = 0
        self.byte(0x00) # reference-y = 0
        self.byte(len(polygons))

        address = base_address + 4 + 6*len(polygons)
        for i, polygon in enumerate(polygons):
            points, color = polygon
            po_x, po_y = self.compute_polygon_origin(points)

            self.output.seek(base_address + 4 + 6*i)
            self.word(address/2)
            self.byte(po_x)
            self.byte(po_y)
            self.word(color << 8) #yep, we waste a byte!
            self.output.seek(address)
            address += self.output_polygon(points, color)

        self.output.close()
        print(f"Polygon data successfully encoded to {output_filename}")



import sys
if len(sys.argv) != 3:
    print(f"usage: {sys.argv[0]} <input.svg> <output.bin>")
    sys.exit(-1)


input_svg = sys.argv[1]
output_filename = sys.argv[2]
pe = PolygonEncoder(input_svg, output_filename)
pe.encode()
