use crate::{
  fn_hooks::get_base_address,
  patcher::CodePatch,
  state::{GAME_STATE, GameState},
};

pub fn disable_game_unlocks() {
  // skip over the check that displays unlocks on the results screen
  // also reset unlocks based on what we've gotten from the apworld.
  // note: this is a jank solution but it saves me hunting down wherever
  // the game is defering unlocks to
  unsafe {
    let addr = (get_base_address() + 0xee19d) as *mut u8;
    let dest_addr = (get_base_address() + 0xee32c) as *mut u8;
    //patch_fn_call_with_jump(addr, rewrite_unlocks, dest_addr);

    CodePatch::at(addr)
      .payload(rewrite_unlocks as extern "C" fn())
      .continuation(dest_addr)
      .write();
  }
}

extern "C" fn rewrite_unlocks() {
  let mut state = GAME_STATE.lock().unwrap();
  if let GameState::Connected(state) = &mut *state {
    state.update_unlocks();
  }
}
