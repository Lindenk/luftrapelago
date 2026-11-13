use std::sync::{Arc, Mutex};

use crate::{
  archipelago::client::APClient,
  define_detour_hooks,
  game_data::{
    memory::GameMemory,
    plane::{BodyType, EngineType, Part, WeaponType},
  },
  memory_matcher::MemoryMatcher,
  state::GameState,
};

define_detour_hooks! {
  static MasterGameInit: extern "stdcall" fn(*mut std::ffi::c_void) -> () => 0x9d8b0;
}

pub fn init_mod(ap_client: APClient) {
  let client = Arc::new(Mutex::new(Some(ap_client)));

  // hook right after the game successfully loads it's save file and grab
  // the global game stats pointer, and modify the starting data as needed
  let _ = MasterGameInitHook.init_with_self(move |param1| {
    println!("Attempting to find a match...");
    let _ = MemoryMatcher::new()
      .addr_at()
      .skip(8)
      .match_bytes(&[0x89, 0x74, 0x24, 0x3c, 0x89, 0x7c, 0x24, 0x38, 0xc6])
      .skip(4)
      .match_bytes(&[0xe8])
      .follow_rel()
      .match_bytes(&[0x55, 0x8b, 0xec]);

    println!("Calling original init game function");
    let res = MasterGameInitHook.call(param1);

    // init the state through absolute rust-forced jank
    // (although tbf if this triggers more than once somehow, this stops the
    // mod from initing 2+ times)
    if let Some(client) = (*client.clone().lock().unwrap()).take() {
      GameState::init(client, GameMemory::init());
      GameState::with(|client, memory, _| {
        memory.enable_hangar();
        memory.set_sfmt(client.client().slot_data().enable_sfmt);

        // update our unlocked parts
        let unlocked_parts: Vec<_> = client.get_parts().collect();
        memory.set_unlocked_parts(unlocked_parts.clone().into_iter());

        // set our default loadout if the save doesn't have unlocked parts equipped
        set_default_parts(memory, &unlocked_parts);

        // run one update tick to fetch one-time items that we don't want
        // to trigger at the start
        client.client().update();
      });
    }

    res
  });
  let _ = unsafe { MasterGameInitHook.enable() };
}

fn set_default_parts(memory: &GameMemory, unlocked_parts: &[Part]) {
  if let (Some(w), Some(b), Some(e)) = (
    memory.get_equipped_weapon(),
    memory.get_equipped_body(),
    memory.get_equipped_engine(),
  ) {
    if [Part::from(w), Part::from(b), Part::from(e)]
      .into_iter()
      .any(|p| !unlocked_parts.contains(&p))
    {
      // set the first part in each category
      let w = unlocked_parts
        .iter()
        .find_map(|p| WeaponType::try_from(*p).ok());
      let b = unlocked_parts
        .iter()
        .find_map(|p| BodyType::try_from(*p).ok());
      let e = unlocked_parts
        .iter()
        .find_map(|p| EngineType::try_from(*p).ok());
      if let (Some(w), Some(b), Some(e)) = (w, b, e) {
        memory.set_parts(w, b, e);
      }
    }
  }
}
