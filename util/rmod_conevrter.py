# This Python script converts .fbx and (soon) .dae files into .rmod binary files that can be used
# within the 3D Rust engine. These files store information regarding vertices, normals, bitangents,
# tangents, UVs, materials, and animations. To convert .fbx files, you will need to download the
# Autodesk FBX SDK with Python 2.7 bindings from their website as it is currently proprietary and
# cannot be packaged with external applications like the game engine. This also requires Pillow,
# a maintained fork of PIL for texture data. This can be obtained through 'pip install Pillow'.
#
# Usage:
# - python rmod_converter.py diffuse specular normal shininess input_file output_file
#   This invocation takes an input name and an output name. The script attempts to read and convert
#   the input file and outputs the converted model with the provided output name and textures.
# - python rmod_converter.py diffuse specular normal shininess input_file
#   This abbreviated invocation takes an input name. The script attempts to read and convert the
#   input file and outputs the converted model and textures with the same name and path as the
#   input file but with the .rmod extention instead.
# The '_' character can be used to specify an absensce of a texture map. For example:
# 'python rmod_converter.py diffuse.bmp _ _ 75.0 model.fbx model.rmod'
#
# Brian Ho
# brian@brkho.com


import os
import struct
import sys
import time

# How many decimal places to round floats to.
FLOAT_DECIMAL_PLACES = 5;

# Data class for holding information about a vertex.
class Vertex:
    def __init__(self, pos, normal, tangent, bitangent, tcoord):
        self.pos = pos
        self.normal = normal
        self.tangent = tangent
        self.bitangent = bitangent
        self.tcoord = tcoord

# Output wrapper that prepends a message with timestamp.
def output(msg):
    time_str = time.strftime('%X')
    print '[{}] {}'.format(time_str, msg)

# Helper function to print out an error and terminate the program.
def error(msg):
    output('ERROR: {}'.format(msg))
    sys.exit(1)

# Helper function that indicates an error in the supplied input parameters.
def input_error():
    error('Proper invocation is \'python rmod_converter.py diffuse specular normal shininess ' +
        'input_file [output_file]\'.')

# Processes the diffuse, specular, and normal arguments.
def process_texture_args(argv):
    args = []
    for i in xrange(1, 4):
        args.append(argv[i] if argv[i] != '_' else None)
    args.append(float(argv[4]))
    return args

# Handles the command line case with just an input filename.
def handle_input(argv):
    args = process_texture_args(argv)
    input_name = argv[5]
    base_name = os.path.splitext(input_name)[0]
    args.extend([input_name, base_name + '.rmod'])
    return tuple(args)

# Handles the command line case with both input and output filenames.
def handle_input_output(argv):
    args = process_texture_args(argv)
    args.extend([argv[5], argv[6]])
    return tuple(args)

# Converts a FBX data structure into a nice tuple.
def fbx_to_tuple(item, size):
    converted_result = []
    for i in xrange(0, size):
        converted_result.append(round(item[i], FLOAT_DECIMAL_PLACES))
    return tuple(converted_result)

# Gets vertex information (normal, bitangent, tangent) from a mesh given indices.
def get_vertex_info(info, size, vertex_index, vertex_count):
    mapping_mode = info.GetMappingMode()
    reference_mode = info.GetReferenceMode()
    if mapping_mode == FbxLayerElement.eByControlPoint:
        if reference_mode == FbxLayerElement.eDirect:
            result = info.GetDirectArray().GetAt(vertex_index)
        elif reference_mode == FbxLayerElement.eIndexToDirect:
            index = info.GetIndexArray().GetAt(vertex_index)
            result = info.GetDirectArray().GetAt(index)
        else:
            error('Unsupported reference mode.')
    elif mapping_mode == FbxLayerElement.eByPolygonVertex:
        if reference_mode == FbxLayerElement.eDirect:
            result = info.GetDirectArray().GetAt(vertex_count)
        elif reference_mode == FbxLayerElement.eIndexToDirect:
            index = info.GetIndexArray().GetAt(vertex_count)
            result = info.GetDirectArray().GetAt(index)
        else:
            error('Unsupported reference mode.')
    else:
        error('Unsupported mapping mode.')
    return fbx_to_tuple(result, size)

# Reads in the specified input file and returns the relevant mesh information.
def read_input_file(fname):
    sdk_manager, scene = InitializeSdkObjects()
    load_result = LoadScene(sdk_manager, scene, fname)
    if not load_result:
        error('Unable to load FBX file: {}.'.format(fname))
    root_node = scene.GetRootNode()
    if not root_node:
        error("FBX file is empty.")
    result = []
    for i in xrange(0, root_node.GetChildCount()):
        result.extend(get_child_info(root_node.GetChild(i)))
    return result

# Recursively gets the child node information.
def get_child_info(node):
    result = []
    # We only care about Mesh attributes right now.
    if node.GetNodeAttribute().GetAttributeType() == FbxNodeAttribute.eMesh:
        result.append(get_mesh_info(node.GetNodeAttribute()))
    for i in xrange(0, node.GetChildCount()):
        result.extend(get_child_info(root_node.GetChild(i)))
    return result

