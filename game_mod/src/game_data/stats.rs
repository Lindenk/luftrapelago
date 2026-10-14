use std::ops::Add;

use crate::fn_hooks::get_base_address;

const GLOBAL_STATS_PTR: usize = 0x16b3510;

fn get_global_stats_base() -> usize {
  GLOBAL_STATS_PTR + get_base_address()
}

#[derive(Copy, Clone)]
pub struct GlobalStats;

#[repr(usize)]
#[derive(Copy, Clone)]
enum GlobalStatsOffsets {
  GamesPlayed = 0x20,
  TotalScore = 0x24,
  PlayerLevel = 0x32, // 16-bit int
  LastTime = 0x4c,
  TotalTime = 0x50,

  CurrentScore = 0x14,
  BestScore = 0x44,
  BestCombo = 0x48,

  LastFightersKilled = 0x58,
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,
}

impl GlobalStats {
  fn read_i32(offset: GlobalStatsOffsets) -> i32 {
    unsafe {
      let offset = offset as usize;
      let target = get_global_stats_base().add(offset) as *const i32;
      std::ptr::read_volatile(target)
    }
  }
  fn read_i16(offset: GlobalStatsOffsets) -> i16 {
    unsafe {
      let offset = offset as usize;
      let target = get_global_stats_base().add(offset) as *const i16;
      std::ptr::read_volatile(target)
    }
  }
  pub fn is_mission_completed(mission_num: u8, part_num: u8, part_type: u8) -> bool {
    let mut offset = GlobalStatsOffsets::MissionStart as usize;
    offset += 0x14; // mission completion flag
    offset += 40 * mission_num as usize; // mission tiers are 40 bytes apart
    offset += 40 * (part_num * 5) as usize; // there's 5 missions before the next part
    offset += 40 * (part_type * 5 * 6) as usize; // there's 6 parts before the next type

    unsafe {
      let target = get_global_stats_base().add(offset) as *const i32;
      std::ptr::read_volatile(target) > 0
    }
  }
  pub fn get_player_level() -> u16 {
    Self::read_i16(GlobalStatsOffsets::PlayerLevel) as u16
  }
  pub fn get_total_score() -> i32 {
    Self::read_i32(GlobalStatsOffsets::TotalScore)
  }
  pub fn get_high_score() -> i32 {
    Self::read_i32(GlobalStatsOffsets::BestScore)
  }
}
