//! Although usually bad practice, the design here is to have a
//! global state machine that can be used by hooks. Since hooks
//! can be called whenever in the modded game (to an extent), this
//! should be a fairly safe approach

use std::sync::{LazyLock, Mutex};

use crate::{
  archipelago::{client::APClient, items::APItem},
  game_data::stats::GlobalStats,
};

pub static GAME_STATE: LazyLock<Mutex<GameState>> =
  LazyLock::new(|| Mutex::new(GameState::Disconnected));

pub enum GameState {
  Disconnected,
  Intializing(GameInitializing),
  Connected(GameConnected),
}

pub struct GameInitializing {
  pub client: APClient,
}

pub struct GameConnected {
  pub client: APClient,
  pub stats: GlobalStats,
}

// utility functions for the game. TODO: Organize this better
impl GameConnected {
  // reset the game's unlocks based on what we've gotten from the apworld
  pub fn update_unlocks(&mut self) {
    let items: Vec<_> = self
      .client
      .client()
      .received_items()
      .iter()
      .filter_map(|i| APItem::try_from(i.item().id()).ok())
      .collect();
    let weapons: Vec<_> = items.iter().filter_map(|i| i.as_weapon()).collect();
    let bodies: Vec<_> = items.iter().filter_map(|i| i.as_body()).collect();
    let engines: Vec<_> = items.iter().filter_map(|i| i.as_engine()).collect();

    self.stats.set_unlocked_parts(&weapons, &bodies, &engines);
  }

  pub fn set_default_loadout(&mut self) {
    let items: Vec<_> = self
      .client
      .client()
      .received_items()
      .iter()
      .filter_map(|i| APItem::try_from(i.item().id()).ok())
      .collect();
    let weapon = items.iter().filter_map(|i| i.as_weapon()).next();
    let body = items.iter().filter_map(|i| i.as_body()).next();
    let engine = items.iter().filter_map(|i| i.as_engine()).next();

    match (weapon, body, engine) {
      (Some(w), Some(b), Some(e)) => self.stats.set_parts(w, b, e),
      _ => (),
    }
  }
}
