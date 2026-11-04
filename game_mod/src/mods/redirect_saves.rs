use crate::{fn_hook::get_base_address, patcher::patch_memory};

//
pub fn redirect_saves(path_suffix: &str) {
  let path = format!("ap_{}", path_suffix);
  let c_string = std::ffi::CString::new(path.clone()).unwrap();
  let new_ptr = c_string.as_ptr() as u32;
  let new_len = path.len() as u8;
  std::mem::forget(c_string);

  // patch the save file string to include our ap seed (so it's custom for this apworld)
  unsafe {
    patch_memory((get_base_address() + 0x248ee1) as *mut u8, &[new_len]);

    // patch the ptr to the save game location str
    patch_memory(
      (get_base_address() + 0x248ee3) as *mut u8,
      &new_ptr.to_le_bytes(),
    );
  }
}
