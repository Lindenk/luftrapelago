use windows::Win32::System::Memory::{
  PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};

/// Safely patches a region of memory at the given address with a slice of bytes.
/// Handles changing protection to PAGE_EXECUTE_READWRITE and restoring it afterward.
pub unsafe fn patch_memory(address: *mut u8, bytes: &[u8]) -> bool {
  let size = bytes.len();
  if size == 0 {
    return true;
  }

  let mut old_protect = PAGE_PROTECTION_FLAGS(0);

  // 1. Change memory protection to allow writing
  unsafe {
    if VirtualProtect(
      address as *const _,
      size,
      PAGE_EXECUTE_READWRITE,
      &mut old_protect,
    )
    .is_err()
    {
      println!("Failed to change memory protection at {:p}", address);
      return false;
    }
  }

  // 2. Write bytes volatile to prevent compiler optimizations
  for (i, &byte) in bytes.iter().enumerate() {
    unsafe {
      std::ptr::write_volatile(address.add(i), byte);
    }
  }

  // 3. Restore original memory protection
  let _ = unsafe {
    VirtualProtect(address as *const _, size, old_protect, &mut old_protect);
  };

  true
}
