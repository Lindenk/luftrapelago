use crate::{fn_hook::get_base_address, patcher::PatchBuilder, state::GameState};

pub fn set_up_rank_keys() {
  // sfmt and blimp kill mission check
  //unsafe {
  //  let addr = (get_base_address() + 0xaf917) as *mut u8;
  //  patch_memory(addr, &[0xb0, 0x00, 0xc3]); // force return false
  //}
  /*unsafe {
    let addr = (get_base_address() + 0xaf910) as *mut u8;
    let mut patch = vec![
      0x50, //push eax, the actual arg to our function
      0xb8, // set up our fn call
    ];
    let fn_ptr = check_keys as extern "stdcall" fn(*const i8) -> bool as u32;
    patch.extend(fn_ptr.to_le_bytes());
    patch.extend([
      0xff, 0xd0, //call eax
      0xc3, //ret
    ]);
    patch_memory(addr, &patch);
  }*/
  {
    let addr = (get_base_address() + 0xaf910) as *mut u8;
    PatchBuilder::with_start_addr(addr)
      .append(&[0x50]) //push eax, the actual arg to our function
      .call(check_keys as extern "stdcall" fn(*const i8) -> bool as u32)
      .append(&[0xc3]) //ret
      .write();
  }
}

// mimic the sfmt fn so we return true if this is an invalid mission
// or false if it isn't (yes I know it's backwards)
extern "stdcall" fn check_keys(mission_descriptor: *const i8) -> bool {
  let mission_num = unsafe { mission_descriptor.byte_add(2).read_volatile() };
  // do the game's check for if this mission is valid
  if mission_num == -1 {
    return true; // invalid
  }

  // check rank keys
  GameState::with(|client, _, _| {
    !client.has_enough_rank_keys(mission_num as u8) // false is valid
  })
  .unwrap_or(false)
}
