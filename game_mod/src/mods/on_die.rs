use std::ffi::c_void;

use archipelago_rs::DeathLinkOptions;

use crate::{define_detour_hooks, state::GameState};

define_detour_hooks! {
  static OnDie: extern "thiscall" fn(*mut c_void) -> () => 0xd0c40;
}

pub fn hook_on_die() {
  let _ = OnDieHook.init_with_self(|this| {
    // handle dealthlink stuff here
    GameState::with(|client, _, temp_state| {
      println!("Handling deathlink");
      if !temp_state.deathlink_triggered
        && let Some(deathlink_count) = client.client().slot_data().dealthlink_counter
      {
        temp_state.deathlink_counter += 1;
        if temp_state.deathlink_counter >= deathlink_count {
          temp_state.deathlink_counter = 0;
          let _ = client.client().death_link(DeathLinkOptions::default());
        }
      } else {
        // clear it here so we can make sure we don't send deathlinks on a receive
        temp_state.deathlink_triggered = false;
      }
    });

    OnDieHook.call(this);
  });
  let _ = unsafe { OnDieHook.enable() };
}
