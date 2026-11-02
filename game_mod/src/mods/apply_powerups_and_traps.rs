use std::{ffi::c_void, ops::Div, ptr::addr_of_mut};

use windows::Win32::System::Memory::{
  PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};

use crate::{
  archipelago::items::APItem, define_detour_hooks, fn_hook::get_base_address,
  patcher::patch_memory, state::GameState,
};

define_detour_hooks! {
  static LoadWeaponStats: extern "cdecl" fn(*mut c_void, *mut c_void) -> *mut c_void => 0xd7f60;
  //static LoadBodyStats: extern "cdecl" fn(*mut c_void) -> *mut c_void => 0xd7000;
  //static LoadEngineStats: extern "cdecl" fn(*mut c_void) -> *mut c_void => 0xd5740;
}

pub fn apply_powerups_and_traps() {
  let _ = LoadWeaponStatsHook.init_with_self(|stats, param2| {
    let res = LoadWeaponStatsHook.call(stats, param2);

    GameState::with(|client, _, temp_state| {
      let firerate_items = client
        .get_items()
        .filter(|i| matches!(i, APItem::PowerupFireRate))
        .count() as i32;
      let bullet_items = client
        .get_items()
        .filter(|i| matches!(i, APItem::PowerupBonusBullets))
        .count() as i32;

      // weapon interval is stored at stats+0x1d*4, as an int
      let stats = stats as *mut i32;
      let id = unsafe { stats.read_volatile() };
      unsafe {
        let interval_ptr = stats.add(0x1d);
        let mut interval = interval_ptr.read_volatile();

        // for now give the cannon special treatment
        if id == 4 {
          interval -= 12 * firerate_items;
        } else {
          interval = (interval - firerate_items).max(1);
        }
        interval_ptr.write_volatile(interval);
      }
      unsafe {
        // add bonus bullets (and handle gun jams)
        let numbullet_ptr = stats.add(0x21);
        if temp_state.gunjamed_timeout.is_some() {
          numbullet_ptr.write_volatile(0);
        } else {
          let mut numbullets = numbullet_ptr.read_volatile();
          numbullets += bullet_items * 5;
          numbullet_ptr.write_volatile(numbullets);
        }
      }
      unsafe {
        // increase max bullets too  (for missles)
        let maxbullets_ptr = stats.add(0x1e);
        let mut maxbullets = maxbullets_ptr.read_volatile();
        maxbullets += bullet_items;
        maxbullets_ptr.write_volatile(maxbullets);
      }
    });

    res
  });
  let _ = unsafe { LoadWeaponStatsHook.enable() };

  // Detour hooks this function wrong so just hook the end manually. There's
  // 14 extra alignment bytes after the ret so we should have room
  unsafe {
    // addr of the ret
    let addr = (get_base_address() + 0xd7e51) as *mut u8;
    let mut patch = vec![
      0x9C, 0x60, // push fd, ad
      0x89, 0xC1, // mov ecx, eax (the returned pointer) for our arg
      0xb8, // mov eax, set up the fn call
    ];
    let func_addr = modify_body_stats as extern "fastcall" fn(*mut f32) as u32;
    patch.extend_from_slice(&func_addr.to_le_bytes());
    patch.extend_from_slice(&[
      0xFF, 0xD0, // call eax
      0x61, 0x9D, // pop ad, fd
      0xC3, // ret
    ]);

    // sanity check and patch
    assert!(patch.len() <= 15);
    patch_memory(addr, patch.as_slice());
  }

  // the engine stats also has this problem...damn. Only 8 bytes of alignment at
  // the end... Let's be a little creative with this I suppose
  // allocate some const bytes and mark them as executable
  static mut TRAMPOLINE: [u8; 14] = [
    0x9C, 0x60, // push fd, ad
    0x89, 0xC1, // mov ecx, eax (the returned pointer) for our arg
    0xb8, // mov eax, set up the fn call
    0x00, 0x00, 0x00, 0x00, // fn pointer that will be written
    0xFF, 0xD0, // call eax
    0x61, 0x9D, // pop ad, fd
    0xC3, // ret
  ];

  unsafe {
    let mut old = PAGE_PROTECTION_FLAGS(0);
    let ptr = addr_of_mut!(TRAMPOLINE) as *mut c_void;
    let _ = VirtualProtect(
      ptr,
      std::mem::size_of::<[u8; 14]>(),
      PAGE_EXECUTE_READWRITE,
      &mut old,
    );

    // write our fn into the trampoline
    let fn_addr = modify_engine_stats as *const fn(*mut f32) as u32;
    (ptr as *mut u32).byte_add(5).write_unaligned(fn_addr);

    // write a jump to the trampoline over the ret (+ alignment bytes)
    let hooked_addr = (get_base_address() + 0xd6d67) as *mut u8;
    let trampoline_rel_addr = (ptr as i32) - (hooked_addr as i32 + 5);
    let mut patch = vec![0xe9];
    patch.extend_from_slice(&(trampoline_rel_addr as u32).to_le_bytes());
    patch_memory(hooked_addr as *mut u8, patch.as_slice());
  }
}

extern "fastcall" fn modify_engine_stats(stats: *mut f32) {
  GameState::with(|_, _, temp_state| {
    if temp_state.engine_stall_timeout.is_some() {
      unsafe {
        // set accelaration to 0
        let max_speed_ptr = stats.add(0x32);
        max_speed_ptr.write_volatile(0f32);
        // space accel
        //let max_speed_ptr = stats.add(0x40);
        //max_speed_ptr.write_volatile(0f32);
        // water accel
        //let max_speed_ptr = stats.add(0x3b);
        //max_speed_ptr.write_volatile(0f32);
      }
    }
  });
}

extern "fastcall" fn modify_body_stats(stats: *mut f32) {
  GameState::with(|client, _, _| {
    let maxhealth_items = client
      .get_items()
      .filter(|i| matches!(i, APItem::PowerupHealth))
      .count() as f32;

    let slot_data = client.client().slot_data();
    let health_bonus = (slot_data.powerup_bonuses.health as f32).div(100.);
    let starting_health = (slot_data.starting_health as f32).div(100.);

    let scale = starting_health + (maxhealth_items * health_bonus);

    // max health is stored at stats+0x18*4, as a float
    unsafe {
      let max_hp_ptr = stats.add(0x18);
      let mut maxhealth = max_hp_ptr.read_volatile();
      maxhealth *= scale;
      max_hp_ptr.write_volatile(maxhealth);
    }
    // scale regen rate a bit so the max health feels a little better
    unsafe {
      let regen_ptr = stats.add(0x17);
      let mut regen_rate = regen_ptr.read_volatile();
      regen_rate *= scale.midpoint(1.); // make it half as effective (1 + scale) /2
      regen_ptr.write_volatile(regen_rate);
    }
  });
}
