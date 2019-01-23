import os
import sys
from cffi import FFI

def find_dynamic_lib_file():
    from glob import glob
    import platform

    system = platform.system()

    # For Linux and Darwin platforms, the generated lib file extension is .so
    dyn_lib_name = "cdshealpix*.so"

    if system == 'Windows':
        # On windows, it is a pyd extension file (similar to dll)
        dyn_lib_name = "cdshealpix*.pyd"

    path = os.path.join(os.path.dirname(__file__), dyn_lib_name)
    filename = ""

    try:
        filename = glob(path)[0]
    except IndexError as e:
        print("Cannot find the dynamic lib located in: ", os.path.dirname(__file__))
        # Raising the exception to get the traceback
        raise

    return filename

ffi = FFI()
# Open and read the C wrapper code
with open("./cdshealpix/bindings.h", "r") as f_in:
    ffi.cdef(f_in.read())

# Open the dynamic library generated by setuptools_rust
dyn_lib_path = find_dynamic_lib_file()
lib = ffi.dlopen(dyn_lib_path)

from .healpix import lonlat_to_healpix, \
 healpix_to_lonlat, \
 healpix_to_skycoord, \
 healpix_vertices_lonlat, \
 healpix_vertices_skycoord, \
 healpix_neighbours, \
 cone_search_lonlat
from .version import __version__

__all__ = [
    'lonlat_to_healpix',
    'healpix_to_lonlat',
    'healpix_to_skycoord',
    'healpix_vertices_lonlat',
    'healpix_vertices_skycoord',
    'healpix_neighbours',
    'cone_search_lonlat',
]