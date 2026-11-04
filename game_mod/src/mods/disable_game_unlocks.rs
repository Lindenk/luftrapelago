use crate::{fn_hook::get_base_address, patcher::PatchBuilder, state::GameState};

pub fn disable_game_unlocks() {
  // skip over the check that displays unlocks on the results screen
  // also reset unlocks based on what we've gotten from the apworld.
  // note: this is a jank solution but it saves me hunting down wherever
  // the game is defering unlocks to
  let addr = (get_base_address() + 0xee19d) as *mut u8;
  let dest_addr = (get_base_address() + 0xee32c) as *mut u8;

  PatchBuilder::with_start_addr(addr)
    .call(rewrite_unlocks as extern "C" fn() as u32)
    .jump_to(dest_addr)
    .write();
}

extern "C" fn rewrite_unlocks() {
  GameState::with(|client, memory, _| {
    let unlocked_parts = client.get_parts();
    memory.set_unlocked_parts(unlocked_parts);
  });
}
