use crate::{
  define_detour_hooks,
  fn_hook::get_base_address,
  game_data::plane::{BodyType, EngineType, Part, WeaponType},
  patcher::{CodePatch, Reg, patch_memory},
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
    patch_memory(ptr, &[0x90, 0x90, 0x90, 0x90, 0x90, 0x90]);
    let ptr = (get_base_address() + 0x744b8) as *mut u8;
    patch_memory(ptr, &[0x90, 0x90, 0x90, 0x90, 0x90, 0x90]);
    let ptr = (get_base_address() + 0x7451b) as *mut u8;
    patch_memory(ptr, &[0x90, 0x90]);

    // scroll left unlock check
    let ptr = (get_base_address() + 0x74716) as *mut u8;
    patch_memory(ptr, &[0x90, 0x90, 0x90, 0x90, 0x90, 0x90]);
    let ptr = (get_base_address() + 0x74775) as *mut u8;
    patch_memory(ptr, &[0x90, 0x90, 0x90, 0x90, 0x90, 0x90]);
    let ptr = (get_base_address() + 0x747d1) as *mut u8;
    patch_memory(ptr, &[0x90, 0x90]);

    // don't let the player leave the hangar edit mode with an invalid build
    // this is the check for stopping the player from leaving edit mode with
    // some, but not all, of the urauser parts. Replace the check with the
    // same logic but with an additional check for if the selected parts are
    // also unlocked
    /*{
      let patch_start = (get_base_address() + 0x73e54) as *mut u8;
      let patch_true_branch = get_base_address() + 0x73e95;
      let patch_false_branch = (get_base_address() + 0x73f0f) as *mut u8;
      CodePatch::at(patch_start)
        .arg_reg(Reg::Eax)
        .payload(check_valid_build as extern "C" fn(*const u8) -> u32)
        .branch_if_nonzero(patch_true_branch)
        .continuation(patch_false_branch)
        .postamble(vec![
          0x8a, 0x93, 0xb9, 0x01, 0x00, 0x00, // load DL
          0x8a, 0x83, 0xba, 0x01, 0x00, 0x00, // load AL
        ])
        .write();
    }*/
    {
      let patch_start = get_base_address() + 0x73e54;
      let patch_true_branch = get_base_address() + 0x73e95;
      let patch_false_branch = get_base_address() + 0x73f0f;
      let mut patch_bytes = vec![
        // runs on both branches, just handle it now
        0x8a, 0x8b, 0xb8, 0x01, 0x00, 0x00, // load stack var
        0x60, 0x9c, // PUSHAD, PUSHFD
        0x53, // pass in the host function's param_1
        0xb8, // MOV EAX, imm32 (setup the function call)
      ];
      // patch in our check
      let func_addr = check_valid_build as extern "C" fn(*const u8) -> bool as usize as u32;
      patch_bytes.extend_from_slice(&func_addr.to_le_bytes());

      // result is in AL, patch in branch
      patch_bytes.extend_from_slice(&[
        0xFF, 0xD0, // CALL EAX
        0x83, 0xC4, 0x04, // clean up stack arg
        0x84, 0xc0, // TEST AL, AL
      ]);

      let post_branch_setup = vec![
        0x9D, 0x61, // POPFD, POPAD
        0x8a, 0x93, 0xb9, 0x01, 0x00, 0x00, // load DL
        0x8a, 0x83, 0xba, 0x01, 0x00, 0x00, // load AL
      ]; // will include 5 more bytes for a jump to the real branch

      // calc our relative jump to the false branch
      // our offset is the length of the false branch, followed by the missing
      // 5 bytes for the jump to the real branch
      let false_branch_offset = (post_branch_setup.len() + 5) as i32;
      patch_bytes.extend_from_slice(&[0x0f, 0x84]); // JZ
      patch_bytes.extend_from_slice(&false_branch_offset.to_le_bytes());

      // write in our true branch
      patch_bytes.extend_from_slice(&post_branch_setup);
      let true_branch_offset = (patch_true_branch - (patch_start + patch_bytes.len() + 5)) as i32;
      patch_bytes.push(0xe9); // jmp
      patch_bytes.extend(true_branch_offset.to_le_bytes());

      // write in our false branch
      patch_bytes.extend_from_slice(&post_branch_setup);
      let false_branch_offset = (patch_false_branch - (patch_start + patch_bytes.len() + 5)) as i32;
      patch_bytes.push(0xe9); // jmp
      patch_bytes.extend(false_branch_offset.to_le_bytes());

      // sanity check
      assert!((patch_false_branch > (patch_start + patch_bytes.len())));

      // write our patch in
      patch_memory(patch_start as *mut u8, &patch_bytes);
    }
  }
}

extern "C" fn check_valid_build(hangar_data: *const u8) -> bool {
  let unlocked_parts: Vec<_> = GameState::with(|client, _| client.get_parts().collect())
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
