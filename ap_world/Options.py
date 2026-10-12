import typing
from dataclasses import dataclass

from schema import Schema, Optional

from Options import OptionGroup, Choice, DefaultOnToggle, PerGameCommonOptions, Range, StartInventoryPool, Toggle, OptionCounter


# Future option ideas:
# - Mission sync (use the same seed for mission rando so
#   all players in the ap have the same randomized missions).
# - Progressive plane parts (upgrade the parts as you get dups).
# - Blank filler. Instantly clear the screen of projectiles and
#   deal damage around the player
# - SFMT mode?
# - New plane parts
#   - Rubber (Body) -> bounce off water/space at high speeds
#   - Shield (Weapon) -> Slow fire rate, makes you invincable and deal
#                         heavy damage in a small radius around you


# Goal options
class Goal(Choice):
  """
  The victory condition:

  URauser                   Gather the CLASSIFIED plane parts and launch the URauser
  Unique Blimp Kills        Defeat the Blimp with X unique planes
  Unique CLASSIFIED1 Kills  Defeat the CLASSIFIED 1 with X unique planes
  Unique CLASSIFIED2 Kills  Defeat the CLASSIFIED 2 with X unique planes
  Score Attack              Reach X score in one run
  Score Total               Reach X total score (across all runs)
  """
  display_name = "Goal"
  option_urauser = 0
  option_unique_blimp_kills = 1
  option_unique_classified1_kills = 2
  option_unique_classified2_kills = 3
  option_score_attack = 4
  option_score_total = 5

class RequiredUniquePlanes(Range):
  """
  The number of unique plane builds that must be used for the Unique Blimp Kills,
  Unique CLASSIFIED1 Kills, and Unique CLASSIFIED2 Kills goals. For other goals, it
  signifies the number of checks behind unique build blimp kills.
  """
  display_name = "Required Unique Planes"
  range_start = 1
  range_end = 125
  default = 10

class RequiredScore(Range):
  """
  The score needed for Score Attack and Score Total goals. As a rule of thumb, a
  score of 30,000 is a very strong single run, and a total score of 250,000 per
  hour can be expected for a seasoned player.
  Does nothing for other goals.
  """
  display_name = "Required Score"
  range_start = 1000
  range_end = 10_000_000
  default = 30_000


# Sanity options
class RandomizeStartingLoadout(DefaultOnToggle):
  """
  Randomize your starting parts (not including CLASSIFIED parts)
  """
  display_name = "Randomize Starting Loadout"

class Missionsanity(Toggle):
  """
  Randomize missions (somewhat sanely). Not for the faint of heart.
  """
  display_name = "Missionsanity"

# Item options
class EnableRankKeys(DefaultOnToggle):
  """
  The default game progression locks off parts and missions until specific enemies
  have been defeated for the first time. For Archipelago, these requirements are
  lifted entirely, however progressive Rank Keys can be included in the pool,
  preventing completion of later missions for each part until another rank key
  is found.
  """
  display_name = "Enable Rank Keys"

class RankKeyMissionLock(Range):
  """
  Which missions are blocked off by rank keys, out of the five total for each
  part. For example, '3' means missions 3, 4, and 5 are locked until 1, 2, and
  3 keys are found (respectively).
  """
  display_name = "Rank Key Minimum Mission"
  range_start = 2
  range_end = 5
  default = 3

class ExtraRankKeys(Range):
  """
  How many extra rank keys to include in the pool.
  """
  display_name = "Extra Rank Keys"
  range_start = 0
  range_end = 4
  default = 2

class StartingPower(Range):
  """
  How much to scale your starting Health and Damage (in percent). Lower values make
  you weaker to start, higher values make you stronger.
  """
  display_name = "Starting Power"
  range_start = 50
  range_end = 200
  default = 80

class PowerupDamage(Range):
  """
  The number of bonus damage up items to mix into the pool. Makes the game easier
  as these are found.
  """
  display_name = "Powerup Damage"
  range_start = 0
  range_end = 10
  default = 4

class PowerupHealth(Range):
  """
  The number of bonus heath up items to mix into the pool. Makes the game easier
  as these are found.
  """
  display_name = "Powerup Health"
  range_start = 0
  range_end = 10
  default = 4

class PowerupFireRate(Range):
  """
  The number of bonus fire rate up items to mix into the pool. Makes the game easier
  as these are found.
  """
  display_name = "Powerup Fire rate"
  range_start = 0
  range_end = 5
  default = 2

class PowerupDamageBonus(Range):
  """
  What percentage bonus each Powerup Damage gives.
  """
  display_name = "Powerup Damage Bonus"
  range_start = 5
  range_end = 100
  default = 25

class PowerupHealthBonus(Range):
  """
  What percentage bonus each Powerup Health gives.
  """
  display_name = "Powerup Health Bonus"
  range_start = 5
  range_end = 100
  default = 25

class PowerupFireRateBonus(Range):
  """
  What percentage bonus each Powerup Fire Rate gives.
  """
  display_name = "Powerup Fire Rate Bonus"
  range_start = 5
  range_end = 100
  default = 50

class FillerWeights(OptionCounter):
    """
    The weighting for each filler item (including traps)
    """
    display_name = "Filler Weights"

    # Define valid keys directly so the UI and generator recognize them
    valid_keys = {
      "Patch Up",
      "Blank",
      "Gun Jam Trap",
      "Spawn Trap",
      "Missile Trap",
    }

    default = {
      "Patch Up": 5,
      "Blank": 2,
      "Gun Jam Trap": 2,
      "Spawn Trap": 2,
      "Missile Trap": 2,
    }

# Link options
class DeathLink(Choice):
  """
  When you die a configurable number of times, everyone dies. When any other
  deathlink player dies, you die.
  """
  display_name = "Death Link"
  option_off = 0
  option_on = 1

class DeathLinkCounter(Range):
  """
  The number of deaths needed to send a deathlink to other players.
  """
  display_name = "Death Link Count"
  range_start = 1
  range_end = 100
  default = 20

@dataclass
class LuftrausersOptions(PerGameCommonOptions):
  goal: Goal
  required_unique_planes: RequiredUniquePlanes
  required_score: RequiredScore
  randomize_starting_loadout: RandomizeStartingLoadout
  missionsanity: Missionsanity
  enable_rank_keys: EnableRankKeys
  rank_key_mission_lock: RankKeyMissionLock
  extra_rank_keys: ExtraRankKeys
  starting_power: StartingPower
  powerup_damage: PowerupDamage
  powerup_health: PowerupHealth
  powerup_fire_rate: PowerupFireRate
  powerup_damage_bonus: PowerupDamageBonus
  powerup_health_bonus: PowerupHealthBonus
  powerup_fire_rate_bonus: PowerupFireRateBonus
  filler_weights: FillerWeights
  deathlink: DeathLink
  deathlink_counter: DeathLinkCounter

luftrausers_option_groups: list[OptionGroup] = [
  #OptionGroup("Game", [Goal, RequiredUniquePlanes, RequiredScore]),
  OptionGroup("Sanity", [RandomizeStartingLoadout, Missionsanity]),
  OptionGroup("Items", [EnableRankKeys, RankKeyMissionLock, ExtraRankKeys,
    StartingPower, PowerupDamage, PowerupHealth, PowerupFireRate,
    PowerupDamageBonus, PowerupHealthBonus, PowerupFireRateBonus,
    FillerWeights
  ]),
]
