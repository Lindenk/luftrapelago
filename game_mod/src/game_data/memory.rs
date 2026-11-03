use std::{
  collections::{HashMap, HashSet},
  ops::Add,
};

use crate::game_data::{
  mission::{MissionCode, MissionDescriptor},
  plane::{BodyType, EngineType, Part, Plane, WeaponType},
};

const GLOBAL_STATS_PTR: usize = 0x01ab3510;
const GLOBAL_PLAYER_PTR: usize = 0x01ab34c8;

#[derive(Copy, Clone)]
pub struct GameMemory {
  stats_ptr: usize,
  player_ptr: usize,
}

#[repr(usize)]
#[derive(Copy, Clone)]
enum GlobalStatsOffsets {
  EnableHangar = 0x10, // 8-bit, 1 = enable
  TutorialFlag = 0x14, // this is set elsewhere because it has to init other stuff too
  SFMTEnable = 0x18,
  GamesPlayed = 0x20,
  //PlayerLevel = 0x32,    // 16-bit int
  LastTime = 0x4c,
  TotalTime = 0x50,

  BestScore = 0x44,
  //BestCombo = 0x48,
  LastFightersKilled = 0x58,
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,
}

#[repr(usize)]
#[derive(Copy, Clone)]
enum PlayerOffsets {
  CurrentScore = 0x14,
  QueuedNotifications = 0x18, // ?
  GamesPlayed = 0x20,
  TotalScore = 0x24,
  EquippedBody = 0x2c,   // 8-bit int, counter starting at 0
  EquippedEngine = 0x2d, // 8-bit int, counter starting at 0
  EquippedWeapon = 0x2e, // 8-bit int, counter starting at 0
  PlayerLevel = 0x32,    // 16-bit int
  UnlockedParts = 0x34,

  LastTime = 0x4c,
  TotalTime = 0x50,

  BestScore = 0x44,
  //BestCombo = 0x48,
  LastFightersKilled = 0x58,
  LastBlimpsKilled = 0x78,
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,
}

impl GameMemory {
  pub fn init() -> Self {
    println!(
      "Pointer to global stats and player ram is currently: {:x}, {:x}",
      GLOBAL_STATS_PTR, GLOBAL_PLAYER_PTR
    );
    let stats_ptr = {
      let ptr = (GLOBAL_STATS_PTR) as *const usize;
      unsafe { std::ptr::read_volatile(ptr) }
    };
    println!("Set base global stats ptr: {:x}", stats_ptr);

    let player_ptr = {
      let ptr = (GLOBAL_PLAYER_PTR) as *const usize;
      unsafe { std::ptr::read_volatile(ptr) }
    };
    println!("Set base global player ptr: {:x}", player_ptr);

    Self {
      stats_ptr,
      player_ptr,
    }
  }

  // helper functions
  fn read_global_stats<T>(&self, offset: GlobalStatsOffsets) -> T {
    unsafe {
      let offset = offset as usize;
      let target = self.stats_ptr.add(offset) as *const T;
      std::ptr::read_volatile(target)
    }
  }
  fn write_global_stats<T>(&self, offset: GlobalStatsOffsets, value: T) {
    unsafe {
      let offset = offset as usize;
      let target = self.stats_ptr.add(offset) as *mut T;
      std::ptr::write_volatile(target, value)
    }
  }
  fn modify_global_stats<T, F: Fn(T) -> T>(&self, offset: GlobalStatsOffsets, f: F) {
    // TODO: Put this in a critical section

    let v = self.read_global_stats(offset);
    self.write_global_stats(offset, f(v));
  }
  fn read_player_stats<T>(&self, offset: PlayerOffsets) -> T {
    unsafe {
      let offset = offset as usize;
      let target = self.player_ptr.add(offset) as *const T;
      std::ptr::read_volatile(target)
    }
  }
  fn write_player_stats<T>(&self, offset: PlayerOffsets, value: T) {
    unsafe {
      let offset = offset as usize;
      let target = self.player_ptr.add(offset) as *mut T;
      std::ptr::write_volatile(target, value)
    }
  }
  fn modify_player_stats<T, F: Fn(T) -> T>(&self, offset: PlayerOffsets, f: F) {
    // TODO: Put this in a critical section

    let v = self.read_player_stats(offset);
    self.write_player_stats(offset, f(v));
  }

