use std::collections::HashSet;

use archipelago_rs::AsLocationId;

use crate::state::GameState;

pub fn check_missions() {
  // check missions to see if we need to trigger a check
  GameState::with(|client, memory, _| {
    let all_missions = memory.get_all_missions();
    //let missions: HashSet<_> = all_missions
    //  .iter()
    //  .filter(|m| m.is_completed)
    //  .map(|m| (m.as_location_id(), m))
    //  .collect();
    let locations: HashSet<_> = client
      .client()
      .checked_locations()
      .map(|l| l.as_location_id())
      .collect();

    // check missions that are completed but not marked as checked
    let _ = client.client().mark_checked(
      all_missions
        .iter()
        .filter(|m| m.is_completed)
        .map(|m| m.as_location_id())
        .filter(|m| !locations.contains(m)),
    );

    // internally mark checked missions as complete (to resync save, handle
    // logging into another slot, handle manually force-checked locations, etc)
    memory.set_missions_completed(
      all_missions
        .into_iter()
        .filter(|m| !m.is_completed)
        .filter(|m| locations.contains(&m.as_location_id())),
    );
  });
}
