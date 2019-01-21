
extern crate cdshealpix;

// use cdshealpix::proj;
use cdshealpix::nested::{get_or_create, neighbours, vertices,
                         cone_overlap_approx, cone_overlap_approx_custom, 
                         polygon_overlap_approx};
use cdshealpix::nested::bmoc::*;
use cdshealpix::compass_point::{MainWind};

// Build
// see: https://github.com/getsentry/milksnake

// see: https://users.rust-lang.org/t/calling-into-rust-from-python/8923
// see: https://www.bignerdranch.com/blog/building-an-ios-app-in-rust-part-3/


pub unsafe fn build_array<T>(ptr: *mut T, len: usize) -> &'static [T] {
  assert!(!ptr.is_null());
  std::slice::from_raw_parts(ptr, len)
}

pub unsafe fn build_array_mut<T>(ptr: *mut T, len: usize) -> &'static mut [T] {
  assert!(!ptr.is_null());
  std::slice::from_raw_parts_mut(ptr, len)
}

pub unsafe fn build_vec<T>(input: *mut T, len: usize) -> Vec<T> {
  assert!(!input.is_null());
  Vec::from_raw_parts(input, len, len)
}

/*
#[no_mangle]
pub extern "C" fn hpx_hash(depth: u8, lon: f64, lat: f64) -> u64 {
  hash(depth, lon, lat)
}

/// coords size must be 2 x n_elems
/// res     size mut be     n_elems
#[no_mangle]
pub extern "C" fn hpx_hash_multi(depth: u8, n_elems: u32, coords_ptr: *mut f64, res_ptr: *mut u64) {
  let n_elems = n_elems as usize;
  let coords = unsafe{ build_array(coords_ptr, 2 * n_elems) };
  let res = unsafe{ build_array_mut(res_ptr, n_elems) };
  // let mut res = unsafe{ build_vec(res_ptr, n_elems) };
  let layer = get_or_create(depth);
  for i in (0..coords.len()).step_by(2) {
    res[i >> 1] = layer.hash(coords[i], coords[i + 1]);
  }
  // Do this so that rust will not free the memory of the arrays (they are owned and will be free the caller)
  std::mem::forget(coords);
  std::mem::forget(res);
} */

#[no_mangle]
pub extern "C" fn hpx_hash_lonlat(depth: u8, num_coords: u32, lon: *mut f64, lat: *mut f64, ipixels: *mut u64) {
  let num_coords = num_coords as usize;
  
  let lon = unsafe{ build_array(lon, num_coords) };
  let lat = unsafe{ build_array(lat, num_coords) };

  let res = unsafe{ build_array_mut(ipixels, num_coords) };
  
  let layer = get_or_create(depth);
  for i in 0..num_coords {
      res[i] = layer.hash(lon[i], lat[i]);
  }
}
/*
/// Test if mutable pointers really works.
/// Again, we do not return a LonLat or an array of 2 doubles not to have to explicitly call a function
/// to free the memory on the Python side.
pub extern "C" fn hpx_center(depth: u8, hash: u64, lon: &mut f64, lat: &mut f64) {
  let (l, b) = center(depth, hash);
  *lon = l;
  *lat = b;
  // LonLat::from_tuple(center(depth, hash as u64))
}*/

/// We do not return an array of LonLat not to have to explicitly call a function
/// to free the memory on the Python side.
#[no_mangle]
pub extern "C" fn hpx_center_lonlat(depth: u8, num_ipixels: u32, ipixels: *mut u64, coords: *mut f64) {
  let num_ipixels = num_ipixels as usize;
  
  let hashs = unsafe{ build_array(ipixels, num_ipixels) };
  let res = unsafe{ build_array_mut(coords, num_ipixels << 1) };

  let layer = get_or_create(depth);
  
  for i in (0..res.len()).step_by(2) {
    let (l, b) = layer.center(hashs[i >> 1]);
    res[i] = l;
    res[i + 1] = b;
  }
}

/// The given array must be of size 8
/// [Slon, Slat, Elon, Elat, Nlon, Nlat, Slon, Slat]
pub extern "C" fn hpx_vertices(depth: u8, hash: u64, res_ptr: *mut f64) {
  let res = unsafe{ build_array_mut(res_ptr, 8 as usize) };
  let [(s_lon, s_lat), (e_lon, e_lat), (n_lon, n_lat), (w_lon, w_lat)] = vertices(depth, hash as u64);
  res[0] = s_lon;
  res[1] = s_lat;
  res[2] = e_lon;
  res[3] = e_lat;
  res[4] = n_lon;
  res[5] = n_lat;
  res[6] = w_lon;
  res[7] = w_lat;
  std::mem::forget(res_ptr);
}