  // state manip functions

  pub fn enable_hangar(&self) {
    self.write_global_stats(GlobalStatsOffsets::EnableHangar, 1u8);
  }

  pub fn get_current_run_score(&self) -> u32 {
    self.read_player_stats(PlayerOffsets::CurrentScore)
  }

  pub fn get_blimp_kills(&self) -> u32 {
    //for x in (0..0x58).step_by(4) {
    //  unsafe {
    //    let ptr = self.player_ptr.add(x) as *mut u32;
    //    println!("Offset 0x{:x}: {}", x, ptr.read_volatile());
    //  }
    //}

    self.read_player_stats(PlayerOffsets::LastBlimpsKilled)
  }

  pub fn set_unlocked_parts<P: Iterator<Item = Part>>(&self, parts: P) {
    // calc the full set of unlocked parts
    let mut parts_bin = 0i32;

    for p in parts {
      use Part::*;
      match p {
        Weapon(w) => parts_bin |= 1 << (w as u8),
        Body(b) => parts_bin |= (1 << (b as u8)) << 8,
        Engine(e) => parts_bin |= (1 << (e as u8)) << 16,
      }
    }

    self.write_player_stats(PlayerOffsets::UnlockedParts, parts_bin);
  }

  pub fn is_weapon_unlocked(&self, weapon: WeaponType) -> bool {
    let unlocks: u32 = self.read_player_stats(PlayerOffsets::UnlockedParts);
    let unlocks = (unlocks & 0x3f) as u8; // mask out weapon unlocks

    let mask = 1 << (weapon as u8);
    (unlocks & mask) > 0
  }
  pub fn is_body_unlocked(&self, body: BodyType) -> bool {
    let unlocks: u32 = self.read_player_stats(PlayerOffsets::UnlockedParts);
    let unlocks = ((unlocks >> 8) & 0x3f) as u8; // mask out body unlocks

    let mask = 1 << (body as u8);
    (unlocks & mask) > 0
  }
  pub fn is_engine_unlocked(&self, engine: EngineType) -> bool {
    let unlocks: u32 = self.read_player_stats(PlayerOffsets::UnlockedParts);
    let unlocks = ((unlocks >> 16) & 0x3f) as u8; // mask out egnine unlocks

    let mask = 1 << (engine as u8);
    (unlocks & mask) > 0
  }

  pub fn set_parts(&self, weapon: WeaponType, body: BodyType, engine: EngineType) {
    self.write_player_stats(PlayerOffsets::EquippedWeapon, weapon as u8);
    self.write_player_stats(PlayerOffsets::EquippedBody, body as u8);
    self.write_player_stats(PlayerOffsets::EquippedEngine, engine as u8);
  }

  pub fn get_equipped_weapon(&self) -> Option<WeaponType> {
    let w: u8 = self.read_player_stats(PlayerOffsets::EquippedWeapon);
    WeaponType::try_from(w).ok()
  }
  pub fn get_equipped_body(&self) -> Option<BodyType> {
    let b: u8 = self.read_player_stats(PlayerOffsets::EquippedBody);
    BodyType::try_from(b).ok()
  }
  pub fn get_equipped_engine(&self) -> Option<EngineType> {
    let e: u8 = self.read_player_stats(PlayerOffsets::EquippedEngine);
    EngineType::try_from(e).ok()
  }

  pub fn get_equipped_loadout(&self) -> Option<Plane> {
    match (
      self.get_equipped_weapon(),
      self.get_equipped_body(),
      self.get_equipped_engine(),
    ) {
      (Some(weapon), Some(body), Some(engine)) => Some(Plane {
        weapon,
        body,
        engine,
      }),
      _ => None,
    }
  }

