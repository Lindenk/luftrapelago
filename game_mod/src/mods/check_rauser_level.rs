use std::{cmp::max, collections::HashSet};

use crate::{
  archipelago::location::{APLocation, RauserLevel},
  state::GameState,
};

pub fn check_rauser_level() {
  // check missions to see if we need to trigger a check
  GameState::with(|client, memory, _| {
    let local_level = memory.get_rauser_level();
    let remote_levels: HashSet<_> = client
      .client()
      .checked_locations()
      .filter_map(|l| APLocation::try_from(l).ok())
      .filter_map(|l| l.as_rauser_level())
      .collect();
    let max = {
      let local = RauserLevel::try_new(local_level);
      let remote = remote_levels.iter().max().cloned();
      if let Some(max) = max(local, remote) {
        max
      } else {
        return; // both are too low
      }
    };

    // make sure we have the correct checks. while I'd like to leave in holes
    // so collecting doesn't increase your level, I think it's more importaint
    // to sync the remote level with the local one in case multiple people play
    // the same slot
    if local_level < max.get() {
      memory.set_rauser_level(max.get());
    }
    let _ = client.client().mark_checked(
      (2..=max.get())
        .map(|l| RauserLevel::try_new(l).unwrap())
        .filter(|l| !remote_levels.contains(l)),
    );
  });
}
