from dataclasses import dataclass

from Options import Choice, DefaultOnToggle, OptionCounter, OptionGroup, PerGameCommonOptions, Range, Toggle

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
    The victory condition.


    **URauser:** Gather the CLASSIFIED plane parts and launch the URauser

    **Completions:** Satisfy the completion condition for X unique planes
    """

    display_name = "Goal"
    rich_text_doc = True
    option_urauser = 0
    option_completions = 1

class CompletionCondition(Choice):
    """
    What counts as a "Completion" for a plane.


    **Blimp:** Kill a blimp

    **Score Attack:** Reach X score
    """
    display_name = "Completion Condition"
    rich_text_doc = True
    option_blimp = 0
    option_score_attack = 10

class RequiredCompletions(Range):
    """
    The number of unique plane builds that must earn a completion.

    Each completion will be a check even if this is not the victory condition.

    If Use Unique Completion Checks is enabled, this many random builds will be required instead.
    """

    display_name = "Required Unique Planes"
    range_start = 1
    range_end = 125
    default = 10


class RequiredScore(Range):
    """
    The score needed for Score Attack completions. Does nothing if score attack is not the completion condition.
    """

    display_name = "Required Score"
    range_start = 1000
    range_end = 500_000
    default = 30_000


# Sanity options
class Missionsanity(Toggle):
    """
    Randomize mission criteria.

    Currently not implemented.
    """

    display_name = "Missionsanity"


# Item options
class RandomizeStartingLoadout(Toggle):
    """
    Randomize your starting parts (not including CLASSIFIED parts).

    If **not** randomizing missions, this can lead to very difficult starting missions.
    """

    display_name = "Randomize Starting Loadout"
    rich_text_doc = True

class EnableRankKeys(DefaultOnToggle):
    """
    Lock the later missions for each part behind keys.
    """

    display_name = "Enable Rank Keys"


class RankKeyMissionLock(Range):
    """
    Which missions are blocked off by rank keys, out of the five total for each part.

    For example, '3' means missions 3, 4, and 5 are locked until 1, 2, and 3 keys are found (respectively).
    """

    display_name = "Rank Key Minimum Mission"
    range_start = 2
    range_end = 5
    default = 4


class ExtraRankKeys(Range):
    """
    How many extra rank keys to include in the pool.
    """

    display_name = "Extra Rank Keys"
    range_start = 0
    range_end = 4
    default = 2

class UseUniqueCompletionChecks(Toggle):
    """
    Put checks at specific completions.

    For example, this would mean getting a blimp kill with The Original would give a check specifically for getting a blimp kill with the Original, instead of getting a check for the first unique blimp kill, second, etc, without depending on the specific build that got it.
    """ # noqa: E501
    display_name = "Use Unique Completion Checks"

class StartingHealth(Range):
    """
    How much to scale your starting Health (in percent). Lower values make you weaker to start, higher values make you stronger.
    """ # noqa: E501

    display_name = "Starting Health"
    range_start = 10
    range_end = 200
    default = 80


class PowerupHealth(Range):
    """
    The number of bonus heath up items to mix into the pool. Makes the game easier as these are found.
    """

    display_name = "Powerup Health"
    range_start = 0
    range_end = 30
    default = 10

class PowerupHealthBonus(Range):
    """
    What percentage bonus each Powerup Health gives.
    """

    display_name = "Powerup Health Bonus"
    range_start = 5
    range_end = 100
    default = 30


class PowerupFireRate(Range):
    """
    The number of bonus fire rate up items to mix into the pool. Makes the game easier as these are found.
    """

    display_name = "Powerup Fire Rate"
    range_start = 0
    range_end = 10
    default = 4

class PowerupBullets(Range):
    """
    The number of bonus bullet ups to mix into the pool. Makes the game easier as these are found.
    """

    display_name = "Powerup Bullets"
    range_start = 0
    range_end = 10
    default = 4

class EnableSFMT(Toggle):
  """
  Enable SFMT mode.

  Makes the game **MUCH** more difficult. The setting is only set at start and can be changed in game.
  """
  display_name =  "Enable SFMT mode"
  rich_text_doc = True


class FillerWeights(OptionCounter):
    """
    The weighting for each filler item (including traps).
    """

    display_name = "Filler Weights"

    # Define valid keys directly so the UI and generator recognize them
    valid_keys = (
        "Patch Up",
        #"Blank",
        "Gun Jam Trap",
        "Engine Stall Trap",
    )

    default = { # noqa: RUF012
        "Patch Up": 5,
        #"Blank": 2,
        "Gun Jam Trap": 2,
        "Engine Stall Trap": 2,
    }


# Link options
class DeathLink(Choice):
    """
    When you die a configurable number of times, everyone dies. When any other deathlink player dies, you die.
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
    completion_condition: CompletionCondition
    required_completions: RequiredCompletions
    required_score: RequiredScore
    randomize_starting_loadout: RandomizeStartingLoadout
    missionsanity: Missionsanity
    enable_rank_keys: EnableRankKeys
    rank_key_mission_lock: RankKeyMissionLock
    extra_rank_keys: ExtraRankKeys
    use_unique_completion_checks: UseUniqueCompletionChecks
    starting_health: StartingHealth
    powerup_health: PowerupHealth
    powerup_health_bonus: PowerupHealthBonus
    powerup_fire_rate: PowerupFireRate
    powerup_bullets: PowerupBullets
    enable_sfmt: EnableSFMT
    filler_weights: FillerWeights
    deathlink: DeathLink
    deathlink_counter: DeathLinkCounter


luftrausers_option_groups: list[OptionGroup] = [
    # OptionGroup("Game", [Goal, CompletionCondition, RequiredUniquePlanes, RequiredScore]),
    OptionGroup("Sanity", [Missionsanity]),
    OptionGroup(
        "Items",
        [
            RandomizeStartingLoadout,
            EnableRankKeys,
            RankKeyMissionLock,
            ExtraRankKeys,
            UseUniqueCompletionChecks,
            FillerWeights,
        ],
    ),
    OptionGroup(
        "Powerups and Difficulty",
        [
          StartingHealth,
          PowerupHealth,
          PowerupHealthBonus,
          PowerupFireRate,
          PowerupBullets,
          EnableSFMT
        ]
    )
]
