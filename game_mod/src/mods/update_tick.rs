use std::{
  sync::Mutex,
  time::{Duration, Instant},
};

use crate::{define_detour_hooks, mods::check_hints, state::GameState};

define_detour_hooks! {
  static UpdateTick: extern "thiscall" fn(*mut std::ffi::c_void) -> () => 0xa1510;
}

// Apparently the main loop ticks at loop speed so instead of finding
// another hook spot (that might trigger partway through an update), let's
// just limit based on actual time
static LAST: Mutex<Option<Instant>> = Mutex::new(None);
const INTERVAL: Duration = Duration::from_millis(100);

pub fn hook_update_tick() {
  // we need a place to update random things such as the client. Do that here
  let _ = UpdateTickHook.init_with_self(|this| {
    if let Ok(mut last) = LAST.try_lock() {
      let now = Instant::now();
      if last.map_or(true, |t| now.duration_since(t) >= INTERVAL) {
        *last = Some(now);
        drop(last); //probably won't matter but release the mutex

        //println!("Tick");
        update();
      }
    }

    UpdateTickHook.call(this);
  });
  let _ = unsafe { UpdateTickHook.enable() };
}

fn update() {
  GameState::with(|client, _| {
    client.client().update();
  });

  // TODO: Bind this somewhere more sane
  check_hints();
}
