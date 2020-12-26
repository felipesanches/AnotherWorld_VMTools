import sys
from svgpathtools import svg2paths
if len(sys.argv) != 3:
    print "usage: {} <input.svg> <output.bin>".format(sys.argv[0])
    sys.exit(-1)

def byte(v):
    output.write(chr(v))

def word(v):
    byte(v >> 8)
    byte(v & 0xFF)

input_svg = sys.argv[1]
output = open(sys.argv[2], "r+b")
base_address = 0
cur_color = 0x30

#TODO: detect correct base_address
#      for storing data at the output binary file!

def output_polygon(points, color):
    # make sure we have an even number of points
    if len(points) % 2:
        points.append(points[-1])

    bbox_w, bbox_h = compute_bbox(points)

    byte(0xC0) # we don't declare color info here at the moment...
    byte(bbox_w)
    byte(bbox_h)
    byte(len(points))
    for x, y in points:
        byte(x + bbox_w/2)
        byte(y + bbox_h/2)

    print "\t-> {} {}".format(len(points), points)
    return 4 + len(points)*2

K = 1.0/10
paths, attributes = svg2paths(input_svg)
polygons = []
for path in paths:
    points = []
    print "\n\n" + path.d()
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

def compute_polygon_origin(points):
    # TODO: do something better here?
    return (0, 0)

def compute_bbox(points):
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

byte(0x02) # PolyHierarchy
byte(0x00) # reference-x = 0
byte(0x00) # reference-y = 0
byte(len(polygons))

address = base_address + 4 + 6*len(polygons)
for i, polygon in enumerate(polygons):
    points, color = polygon
    po_x, po_y = compute_polygon_origin(points)

    output.seek(base_address + 4 + 6*i)
    word(address/2)
    byte(po_x)
    byte(po_y)
    word(color << 8) #yep, we waste a byte!
    output.seek(address)
    address += output_polygon(points, color)

output.close()
