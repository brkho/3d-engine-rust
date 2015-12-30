# Python package to convert from the COLLADA image format into a more engine
# friendly custom format used by the engine. This preserves information about
# vertices, indices, normals, UV coords, materials, and animations and
# packages the information into a compact binary representation. This script
# relies on pycollada obtained through PIP.
#
# Usage: python convert_collada.py input.dae output.r3d
#
# TODO: Convert to pure Rust. This was initially done as a Python script for
# rapid development purposes, but having a unified Rust codebase sounds nice.
#
# Brian Ho
# brian@brkho.com

import collada
import sys


def main():
  print 'Hello, World!'


if __name__ == '__main__':
  main()
