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
  {
    let addr = (get_base_address() + MISSION_BLOCK_SKULL_DRAW_CHECK) as *mut u8;
    let end_addr = (get_base_address() + MISSION_BLOCK_SKULL_DRAW_END) as *mut u8;
    unsafe { CodePatch::at(addr).continuation(end_addr).write() };
  }

  // Disable mission text and hints if the part isn't unlocked yet
  /*unsafe {
    // the check for if we have a blimp kill. Replace the condition with an
    // unlock check and display own our own text
    let addr = (get_base_address() + 0xaf9a2) as *mut u8;
    let false_branch_addr = (get_base_address() + 0xaf9d6) as *mut u8;
    let true_branch_addr = (get_base_address() + 0xb00ac) as *mut u8;

    let mut patch = vec![
      0x9C, 0x60, // push fd, ad
      0x89, 0xD9, // load the mission descriptor into ecx (from ebx)
      0x8B, 0x55, 0x0C, // load the string ptr into edx
      0xb8, // prep for our fn call
    ];
    let fn_addr =
      check_mission_locked_text as extern "fastcall" fn(*const u8, *mut u8) -> bool as u32;
    patch.extend(fn_addr.to_le_bytes());
    patch.extend_from_slice(&[
      0xff, 0xd0, // call eax
      0x84, 0xc0, // test al
    ]);

    let post_branch_cleanup = vec![
      0x61, 0x9D, // pop add, fd
    ]; // + 5 bytes for absolute jump after cleanup

    let false_branch_offset = (post_branch_cleanup.len() + 8) as i32;
    patch.extend_from_slice(&[0x0f, 0x84]); // jz
    patch.extend(false_branch_offset.to_le_bytes());

    // write in the true branch
    patch.extend_from_slice(&post_branch_cleanup);
    patch.extend_from_slice(&[0x8b, 0x7d, 0x0c]); // put the str back in edi
    let true_branch_offset = (true_branch_addr as usize - (addr as usize + patch.len() + 5)) as i32;
    patch.push(0xe9); // jmp
    patch.extend(true_branch_offset.to_le_bytes());

    // write in the false branch
    patch.extend(post_branch_cleanup);
    let false_branch_offset =
      (false_branch_addr as usize - (addr as usize + patch.len() + 5)) as i32;
    patch.push(0xe9); // jmp
    patch.extend(false_branch_offset.to_le_bytes());

    // sanity check
    assert!((false_branch_addr as usize > (addr as usize + patch.len())));

    // write it in
    patch_memory(addr, &patch);
  }*/

  // append our hints to the mission text (when it gets generated)
  let _ = GenerateMissionStringHook.init_with_self(|this, that| {
    let s = GenerateMissionStringHook.call(this, that);

    // figure out what mission location this is
    let mission_code = unsafe { *(this as *const u32) };
    let mission = MissionDescriptor::from_mission_code(mission_code);

    // get our hints
    enum Hint {
      Hint(String),
      Pending,
      Locked,
      Invalid,
    }
    let hint = if let Some(mission) = mission {
      GameState::with(|client, _| {
        if client.get_parts().find(|p| *p == mission.part).is_none() {
          return Hint::Locked;
        }

        let player = client.client().this_player().clone();
        client
          .client()
          .scouted_locations()
          .find(|l| l.location().same_location(mission.to_location_id()))
          .map(|l| {
            if *l.receiver() == player {
              Hint::Hint(l.item().name().to_string().to_lowercase())
            } else {
              Hint::Hint(format!(
                "{}'s {}",
                l.receiver().name().to_lowercase(),
                l.item().name().to_lowercase()
              ))
            }
          })
          .unwrap_or(Hint::Pending)
      })
      .unwrap_or(Hint::Pending)
    } else {
      Hint::Invalid
    };

    let hint = match hint {
      Hint::Hint(h) => h,
      Hint::Pending => "fetching hint...".to_string(),
      Hint::Invalid => {
        format!("invalid mission! {:x}", mission_code)
      }
      Hint::Locked => {
        GameString::assign_from_str(s, "part locked");
        return s;
      }
    };

    let gs = GameString::copy_from_game(s);
    let gs = gs.append(format!("\n  {}", hint));
    gs.assign(s);

    s
  });
  let _ = unsafe { GenerateMissionStringHook.enable() };
}

// see if we should block off this mission text
/*
extern "fastcall" fn check_mission_locked_text(this: *const u8, msg: *mut u8) -> bool {
  println!("Calling check mission locked text");

  let mission_code = unsafe { (this as *const u32).read_volatile() };
  let mission = if let Some(m) = MissionDescriptor::from_mission_code(mission_code) {
    m
  } else {
    return false;
  };

  println!("Str ptr: {:x}", msg as u32);

  GameState::with(|client, _| {
    if client.get_parts().find(|p| *p == mission.part).is_none() {
      //let new_str = GameString::from_str("part locked");

      // init the string ptr

      unsafe {
        (msg.add(0x10) as *mut u32).write_volatile(0);
        (msg.add(0x14) as *mut u32).write_volatile(0x0f);
        msg.write_volatile(0);
      }

      GameString::assign_from_str(msg, "part locked");
      true
    } else {
      false
    }
  })
  .unwrap_or(true)
}*/

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
