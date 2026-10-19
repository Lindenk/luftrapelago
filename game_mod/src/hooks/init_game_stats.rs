use std::mem;

use crate::{
  archipelago::items::APItem,
  fn_hooks::MasterGameInitHook,
  game_data::stats::GlobalStats,
  state::{GAME_STATE, GameConnected, GameState},
};

pub fn init_game_stats() {
  // hook right after the game successfully loads it's save file and grab
  // the global game stats pointer, and modify the starting data as needed
  let _ = MasterGameInitHook.init_with_self(move |param1| {
    println!("Calling original init game function");
    let res = MasterGameInitHook.call(param1);

    println!(
      "Checking for state transition away from Intializing: {:x}",
      param1 as usize
    );
    {
      let mut state = GAME_STATE.lock().unwrap();
      if matches!(*state, GameState::Intializing(_)) {
        println!("Initializing global stats ptr...");

        // init our global stats and update the game state
        let stats = GlobalStats::init();
        stats.enable_hangar();

        // transition to connected
        let old_state = mem::replace(&mut *state, GameState::Disconnected);
        let client = match old_state {
          GameState::Intializing(g) => g.client,
          // Safety: We checked for this above and hold a lock on GameState
          _ => unreachable!(),
        };

        let mut game_connected = GameConnected { client, stats };
        game_connected.update_unlocks();
        game_connected.set_default_loadout();

        // finish transition
        *state = GameState::Connected(game_connected);

        println!("Successfully initialized game state");
      }
    }

    res
  });
  let _ = unsafe { MasterGameInitHook.enable() };
}