  /*
  pub fn unlock_weapon(&self, weapon: WeaponType) {
    let mask = 1 << (weapon as u8);
    self.modify_player_stats(PlayerOffsets::UnlockedParts, |v: u32| v | mask);
  }
  pub fn unlock_body(&self, body: BodyType) {
    let mut mask = 1 << (body as u8);
    mask <<= 8;
    self.modify_player_stats(PlayerOffsets::UnlockedParts, |v: u32| v | mask);
  }
  pub fn unlock_engine(&self, engine: EngineType) {
    let mut mask = 1 << (engine as u8);
    mask <<= 16;
    self.modify_player_stats(PlayerOffsets::UnlockedParts, |v: u32| v | mask);
  }*/

  //pub fn is_mission_completed(&self, mission_num: u8, part_num: u8, part_type: u8) -> bool {
  //  let mut offset = GlobalStatsOffsets::MissionStart as usize;
  //  offset += 0x14; // mission completion flag
  //  offset += 40 * mission_num as usize; // mission tiers are 40 bytes apart
  //  offset += 40 * (part_num * 5) as usize; // there's 5 missions before the next part
  //  offset += 40 * (part_type * 5 * 6) as usize; // there's 6 parts before the next type
  //
  //  unsafe {
  //    let target = self.base_ptr.add(offset) as *const i32;
  //    std::ptr::read_volatile(target) > 0
  //  }
  //}
  pub fn get_rauser_level(&self) -> u16 {
    self.read_player_stats(PlayerOffsets::PlayerLevel)
  }
  pub fn set_rauser_level(&self, level: u16) {
    self.write_player_stats(PlayerOffsets::PlayerLevel, level);
  }
  //pub fn get_total_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::TotalScore)
  //}
  //pub fn get_high_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::BestScore)
  //}

  fn read_missions(&self) -> impl Iterator<Item = *mut u32> {
    // there are exactly num_part_type * num_parts * 5 mission, or
    // 3 * 6 * 5 = 90 total
    // interate over them all in memory
    const STRIDE: usize = 40; // bytes
    let start = self.player_ptr.add(PlayerOffsets::MissionStart as usize);

    (0..90).into_iter().map(move |i| {
      let offset = i * STRIDE;
      start.add(offset) as *mut u32
    })
  }

  pub fn get_all_missions(&self) -> Vec<MissionDescriptor> {
    let mut missions = Vec::new();

    for m in self.read_missions() {
      let mission_code = MissionCode(unsafe { *(m.byte_add(0x0) as *const u32) });
      let is_completed = unsafe { *(m.byte_add(0x14) as *const u32) } > 0;

      if let Ok(mut desc) = MissionDescriptor::try_from(mission_code) {
        desc.is_completed = is_completed;
        missions.push(desc);
      } else {
        println!("Failed to parse mission at: {:x}", m as usize);
      }
    }

    missions
  }

  pub fn get_all_active_missions(
    &self,
    unlocked_parts: &[Part],
  ) -> HashMap<Part, MissionDescriptor> {
    let mut missions: HashMap<Part, MissionDescriptor> = HashMap::new();

    for m in self
      .get_all_missions()
      .into_iter()
      .filter(|m| !m.is_completed)
      .filter(|m| unlocked_parts.contains(&m.part))
    {
      // find the lowest mission num for each part
      missions
        .entry(m.part)
        .and_modify(|e| e.num = e.num.min(m.num))
        .or_insert(m);
    }

    missions
  }

  pub fn set_missions_completed(&self, missions: impl Iterator<Item = MissionDescriptor>) {
    let missions: HashSet<_> = missions.map(|m| MissionCode::from(m).0).collect();

    unsafe {
      self
        .read_missions()
        .filter(|p| missions.contains(&p.read_volatile()))
        .for_each(|p| p.byte_offset(0x14).write_volatile(1));
    }
  }
}
