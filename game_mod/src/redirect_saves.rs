use std::ffi::CString;

use windows::core::PCSTR;

use crate::fn_hooks::FiopenHook;

pub fn redirect_saves(path_suffix: &str) {
  let suffix = path_suffix.to_string();
  let _ = FiopenHook.init_with_self(move |filename, mode, prot| {
    let c_str = unsafe { std::ffi::CStr::from_ptr(filename) };

    let path = c_str.to_string_lossy();
    if path.contains("save") {
      println!("Save data access intercepted, {}", path);

      let bytes = c_str.to_bytes();
      let mut new_bytes = Vec::with_capacity(bytes.len() + 1 + suffix.len());
      new_bytes.extend_from_slice(bytes);
      new_bytes.push(b'_');
      new_bytes.extend_from_slice(suffix.as_bytes());
      let new_cstr = std::ffi::CString::new(new_bytes)
        .expect("Unparsable save data path somehow. This shouldn't be able to happen.");

      println!("Redirecting save file via direct byte-suffix append");
      FiopenHook.call(new_cstr.as_ptr(), mode, prot)
    } else {
      FiopenHook.call(filename, mode, prot)
    }
  });
  let _ = unsafe { FiopenHook.enable() };
}