# Gets information about a mesh node. Returns (elist, vlist).
def get_mesh_info(mesh):
    name = 'UNNAMED' if mesh.GetName() == '' else mesh.GetName()
    output('Processing mesh: {}.'.format(name))
    vertex_map = {}
    vertex_list = []
    element_list = []
    control_points = mesh.GetControlPoints()
    normals = mesh.GetElementNormal(0)
    tangents = mesh.GetElementTangent(0)
    bitangents = mesh.GetElementBinormal(0)
    uvs = mesh.GetElementUV(0)
    vertex_count = 0
    for i in xrange(0, mesh.GetPolygonCount()):
        if mesh.GetPolygonSize(i) != 3:
            error('Mesh must be triangulated.')
        for j in xrange(0, 3):
            vertex_index = mesh.GetPolygonVertex(i, j)
            pos = fbx_to_tuple(control_points[vertex_index], 3)
            normal = get_vertex_info(normals, 3, vertex_index, vertex_count)
            tangent = get_vertex_info(tangents, 3, vertex_index, vertex_count)
            bitangent = get_vertex_info(bitangents, 3, vertex_index, vertex_count)
            tcoord = get_vertex_info(uvs, 2, vertex_index, vertex_count)
            key = (pos, normal, tangent, bitangent, tcoord)
            if key not in vertex_map:
                # print key
                new_vertex = Vertex(pos, normal, tangent, bitangent, tcoord)
                vertex_list.append(new_vertex)
                vertex_map[key] = len(vertex_map)
            element_list.append(vertex_map[key])
            vertex_count += 1
    output('Finished reading {} triangles with {} vertices ({} unique).'.format(
        mesh.GetPolygonCount(), len(element_list), len(vertex_list)))
    return (element_list, vertex_list)

# Serializes a single byte into the byte array.
def serialize_byte(num, bytes):
    if num > 255 or num < 0:
        error('Cannot store {} in a single byte.'.format(num))
    bytes.append(num)

# Serializes a 32 bit floating point number into bytes in a platform independent manner as
# specified by the IEEE 754. The exponent and the mantissa are stored with most significant byte
# first.
def serialize_float32(num, bytes):
    # Technically we can store a lot more in a float, but let's give it a reasonable bound...
    if abs(num) > pow(2, 31):
        error('Cannot store {} in 4 bytes.'.format(num))
    packed = struct.pack('>f', num)
    bytes.extend([ord(elem) for elem in packed])


# Serializes a 32 bit unsigned integer into bytes in a platform independent manner. The number is
# stored with most significant byte first.
def serialize_uint32(num, bytes):
    if num > pow(2, 32) or num < 0:
        error('Cannot store {} in 4 bytes.'.format(num))
    packed = struct.pack('>I', num)
    bytes.extend([ord(elem) for elem in packed])
    # print ''.join(format(x, '02x') for x in bytes)

# Loads and serializes a texture image given a path.
def serialize_image(path, bytes):
    if path is None:
        serialize_uint32(0, bytes)
        serialize_uint32(0, bytes)
        return
    try:
        image = Image.open(path)
    except IOError:
        error('Cannot open texture map: {}.'.format(path))
    output('Loaded texture map: {}.'.format(path))
    width, height = image.size
    serialize_uint32(width, bytes)
    serialize_uint32(height, bytes)
    for pixel in image.getdata():
        serialize_byte(pixel[0], bytes)
        serialize_byte(pixel[1], bytes)
        serialize_byte(pixel[2], bytes)
        serialize_byte(pixel[3], bytes)

# Main entry point for the converter.
def main():
    handle_arguments = { 6: handle_input, 7: handle_input_output }
    if len(sys.argv) not in handle_arguments:
        input_error()
    diffuse, spec, norm, shininess, input_name, output_name = handle_arguments[
        len(sys.argv)](sys.argv)
    output("Starting conversion of {} and outputting as {}.".format(input_name, output_name))
    # Magic header that represents the string "RUSTGAME".
    bytes = bytearray([82, 85, 83, 84, 71, 65, 77, 69])
    serialize_image(diffuse, bytes)
    serialize_image(spec, bytes)
    serialize_image(norm, bytes)
    serialize_float32(shininess, bytes)
    element_list, vertex_list = read_input_file(input_name)[0]
    serialize_uint32(len(vertex_list), bytes)
    for vertex in vertex_list:
        serialize_float32(vertex.pos[0], bytes)
        serialize_float32(vertex.pos[1], bytes)
        serialize_float32(vertex.pos[2], bytes)
        serialize_float32(vertex.normal[0], bytes)
        serialize_float32(vertex.normal[1], bytes)
        serialize_float32(vertex.normal[2], bytes)
        serialize_float32(vertex.tangent[0], bytes)
        serialize_float32(vertex.tangent[1], bytes)
        serialize_float32(vertex.tangent[2], bytes)
        serialize_float32(vertex.bitangent[0], bytes)
        serialize_float32(vertex.bitangent[1], bytes)
        serialize_float32(vertex.bitangent[2], bytes)
        serialize_float32(vertex.tcoord[0], bytes)
        serialize_float32(vertex.tcoord[1], bytes)
    serialize_uint32(len(element_list), bytes)
    for element in element_list:
        serialize_uint32(element, bytes)
    with open(output_name, 'wb') as of:
        of.write(bytes)
    output("Saved converted model as as {}.".format(output_name))

if __name__ == '__main__':
    try:
        from FbxCommon import *
        from fbx import FbxLayerElement
    except ImportError:
        error('Autodesk FBX SDK with Python bindings is required.')
    try:
        from PIL import Image
    except ImportError:
        error('Pillow (PIL fork) is required.')
    main()