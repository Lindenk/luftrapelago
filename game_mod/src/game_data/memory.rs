use std::{
  collections::{HashMap, HashSet},
  ops::Add,
};

use archipelago_rs::AsLocationId;
use num_enum::TryFromPrimitive;

use crate::{
  archipelago::items::{APItem, BASE_ID},
  game_data::plane::{BodyType, EngineType, Part, WeaponType},
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
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,

  UnlockBuffer1 = 0x2770,
  UnlockBuffer2 = 0x2774,
  UnlockBuffer3 = 0x2778,
}

#[derive(Debug)]
pub struct MissionDescriptor {
  num: u8,
  part: Part,
  is_completed: bool,
}

impl MissionDescriptor {
  pub fn to_location_id(&self) -> impl AsLocationId {
    let mut id = BASE_ID;
    id += self.num as i64;
    id += match self.part {
      Part::Weapon(w) => (w as i64) * 5,
      Part::Body(b) => (b as i64) * 5 + 0x100,
      Part::Engine(e) => (e as i64) * 5 + 0x200,
    };

    id
  }
  pub fn from_mission_code(code: u32) -> Option<Self> {
    let (part, num) = parse_mission_code(code)?;
    Some(Self {
      part,
      num,
      is_completed: false,
    })
  }
}

pub fn parse_mission_code(code: u32) -> Option<(Part, u8)> {
  let part_type = code & 0xff;
  let part_num = (code >> 8) & 0xff;
  let mission_num = (code >> 16) & 0xff;

  let part = match part_type {
    0 => WeaponType::try_from_primitive(part_num as u8)
      .ok()
      .map(|w| Part::Weapon(w)),
    1 => BodyType::try_from_primitive(part_num as u8)
      .ok()
      .map(|b| Part::Body(b)),
    2 => EngineType::try_from_primitive(part_num as u8)
      .ok()
      .map(|e| Part::Engine(e)),
    _ => None,
  };

  part.map(|p| (p, mission_num as u8))
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

  pub fn set_unlocked_parts<P: Iterator<Item = Part>>(&self, parts: P) {
    // calc the full set of unlocked parts
    let mut parts_bin = 0i32;

    for p in parts {
      use Part::*;
      match p {
        Weapon(w) => parts_bin |= 1 << (w as u8),
        Body(b) => parts_bin |= (1 << (b as u8)) << 8,
        Engine(e) => parts_bin |= (1 << (e as u8)) << 16,
        _ => (),
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
  pub fn get_player_level(&self) -> u16 {
    self.read_player_stats(PlayerOffsets::PlayerLevel)
  }
  //pub fn get_total_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::TotalScore)
  //}
  //pub fn get_high_score(&self) -> i32 {
  //  self.read_i32(GlobalStatsOffsets::BestScore)
  //}

  pub fn get_all_missions(&self) -> Vec<MissionDescriptor> {
    // there are exactly num_part_type * num_parts * 5 mission, or
    // 3 * 6 * 5 = 90 total
    // interate over them all in memory
    const STRIDE: usize = 40; // bytes
    let start = self.player_ptr.add(PlayerOffsets::MissionStart as usize);
    let mut missions = Vec::new();

    for i in 0..90 {
      let offset = i * STRIDE;
      let mission_start = start.add(offset);

      let mission_code = unsafe { *(mission_start.add(0x0) as *const u32) };
      let is_completed = unsafe { *(mission_start.add(0x14) as *const u32) } > 0;

      //println!("Parsing mission: {:?}", mission_code);

      let (part, mission_num) = {
        if let Some(code) = parse_mission_code(mission_code) {
          code
        } else {
          println!("Failed to parse mission at: {:x}", mission_start);
          continue;
        }
      };

      missions.push(MissionDescriptor {
        part,
        num: mission_num,
        is_completed,
      });
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
}
