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
  pub health: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SlotData {
  pub goal: Goal,
  pub starting_health: u16,
  pub powerup_bonuses: PowerupBonuses,
  pub rank_key_mission_lock: Option<u8>,
  pub dealthlink_counter: Option<u8>,
}
