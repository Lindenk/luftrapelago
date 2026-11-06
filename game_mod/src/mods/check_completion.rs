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

      // check if we're done
      if let Goal::Completions(n) = options.goal
        && completed_planes.len() >= n as usize
      {
        let _ = client.client().set_status(ClientStatus::Goal);
      }

      // check if we finished a generic completion (seperately to avoid weird
      // races)
      if options.send_generic_completions {
        if let Some(highest_generic_completion) = client
          .client()
          .checked_locations()
          .filter_map(|l| APLocation::try_from(l).ok())
          .filter_map(|l| match l {
            APLocation::GenericCompletion(c) => Some(c as usize),
            _ => None,
          })
          .max()
        {
          if completed_planes.len() > highest_generic_completion {
            // mark all generic completions up to it
            let _ = client.client().mark_checked(
              (highest_generic_completion..=completed_planes.len())
                .map(|i| APLocation::GenericCompletion(i as u8)),
            );
          }
        }
      }

      // if we haven't got this check, trigger it
      if !completed_planes.contains(&plane) {
        let _ = client.client().mark_checked([APLocation::from(plane)]);
      }
    }
  });
}
