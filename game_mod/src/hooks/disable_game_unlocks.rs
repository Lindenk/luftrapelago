use crate::{
  archipelago::items::APItem,
  fn_hooks::{DisplayUnlocksHook, get_base_address},
  state::{GAME_STATE, GameState},
  util::{patch_fn_call_with_jump, patch_memory},
};
use detour::signature;

pub fn disable_game_unlocks() {
  // skip over the check that displays unlocks on the results screen
  unsafe {
    let addr = (get_base_address() + 0xee19d) as *mut u8;
    let dest_addr = (get_base_address() + 0xee32c) as *mut u8;
    //patch_memory(ptr, &[0xe9, 0x8a, 0x01, 0x00, 0x00, 0x90]);
    patch_fn_call_with_jump(addr, rewrite_unlocks, dest_addr);
  }

  // when entering the results screen, clear any "unlocks" the player
  // got while playing, which should disable the game from giving the
  // player items they shouldn't (and the big popup)
  /*
  let _ = DisplayUnlocksHook.init_with_self(move |param1| {
    println!("stubbed a display thing");

    /*unsafe {
      // skip the check that triggers mission completion
      let ptr = (get_base_address() + 0x004b0618) as *mut u8;
      patch_memory(ptr, &[0x90, 0xE9]);
    }*/

    /*{
      let state = GAME_STATE.lock().unwrap();
      if let GameState::Connected(state) = &*state {
        state.stats.clear_game_unlock_buffer();
      }
    }*/

    //DisplayUnlocksHook.call(param1);
  });
  let _ = unsafe { DisplayUnlocksHook.enable() };
  */
}

extern "C" fn rewrite_unlocks() {
  let mut state = GAME_STATE.lock().unwrap();
  if let GameState::Connected(state) = &mut *state {
    let items: Vec<_> = state
      .client
      .client()
      .received_items()
      .iter()
      .filter_map(|i| APItem::try_from(i.item().id()).ok())
      .collect();

    let weapons: Vec<_> = items.iter().filter_map(|i| i.as_weapon()).collect();
    let bodies: Vec<_> = items.iter().filter_map(|i| i.as_body()).collect();
    let engines: Vec<_> = items.iter().filter_map(|i| i.as_engine()).collect();
    state.stats.set_unlocked_parts(&weapons, &bodies, &engines);
  }
}
