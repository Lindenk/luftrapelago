use crate::{
  fn_hooks::{CreateFileAHook, FiopenHook, get_base_address},
  util::patch_memory,
};

pub fn redirect_saves(path_suffix: &str) {
  // Private helper to handle the common path inspection and suffix appending logic
  let redirect_path = |c_str: &std::ffi::CStr, suffix: &str| -> Option<std::ffi::CString> {
    let path = c_str.to_string_lossy();
    if path.contains("save") {
      let bytes = c_str.to_bytes();
      let mut new_bytes = Vec::with_capacity(bytes.len() + 1 + suffix.len());
      new_bytes.extend_from_slice(bytes);
      new_bytes.push(b'_');
      new_bytes.extend_from_slice(suffix.as_bytes());
      std::ffi::CString::new(new_bytes).ok()
    } else {
      None
    }
  };

  // Redirect file opens
  let suffix_fiopen = path_suffix.to_string();
  let _ = FiopenHook.init_with_self(move |filename, mode, prot| {
    if filename.is_null() {
      return FiopenHook.call(filename, mode, prot);
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(filename) };

    if let Some(new_cstr) = redirect_path(c_str, &suffix_fiopen) {
      println!(
        "Redirecting save file via Fiopen: {}",
        new_cstr.to_string_lossy()
      );
      FiopenHook.call(new_cstr.as_ptr(), mode, prot)
    } else {
      FiopenHook.call(filename, mode, prot)
    }
  });
  let _ = unsafe { FiopenHook.enable() };

  // Redirect file creation
  let suffix_create = path_suffix.to_string();
  let _ = CreateFileAHook.init_with_self(
    move |lp_file_name,
          desired_access,
          share_mode,
          sec_attr,
          creation_disp,
          flags_attrs,
          template_file| {
      if lp_file_name.is_null() {
        return CreateFileAHook.call(
          lp_file_name,
          desired_access,
          share_mode,
          sec_attr,
          creation_disp,
          flags_attrs,
          template_file,
        );
      }

      let c_str = unsafe { std::ffi::CStr::from_ptr(lp_file_name) };

      if let Some(new_cstr) = redirect_path(c_str, &suffix_create) {
        println!(
          "Redirecting save file via CreateFileA: {}",
          new_cstr.to_string_lossy()
        );
        CreateFileAHook.call(
          new_cstr.as_ptr(),
          desired_access,
          share_mode,
          sec_attr,
          creation_disp,
          flags_attrs,
          template_file,
        )
      } else {
        CreateFileAHook.call(
          lp_file_name,
          desired_access,
          share_mode,
          sec_attr,
          creation_disp,
          flags_attrs,
          template_file,
        )
      }
    },
  );
  let _ = unsafe { CreateFileAHook.enable() };
}

pub fn redirect_saves_patch(path_suffix: &str) {
  let path = format!("ap_{}", path_suffix);
  let c_string = std::ffi::CString::new(path.clone()).unwrap();
  let new_ptr = c_string.as_ptr() as u32;
  let new_len = path.len() as u8;
  std::mem::forget(c_string);

  // 2. Patch the length byte (1 byte)
  unsafe {
    if !patch_memory((get_base_address() + 0x248ee1) as *mut u8, &[new_len]) {
      println!("Failed to patch string length!");
      return;
    }

    // 3. Patch the pointer (4 bytes, converted to little-endian bytes)
    if !patch_memory(
      (get_base_address() + 0x248ee3) as *mut u8,
      &new_ptr.to_le_bytes(),
    ) {
      println!("Failed to patch string pointer!");
      return;
    }
  }

  println!("Successfully patched instructions for: {}", path);
}
