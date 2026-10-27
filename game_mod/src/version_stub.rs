use std::ffi::{c_char, c_void};
use std::sync::OnceLock;
use windows::Win32::Foundation::{BOOL, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

// stub the functions the game actually calls
type GetFileVersionInfoAFn =
  unsafe extern "system" fn(*const c_char, u32, u32, *mut c_void) -> BOOL;
type GetFileVersionInfoSizeAFn = unsafe extern "system" fn(*const c_char, *mut u32) -> u32;
type VerQueryValueAFn =
  unsafe extern "system" fn(*const c_void, *const c_char, *mut *mut c_void, *mut u32) -> BOOL;

type GetFileVersionInfoWFn = unsafe extern "system" fn(*const u16, u32, u32, *mut c_void) -> BOOL;
type GetFileVersionInfoSizeWFn = unsafe extern "system" fn(*const u16, *mut u32) -> u32;
type VerQueryValueWFn =
  unsafe extern "system" fn(*const c_void, *const u16, *mut *mut c_void, *mut u32) -> BOOL;

struct RealVersionDll {
  _handle: HMODULE,
  get_file_version_info_a: Option<GetFileVersionInfoAFn>,
  get_file_version_info_size_a: Option<GetFileVersionInfoSizeAFn>,
  ver_query_value_a: Option<VerQueryValueAFn>,
  get_file_version_info_w: Option<GetFileVersionInfoWFn>,
  get_file_version_info_size_w: Option<GetFileVersionInfoSizeWFn>,
  ver_query_value_w: Option<VerQueryValueWFn>,
}

unsafe impl Send for RealVersionDll {}
unsafe impl Sync for RealVersionDll {}

static REAL_VERSION: OnceLock<RealVersionDll> = OnceLock::new();

fn get_real_version() -> &'static RealVersionDll {
  REAL_VERSION.get_or_init(|| unsafe {
    let path = c"C:\\Windows\\System32\\version.dll";
    let handle = LoadLibraryA(windows::core::PCSTR(path.as_ptr() as *const u8))
      .expect("Failed to load system version.dll");

    let get_info_a = GetProcAddress(handle, windows::core::s!("GetFileVersionInfoA"))
      .map(|addr| std::mem::transmute(addr));
    let get_size_a = GetProcAddress(handle, windows::core::s!("GetFileVersionInfoSizeA"))
      .map(|addr| std::mem::transmute(addr));
    let query_val_a = GetProcAddress(handle, windows::core::s!("VerQueryValueA"))
      .map(|addr| std::mem::transmute(addr));

    let get_info_w = GetProcAddress(handle, windows::core::s!("GetFileVersionInfoW"))
      .map(|addr| std::mem::transmute(addr));
    let get_size_w = GetProcAddress(handle, windows::core::s!("GetFileVersionInfoSizeW"))
      .map(|addr| std::mem::transmute(addr));
    let query_val_w = GetProcAddress(handle, windows::core::s!("VerQueryValueW"))
      .map(|addr| std::mem::transmute(addr));

    RealVersionDll {
      _handle: handle,
      get_file_version_info_a: get_info_a,
      get_file_version_info_size_a: get_size_a,
      ver_query_value_a: query_val_a,
      get_file_version_info_w: get_info_w,
      get_file_version_info_size_w: get_size_w,
      ver_query_value_w: query_val_w,
    }
  })
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn GetFileVersionInfoA(
  lptstrfilename: *const c_char,
  dwhandle: u32,
  dwlen: u32,
  lpdata: *mut c_void,
) -> BOOL {
  let dll = get_real_version();
  if let Some(func) = dll.get_file_version_info_a {
    unsafe { func(lptstrfilename, dwhandle, dwlen, lpdata) }
  } else {
    BOOL(0)
  }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn GetFileVersionInfoSizeA(
  lptstrfilename: *const c_char,
  lpdwhandle: *mut u32,
) -> u32 {
  let dll = get_real_version();
  if let Some(func) = dll.get_file_version_info_size_a {
    unsafe { func(lptstrfilename, lpdwhandle) }
  } else {
    0
  }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn VerQueryValueA(
  pblock: *const c_void,
  lpstrsubblock: *const c_char,
  lplpbuffer: *mut *mut c_void,
  puisize: *mut u32,
) -> BOOL {
  let dll = get_real_version();
  if let Some(func) = dll.ver_query_value_a {
    unsafe { func(pblock, lpstrsubblock, lplpbuffer, puisize) }
  } else {
    BOOL(0)
  }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn GetFileVersionInfoW(
  lptstrfilename: *const u16,
  dwhandle: u32,
  dwlen: u32,
  lpdata: *mut c_void,
) -> BOOL {
  let dll = get_real_version();
  if let Some(func) = dll.get_file_version_info_w {
    unsafe { func(lptstrfilename, dwhandle, dwlen, lpdata) }
  } else {
    BOOL(0)
  }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn GetFileVersionInfoSizeW(
  lptstrfilename: *const u16,
  lpdwhandle: *mut u32,
) -> u32 {
  let dll = get_real_version();
  if let Some(func) = dll.get_file_version_info_size_w {
    unsafe { func(lptstrfilename, lpdwhandle) }
  } else {
    0
  }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn VerQueryValueW(
  pblock: *const c_void,
  lpstrsubblock: *const u16,
  lplpbuffer: *mut *mut c_void,
  puisize: *mut u32,
) -> BOOL {
  let dll = get_real_version();
  if let Some(func) = dll.ver_query_value_w {
    unsafe { func(pblock, lpstrsubblock, lplpbuffer, puisize) }
  } else {
    BOOL(0)
  }
}
