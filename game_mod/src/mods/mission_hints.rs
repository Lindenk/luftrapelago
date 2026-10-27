use std::{
  collections::{HashMap, HashSet},
  sync::{LazyLock, Mutex},
  thread,
  time::Duration,
};

use archipelago_rs::{AsLocationId, LocatedItem};

use crate::{
  define_detour_hooks,
  fn_hooks::get_base_address,
  game_data::{game_string::GameString, stats::MissionDescriptor},
  patcher::CodePatch,
  state::{GAME_STATE, GameState},
};

static HINTS: Mutex<LazyLock<HashMap<i64, LocatedItem>>> =
  Mutex::new(LazyLock::new(|| HashMap::new()));

define_detour_hooks! {
  static GenerateMissionString: extern "stdcall" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut u8 => 0xaf950;
}

const MISSION_BLOCK_SKULL_DRAW_CHECK: usize = 0xb2047;
const MISSION_BLOCK_SKULL_DRAW_END: usize = 0xb20e1;

pub fn draw_mission_hints() {
  // nop out the bit that draws the skulls on missions
  let addr = (get_base_address() + MISSION_BLOCK_SKULL_DRAW_CHECK) as *mut u8;
  let end_addr = (get_base_address() + MISSION_BLOCK_SKULL_DRAW_END) as *mut u8;
  unsafe { CodePatch::at(addr).continuation(end_addr).write() };

  // spawn a thread to fetch all active hints
  thread::spawn(|| {
    loop {
      check_hints();
      thread::sleep(Duration::from_millis(200));
    }
  });

  // append our hints to the mission text (when it gets generated)
  let _ = GenerateMissionStringHook.init_with_self(|this, that| {
    let s = GenerateMissionStringHook.call(this, that);
    let new_s: String;

    // figure out what mission location this is
    let mission_code = unsafe { *(this as *const u32) };
    let mission = MissionDescriptor::from_mission_code(mission_code);
    //let mission = calc_mission_location(mission);

    // get our hints
    if let Some(mission) = mission {
      if let Ok(hints) = HINTS.lock()
        && let Some(scounted_loc) = hints.get(&mission.to_location_id().as_location_id())
      {
        new_s = format!(
          "{}'s {}",
          scounted_loc.receiver().name(),
          scounted_loc.item()
        );
      } else {
        new_s = "Fetching hint...".to_string();
      }
    } else {
      new_s = format!("Invalid mission! {:x}", mission_code);
    }

    let gs = GameString::copy_from_game(s);
    let gs = gs.append(format!("\n  {}", new_s));
    gs.assign(s);

    s
  });
  let _ = unsafe { GenerateMissionStringHook.enable() };
}

fn check_hints() {
  // do this the dumb way: check all active missions (ie. missions that the
  // player can do next), then check if we have hints for them all

  // get our active missions to check against
  let active_missions = {
    let mut state = GAME_STATE.lock().unwrap();
    if let GameState::Connected(state) = &mut *state {
      // tick here for the moment
      state.client.update();
      state.stats.get_all_active_missions()
    } else {
      return; // nothing to update if we aren't running yet
    }
  };
  //println!("Fetching hints...");

  let active_missions: HashSet<i64> = active_missions
    .into_values()
    .map(|m| m.to_location_id().as_location_id())
    .collect();

  // check for existing corrisponding hints
  let new_hints: HashSet<i64> = {
    let hints = HINTS.lock().unwrap();
    active_missions
      .iter()
      .filter(|m| !hints.contains_key(m))
      .cloned()
      .collect()
  };
  //println!("We need hints for: {:?}", active_missions);
  //println!("We need new hints for: {:?}", new_hints);
  if new_hints.len() == 0 {
    return;
  }

  // try to fetch any outstanding hints
  let outstanding_hints = {
    let mut state = GAME_STATE.lock().unwrap();
    if let GameState::Connected(state) = &mut *state {
      let locs = state.client.client().scout_locations(
        active_missions.iter().cloned(),
        archipelago_rs::CreateAsHint::New,
      );
      thread::sleep(Duration::from_millis(200));
      state.client.client().update();
      locs
    } else {
      return; // nothing to update if we aren't running yet
    }
  };

  match outstanding_hints.recv_timeout(Duration::from_millis(3000)) {
    Ok(Ok(new_hints)) => {
      let mut hints = HINTS.lock().unwrap();
      //println!("Got new hints! {:?}", new_hints);
      for h in new_hints {
        hints.insert(h.location().as_location_id(), h);
      }
    }
    _ => {
      println!("Timed out getting hints");
      return;
    }
  }
}
