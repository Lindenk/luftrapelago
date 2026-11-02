use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
  Urauser,
  Completions,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionCondition {
  Blimp,
  Score,
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
  pub completion_condition: CompletionCondition,
  pub completion_locations: Vec<i64>,
  pub completion_score: u32,
  pub send_generic_completions: bool,
  pub starting_health: u16,
  pub powerup_bonuses: PowerupBonuses,
  pub rank_key_mission_lock: Option<u8>,
  pub dealthlink_counter: Option<u8>,
}
