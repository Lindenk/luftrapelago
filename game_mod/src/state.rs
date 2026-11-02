use std::{
  sync::{LazyLock, Mutex},
  time::Instant,
};

use crate::{archipelago::client::APClient, game_data::memory::GameMemory};

static GAME_STATE: LazyLock<Mutex<GameState>> = LazyLock::new(|| Mutex::default());

// for state that changes often
#[derive(Default)]
pub struct GameTempState {
  pub patchup_active: bool,
  pub current_max_hp: f32,
  pub deathlink_triggered: bool,
  pub deathlink_counter: u8,
  pub gunjamed_timeout: Option<Instant>,
  pub engine_stall_timeout: Option<Instant>,
}

#[derive(Default)]
pub struct GameState {
  client: Option<APClient>,
  memory: Option<GameMemory>,
  temp_state: GameTempState,
}

impl GameState {
  pub fn init(client: APClient, memory: GameMemory) {
    let mut state = GAME_STATE.lock().unwrap();
    *state = Self {
      client: Some(client),
      memory: Some(memory),
      temp_state: Default::default(),
    };
  }
  pub fn with<T, F: FnMut(&mut APClient, &GameMemory, &mut GameTempState) -> T>(
    mut f: F,
  ) -> Option<T> {
    let mut state = GAME_STATE.lock().unwrap();
    let GameState {
      client,
      memory,
      temp_state,
    } = &mut *state;
    if let (Some(client), Some(memory)) = (client.as_mut(), memory.as_mut()) {
      Some(f(client, memory, temp_state))
    } else {
      None
    }
  }
}
