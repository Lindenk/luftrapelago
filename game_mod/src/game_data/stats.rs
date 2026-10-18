use std::ops::Add;

use crate::{
  fn_hooks::get_base_address,
  game_data::{
    plane::{BodyType, EngineType, WeaponType},
    stats::GlobalStatsOffsets::MissionStart,
  },
};

const GLOBAL_STATS_PTR: usize = 0x01ab3510;
const GLOBAL_PLAYER_PTR: usize = 0x01ab34c8;

#[derive(Copy, Clone)]
pub struct GlobalStats {
  stats_ptr: usize,
  player_ptr: usize,
}

#[repr(usize)]
#[derive(Copy, Clone)]
enum GlobalStatsOffsets {
  EnableHangar = 0x10, // 8-bit, 1 = enable
  TutorialFlag = 0x14, // this is set elsewhere because it has to init other stuff too
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
  CurrentScore = 0x18, // ?
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
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,
}

impl GlobalStats {
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

  pub fn set_initial_parts(
    &self,
    weapons: &[WeaponType],
    bodies: &[BodyType],
    engines: &[EngineType],
  ) {
    // calc the full set of unlocked parts
    let mut parts = 0i32;

    for w in weapons {
      parts |= 1 << (*w as u8);
    }
    for b in bodies {
      parts |= (1 << (*b as u8)) << 8;
    }
    for e in engines {
      parts |= (1 << (*e as u8)) << 16;
    }

    self.write_player_stats(PlayerOffsets::UnlockedParts, parts);

    // find first part of each and set that as the current loadout
    if let Some(w) = weapons.iter().next() {
      self.write_player_stats(PlayerOffsets::EquippedWeapon, (*w as u8));
    }
    if let Some(b) = bodies.iter().next() {
      self.write_player_stats(PlayerOffsets::EquippedBody, (*b as u8));
    }
    if let Some(e) = engines.iter().next() {
      self.write_player_stats(PlayerOffsets::EquippedEngine, (*e as u8));
    }
  }

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
  }

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
  pub fn get_player_level(&self) -> u16 {
    self.read_player_stats(PlayerOffsets::PlayerLevel)
  }
  //pub fn get_total_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::TotalScore)
  //}
  //pub fn get_high_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::BestScore)
  //}
}
