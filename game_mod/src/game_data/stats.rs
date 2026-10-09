#[repr(usize)]
pub enum GlobalStats {
  GamesPlayed = 0x20,
  TotalScore = 0x24,
  PlayerLevel = 0x32, // 16-bit int
  LastTime = 0x4c,
  TotalTime = 0x50,

  CurrentScore = 0x14,
  LastScore = 0x44,
  BestCombo = 0x48,

  LastFightersKilled = 0x58,
  // ...
  TotalFightersKilled = 0x8c,
  // ...
  MissionStart = 0xc8,
}
