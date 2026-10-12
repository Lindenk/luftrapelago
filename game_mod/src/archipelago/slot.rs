use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "name")]
pub enum Goal {
  Urauser,
  UniqueBlimpKills { planes: u16 },
  UniqueClassified1Kills { planes: u16 },
  UniqueClassified2Kills { planes: u16 },
  ScoreAttack { score: u32 },
  ScoreTotal { score: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PowerupBonuses {
  damage: u16,
  health: u16,
  fire_rate: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SlotData {
  goal: Goal,
  starting_power: u16,
  powerup_bonuses: PowerupBonuses,
  rank_key_mission_lock: Option<u8>,
  dealthlink_counter: Option<u8>,
}
