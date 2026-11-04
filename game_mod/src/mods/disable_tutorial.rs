use crate::{fn_hook::get_base_address, patcher::patch_memory};

pub fn disable_tutorial() {
  // patch out the check that triggers it
  let ptr = (get_base_address() + 0x12577b) as *mut u8;

  // patch the flags it should set and jump past the tutorial
  unsafe {
    patch_memory(
      ptr,
      &[
        0xc6, 0x80, 0x89, 0x27, 0x00, 0x00, 0x01, // set global_stats[0x2789] to 1
        0xc6, 0x40, 0x14, 0x00, // set global_stats[0x14] to 0
        0xeb, 0x5b, // jump past the tutorial init
      ],
    );
  }
}
