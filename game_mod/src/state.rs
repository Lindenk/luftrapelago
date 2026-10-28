use std::sync::{LazyLock, Mutex};

use crate::{archipelago::client::APClient, game_data::memory::GameMemory};

static GAME_STATE: LazyLock<Mutex<GameState>> = LazyLock::new(|| Mutex::default());

#[derive(Default)]
pub struct GameState {
  client: Option<APClient>,
  memory: Option<GameMemory>,
}

impl GameState {
  pub fn init(client: APClient, memory: GameMemory) {
    let mut state = GAME_STATE.lock().unwrap();
    *state = Self {
      client: Some(client),
      memory: Some(memory),
    };
  }
  pub fn with<T, F: FnMut(&mut APClient, &GameMemory) -> T>(mut f: F) -> Option<T> {
    let mut state = GAME_STATE.lock().unwrap();
    let GameState { client, memory, .. } = &mut *state;
    if let (Some(client), Some(memory)) = (client.as_mut(), memory.as_mut()) {
      Some(f(client, memory))
    } else {
      None
    }
  }
}
