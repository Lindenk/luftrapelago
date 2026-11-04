use crate::{
  define_detour_hooks,
  fn_hook::get_base_address,
  game_data::plane::{BodyType, EngineType, Part, WeaponType},
  patcher::{PatchBuilder, patch_memory},
  state::GameState,
};

define_detour_hooks! {
  static RandomUnlockCheck: extern "thiscall" fn(u8, u8) -> u32 => 0x6ee00;
}

// By default the game expects unlocked parts to be sequantial, and will
// incorrectly cycle through them if there are any gaps. Also it assumes
// the original is always unlocked. So patch it to fill in the first available
// parts when entering the screen, and fix the cycling logic so it selects
// the next unlocked part
pub fn fix_hangar_screen() {
  // always allow random as a selection
  //let _ = RandomUnlockCheckHook.init_with_self(|_, _| 0);
  //let _ = unsafe { RandomUnlockCheckHook.enable() };

  // disable the hangar icon unlock
  unsafe {
    let ptr = (get_base_address() + 0x7205f) as *mut u8;
    // loads the flag to check for if RANDOM is unlocked (6, 7, and 8 are unused
    // so they should also be false)
    patch_memory(ptr, &[0xba, 0x07, 0x00, 0x00, 0x00]);
  }

  // nop the if statement jumps so we're always allowed to cycle
  unsafe {
    // scroll right unlock check
    let ptr = (get_base_address() + 0x74452) as *mut u8;
    patch_memory(ptr, &[0x90; 6]);
    let ptr = (get_base_address() + 0x744b8) as *mut u8;
    patch_memory(ptr, &[0x90; 6]);
    let ptr = (get_base_address() + 0x7451b) as *mut u8;
    patch_memory(ptr, &[0x90; 2]);

    // scroll left unlock check
    let ptr = (get_base_address() + 0x74716) as *mut u8;
    patch_memory(ptr, &[0x90; 6]);
    let ptr = (get_base_address() + 0x74775) as *mut u8;
    patch_memory(ptr, &[0x90; 6]);
    let ptr = (get_base_address() + 0x747d1) as *mut u8;
    patch_memory(ptr, &[0x90; 2]);

    // don't let the player leave the hangar edit mode with an invalid build
    // this is the check for stopping the player from leaving edit mode with
    // some, but not all, of the urauser parts. Replace the check with the
    // same logic but with an additional check for if the selected parts are
    // also unlocked
    {
      let addr = (get_base_address() + 0x73e54) as *mut u8;
      let true_branch = (get_base_address() + 0x73e95) as *const u8;
      let false_branch = (get_base_address() + 0x73f0f) as *const u8;
      PatchBuilder::with_start_addr(addr)
        .append(&[
          // set up registers for later
          0x8a, 0x8b, 0xb8, 0x01, 0x00, 0x00, // load stack var
          0x8a, 0x93, 0xb9, 0x01, 0x00, 0x00, // load DL
          0x8a, 0x83, 0xba, 0x01, 0x00, 0x00, // load AL
        ])
        .push_all()
        .append(&[0x53]) // pass in ebx
        .call(check_valid_build as extern "C" fn(*const u8) -> bool as u32)
        .append(&[0x83, 0xC4, 0x04]) // clean up stack
        .pop_jz(false_branch)
        .pop_all()
        .jump_to(true_branch)
        .write();
    }
  }
}

extern "C" fn check_valid_build(hangar_data: *const u8) -> bool {
  let unlocked_parts: Vec<_> = GameState::with(|client, _, _| client.get_parts().collect())
    .expect("Somehow we checked the hangar build without an ap client!");

  let parts = {
    let w = WeaponType::try_from(unsafe {
      hangar_data
        .byte_offset(0x1b8)
        .read_volatile()
        .wrapping_sub(1)
    })
    .ok();
    let b = BodyType::try_from(unsafe {
      hangar_data
        .byte_offset(0x1b9)
        .read_volatile()
        .wrapping_sub(1)
    })
    .ok();
    let e = EngineType::try_from(unsafe {
      hangar_data
        .byte_offset(0x1ba)
        .read_volatile()
        .wrapping_sub(1)
    })
    .ok();
    (w, b, e)
  };

  match parts {
    (Some(w), Some(b), Some(e)) => {
      // make sure all parts selected are unlocked
      let is_unlocked = [Part::from(w), Part::from(b), Part::from(e)]
        .iter()
        .all(|p| unlocked_parts.contains(p));
      // make sure we have either selected all urauser parts or none
      let is_valid_urauser = match (w, b, e) {
        (WeaponType::URauser, BodyType::URauser, EngineType::URauser) => true,
        (WeaponType::URauser, _, _) => false,
        (_, BodyType::URauser, _) => false,
        (_, _, EngineType::URauser) => false,
        _ => true,
      };
      is_unlocked && is_valid_urauser
    }
    _ => {
      println!("Failed to get valid parts!");
      false
    }
  }
}
