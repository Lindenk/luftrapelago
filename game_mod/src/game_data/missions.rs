pub enum PlanePartType {
  Weapon = 0,
  Body = 1,
  Engine = 2,
}

pub struct MissionRef {
  num: u8,
  part_type: PlanePartType,
  part_number: u8,
}

pub enum MissionObjective {
  Kill = 0,
  PlayGames = 1,
  Score = 2,
  KillTypes = 3,
  Combo = 4,
  Bomb = 5,
  Encounter = 6,
  KillStreak = 7,
}

pub enum MissionCondition {
  Total = 0,
  WithinCombo = 1,
  WhileFalling = 2,
  WhileUndamaged = 3,
  WithoutFiring = 4,
  WhileFiring = 5,
  WithoutMissing = 6,
  WhileOnFire = 7,
  AtMaxCombo = 8,
  WhileBoosting = 9,
  InSetTime = 10,
  AfterDeath = 11,
  InOneGame = 12,
  EyesClosed = 13,
}

pub enum EnemyType {
  Any = 0,
  Fighter = 1,
  Boat = 2,
  Battleship = 3,
  Jet = 4,
  Ace = 5,
  Classified1 = 6,
  Sub = 7,
  Classified2 = 8,
  Blimp = 9,
  Missles = 10,
}

pub struct Mission {
  component: MissionRef,
  objective: MissionObjective,
  condition: MissionCondition,
  num_required: u32,
  enemy_type: EnemyType,
  is_completed: bool,
  progress: u32,
}
