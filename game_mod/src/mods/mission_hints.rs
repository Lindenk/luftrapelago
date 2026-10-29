use std::ops::Not;

use archipelago_rs::AsLocationId;

use crate::{
  define_detour_hooks,
  fn_hook::get_base_address,
  game_data::{game_string::GameString, memory::MissionDescriptor},
  patcher::CodePatch,
  state::GameState,
};

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
  //thread::spawn(|| {
  //  loop {
  //    check_hints();
  //    thread::sleep(Duration::from_millis(200));
  //  }
  //});

  // append our hints to the mission text (when it gets generated)
  let _ = GenerateMissionStringHook.init_with_self(|this, that| {
    let s = GenerateMissionStringHook.call(this, that);

    // figure out what mission location this is
    let mission_code = unsafe { *(this as *const u32) };
    let mission = MissionDescriptor::from_mission_code(mission_code);

    // get our hints
    let hint = if let Some(mission) = mission {
      GameState::with(|client, _| {
        let player = client.client().this_player().clone();
        client
          .client()
          .scouted_locations()
          .find(|l| l.location().same_location(mission.to_location_id()))
          .map(|l| {
            if *l.receiver() == player {
              l.item().name().to_string()
            } else {
              format!("{}'s {}", l.receiver().name(), l.item().name())
            }
          })
      })
      .flatten()
      .unwrap_or("Fetching hint...".to_string())
    } else {
      format!("Invalid mission! {:x}", mission_code)
    };

    let gs = GameString::copy_from_game(s);
    let gs = gs.append(format!("\n  {}", hint));
    gs.assign(s);

    s
  });
  let _ = unsafe { GenerateMissionStringHook.enable() };
}

// TODO: Hook this in somewhere sane, like at the end of a round
pub fn check_hints() {
  GameState::with(|client, memory| {
    let unlocked_parts: Vec<_> = client.get_parts().collect();
    let active_missions = memory.get_all_active_missions(&unlocked_parts);
    //println!("Active missions: {:?}", active_missions);
    let scouted_locations: Vec<_> = client
      .client()
      .scouted_locations()
      .map(|l| l.location().as_location_id())
      .collect();
    //println!("Scouted locations: {:?}", scouted_locations);
    let unhinted_missions: Vec<_> = active_missions
      .iter()
      .filter_map(|(_, m)| {
        let id = m.to_location_id().as_location_id();
        scouted_locations.contains(&id).not().then_some(id)
      })
      .collect();
    //println!("Unhinted missions: {:?}", unhinted_missions);
    if unhinted_missions.len() > 0 {
      client
        .client()
        .scout_locations(unhinted_missions, archipelago_rs::CreateAsHint::New);
    }
  });
}
