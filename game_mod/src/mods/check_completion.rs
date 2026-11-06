use archipelago_rs::ClientStatus;

use crate::{
  archipelago::{
    location::APLocation,
    slot::{CompletionCondition, Goal},
  },
  state::GameState,
};

pub fn check_completion() {
  GameState::with(|client, memory, _| {
    let options = client.client().slot_data().clone();
    let is_completed = match options.completion_condition {
      CompletionCondition::Blimp => memory.get_blimp_kills() > 0,
      CompletionCondition::Score => {
        let score = memory.get_current_run_score();
        score >= options.completion_score
      }
    };

    if is_completed && let Some(plane) = memory.get_equipped_loadout() {
      let completed_planes: Vec<_> = client
        .client()
        .checked_locations()
        .filter_map(|l| APLocation::try_from(l).ok())
        .filter_map(|l| match l {
          APLocation::Completion(c) => Some(c),
          _ => None,
        })
        .collect();

      if !completed_planes.contains(&plane) {
        // if we haven't got this check, trigger it
        if options.send_generic_completions {
          let _ = client.client().mark_checked([
            APLocation::from(plane),
            APLocation::GenericCompletion(1 + completed_planes.len() as u8),
          ]);
        } else {
          let _ = client.client().mark_checked([APLocation::from(plane)]);
        }

        // check if we're done
        if let Goal::Completions(n) = options.goal
          && completed_planes.len() + 1 >= n as usize
        {
          let _ = client.client().set_status(ClientStatus::Goal);
        }
      }
    }
  });
}
