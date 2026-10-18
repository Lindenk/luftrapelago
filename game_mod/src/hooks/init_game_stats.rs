use std::mem;

use crate::{
  archipelago::{client, items::APItem},
  fn_hooks::MasterGameInitHook,
  game_data::stats::GlobalStats,
  state::{GAME_STATE, GameConnected, GameInitializing, GameState},
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
        let mut client = match old_state {
          GameState::Intializing(g) => g.client,
          // Safety: We checked for this above and hold a lock on GameState
          _ => unreachable!(),
        };
        println!("All initial items {:?}", client.client().received_items());

        let initial_items: Vec<_> = client
          .client()
          .received_items()
          .iter()
          .filter_map(|i| APItem::try_from(i.item().id()).ok())
          .collect();
        let weapons: Vec<_> = initial_items.iter().filter_map(|i| i.as_weapon()).collect();
        let bodies: Vec<_> = initial_items.iter().filter_map(|i| i.as_body()).collect();
        let engines: Vec<_> = initial_items.iter().filter_map(|i| i.as_engine()).collect();

        println!(
          "Setting initial build {:?} {:?} {:?} {:?}",
          initial_items, weapons, bodies, engines
        );
        stats.set_initial_parts(weapons.as_slice(), bodies.as_slice(), engines.as_slice());

        // finish transition
        *state = GameState::Connected(GameConnected { client, stats });

        println!("Successfully initialized game state");
      }
    }

    res
  });
  let _ = unsafe { MasterGameInitHook.enable() };
}
