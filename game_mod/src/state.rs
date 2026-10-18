//! Although usually bad practice, the design here is to have a
//! global state machine that can be used by hooks. Since hooks
//! can be called whenever in the modded game (to an extent), this
//! should be a fairly safe approach

use std::sync::{LazyLock, Mutex};

use crate::{archipelago::client::APClient, game_data::stats::GlobalStats};

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
