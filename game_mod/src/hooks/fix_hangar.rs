use crate::{
  fn_hooks::get_base_address,
  game_data::plane::{BodyType, EngineType, WeaponType},
  state::{GAME_STATE, GameState},
  util::{patch_fn_call, patch_fn_call_with_jump, patch_memory},
};

// By default the game expects unlocked parts to be sequantial, and will
// incorrectly cycle through them if there are any gaps. Also it assumes
// the original is always unlocked. So patch it to fill in the first available
// parts when entering the screen, and fix the cycling logic so it selects
// the next unlocked part
pub fn fix_hangar_screen() {
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
    {
      let START = get_base_address() + 0x73e54;
      let TRUE_BRANCH = get_base_address() + 0x73e95;
      let FALSE_BRANCH = get_base_address() + 0x73f0f;
      let mut patch_bytes = vec![
        // runs on both branches, just handle it now
        0x8a, 0x8b, 0xb8, 0x01, 0x00, 0x00, // load stack var
        0x60, 0x9c, // PUSHAD, PUSHFD
        0x53, // pass in the host function's param_1
        0xb8, // MOV EAX, imm32 (setup the function call)
      ];
      // patch in our check
      let func_addr = check_valid_build as usize as u32;
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
      let true_branch_offset = (TRUE_BRANCH - (START + patch_bytes.len() + 5)) as i32;
      patch_bytes.push(0xe9); // jmp
      patch_bytes.extend(true_branch_offset.to_le_bytes());

      // write in our false branch
      patch_bytes.extend_from_slice(&post_branch_setup);
      let false_branch_offset = (FALSE_BRANCH - (START + patch_bytes.len() + 5)) as i32;
      patch_bytes.push(0xe9); // jmp
      patch_bytes.extend(false_branch_offset.to_le_bytes());

      // sanity check
      assert!((FALSE_BRANCH > (START + patch_bytes.len())));

      // write our patch in
      patch_memory(START as *mut u8, &patch_bytes);
    }
  }
}

extern "C" fn check_valid_build(hangar_data: *const u8) -> bool {
  let mut state = GAME_STATE.lock().unwrap();
  if let GameState::Connected(state) = &mut *state {
    let stats = state.stats;

    println!("Pointer received from fn: {:x}", hangar_data as usize);

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

    let is_valid = match parts {
      (Some(w), Some(b), Some(e)) => {
        println!("Parts selected: {:?}, {:?}, {:?}", w, b, e);

        // make sure all parts selected are unlocked
        let is_unlocked =
          stats.is_weapon_unlocked(w) && stats.is_body_unlocked(b) && stats.is_engine_unlocked(e);
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
    };

    is_valid
  } else {
    println!("Failed to get state while exiting hangar edit!");
    false
  }
}