/// The given array must be of size 9
/// `[S, SE, E, SW, C, NE, W, NW, N]`
pub extern "C" fn hpx_neighbours(depth: u8, hash: f64, res_ptr: *mut i64) {
  let res = unsafe{ build_array_mut(res_ptr, 9 as usize) };
  let neig_map = neighbours(depth, hash as u64, true);
  res[0] = to_i64(neig_map.get(MainWind::S));
  res[1] = to_i64(neig_map.get(MainWind::SE));
  res[2] = to_i64(neig_map.get(MainWind::E));
  res[3] = to_i64(neig_map.get(MainWind::SW));
  res[4] = hash as i64;
  res[5] = to_i64(neig_map.get(MainWind::NE));
  res[6] = to_i64(neig_map.get(MainWind::W));
  res[7] = to_i64(neig_map.get(MainWind::NW));
  res[8] = to_i64(neig_map.get(MainWind::N));
  // Do this so that rust will not free the memory of the arrays (they are owned and will be free the caller)
  std::mem::forget(res_ptr);
}

fn to_i64(val: Option<&u64>) -> i64 {
  match val {
    Some(&val) => val as i64,
    None => -1_i64,
  }
}


// https://doc.rust-lang.org/1.7.0/src/libc/lib.rs.html#109

#[derive(Debug)]
#[repr(C)]
pub struct PyBMOC {
  len: u32,
  data: *mut BMOCCell,
}

#[derive(Debug)]
#[repr(C)]
pub struct BMOCCell {
  depth: u8,
  hash: u64,
  flag: u8,
}

#[no_mangle]
pub extern "C" fn bmoc_free(ptr: *mut PyBMOC) {
  if !ptr.is_null() {
    unsafe { Box::from_raw(ptr) };
  }
}

/*#[no_mangle]
pub extern "C" fn length(ptr: *const BMOCCell) -> f64 {
  let array = unsafe {
    assert!(!ptr.is_null());
    &*ptr
  };
  array.length()
}*/


#[no_mangle]
pub extern "C" fn hpx_query_cone_approx(depth: u8, lon: f64, lat: f64, radius: f64) -> *mut PyBMOC {
  let mut cells = to_bmoc_cell_array(cone_overlap_approx(depth, lon, lat, radius));
  cells.shrink_to_fit();
  let data = cells.as_mut_ptr();
  let len = cells.len() as u32;
  std::mem::forget(cells);
  let bmoc = Box::new(PyBMOC {
    len,
    data,
  });
  Box::into_raw(bmoc)
}

#[no_mangle]
pub extern "C" fn hpx_query_cone_approx_custom(depth: u8, delta_depth: u8, lon: f64, lat: f64, radius: f64) -> *mut PyBMOC {
  let mut cells = to_bmoc_cell_array(cone_overlap_approx_custom(depth, delta_depth, lon, lat, radius));
  cells.shrink_to_fit();
  let data = cells.as_mut_ptr();
  let len = cells.len() as u32;
  std::mem::forget(cells);
  let bmoc = Box::new(PyBMOC {
    len,
    data,
  });
  Box::into_raw(bmoc)
}

#[no_mangle]
pub extern "C" fn hpx_query_polygon_approx(depth: u8, n_vertices: u32, vertices_ptr: *mut f64) -> *mut PyBMOC  { // *mut [BMOCCell]
  let n_vertices = n_vertices as usize;
  let vertices_coos = unsafe{ build_array(vertices_ptr, 2 * n_vertices) };
  let mut vertices: Vec<(f64, f64)> = Vec::with_capacity(n_vertices);
  for i in (0..n_vertices << 1).step_by(2) {
    vertices.push((vertices_coos[i], vertices_coos[i+1]));
  }
  // println!("entry len: {}", vertices.len());
  std::mem::forget(vertices_coos);
  let mut cells = to_bmoc_cell_array(polygon_overlap_approx(depth, &vertices.into_boxed_slice()));
  cells.shrink_to_fit();
  let ptr = cells.as_mut_ptr();
  let len = cells.len() as u32;
  std::mem::forget(cells);
  // Box::into_raw(cells)
  let bmoc = Box::new(PyBMOC {
    len,
    data: ptr, //Box::into_raw(cells),
  });
  Box::into_raw(bmoc)
}

fn to_bmoc_cell_array(bmoc: BMOC) -> Vec<BMOCCell> {
  let n_elems = bmoc.entries.len();
  let mut cells: Vec<BMOCCell> = Vec::with_capacity(n_elems);
  for raw_value in bmoc.entries.iter() {
    let cell = bmoc.from_raw_value(*raw_value);
    cells.push(BMOCCell {
      depth: cell.depth,
      hash: cell.hash,
      flag: cell.is_full as u8,
    });
  }
  cells
}

/*fn to_bmoc_cell_array(bmoc: BMOC) -> Box<[BMOCCell]> {
  let n_elems = bmoc.entries.len();
  let mut cells: Vec<BMOCCell> = Vec::with_capacity(n_elems);
  for raw_value in bmoc.entries.iter() {
    let cell = bmoc.from_raw_value(*raw_value);
    cells.push(BMOCCell {
      depth: cell.depth,
      hash: cell.hash,
      flag: cell.is_full as u8,
    });
  }
  cells.into_boxed_slice()
}*/
