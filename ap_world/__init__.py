import pkgutil, random
from typing import Any, TextIO, cast, ClassVar

import orjson

from BaseClasses import Item, ItemClassification, MultiWorld, Tutorial, Region
from worlds.AutoWorld import WebWorld, World

from . import Names
from worlds.generic.Rules import add_rule
from .Options import Goal, LuftrausersOptions, luftrausers_option_groups
from .Items import LuftrausersItem, item_groups, items_by_id, item_table, parts, powerups, misc
from .Locations import LuftrausersLocation, all_locations, location_table, location_groups, mission_locations

class LuftrausersWebWorld(WebWorld):
  def __init__(self):
    self.theme = "ocean"
    self.tutorials = [
      Tutorial(
        tutorial_name="Setup Guide",
        description="A guild for setting up Luftrausers for AP",
        language="English",
        file_name="guide_en.md",
        link="guide/en",
        authors=["lindenk"],
      )
    ]
  option_groups = luftrausers_option_groups

BASE_ID = 0x10F7A405345

class LuftrausersWorld(World):
  game = "Luftrausers"
  web = LuftrausersWebWorld()
  options_dataclass = LuftrausersOptions
  base_id = BASE_ID
  topology_present = False

  item_name_to_id: ClassVar[dict[str, int]] = {n: BASE_ID + v.code for n, v in item_table.items()}
  location_name_to_id: ClassVar[dict[str, int]] = {name: BASE_ID + v.code for name, v in location_table.items()}

  item_name_groups = item_groups
  location_name_groups = location_groups

  def __init__(self, multiworld: MultiWorld, player: int):
    super().__init__(multiworld, player)

    self.excluded_locations: list[str] = []

  def generate_early(self) -> None:
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

    # Remove excluded locations from the pool
    # all completions past the goal completion
    self.excluded_locations.extend(Names.generic_completions[options.required_unique_planes:])

  def create_regions(self) -> None:
    """
    This game doesn't really have regions, so just make a menu region. Regions
    could be used for mission tiers or part missions, but since they overlap
    I'd prefer to handle both requirements in the same way
    """
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)
    menu = Region("Menu", self.player, self.multiworld)

    # exclude locations
    locations = [LuftrausersLocation(self.player, loc.name, self.base_id + loc.code)
      for loc in all_locations
      if loc.name not in self.excluded_locations]

    # victory locations
    if options.goal == 0: # launch urauser
      locations.append(
        LuftrausersLocation(self.player, Names.VictoryLocations.LaunchURauser, None))
    elif options.goal in [1, 2, 3]: # Unique 'completions'
      locations.append(
        LuftrausersLocation(self.player, Names.VictoryLocations.FinishedUniqueCompletions, None))
    elif options.goal in [4, 5]: # score attack
      locations.append(
        LuftrausersLocation(self.player, Names.VictoryLocations.ScoreAttack, None))

    # add them in
    for loc in locations:
      loc.parent_region = menu

    menu.locations.extend(locations)

    self.multiworld.regions.append(menu)


  def create_item(self, name: str) -> Item:
    item = item_table[name]
    return LuftrausersItem(name, item.progression, self.base_id + item.code, self.player)

  def create_items(self) -> None:
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)
    total_locations = len(self.multiworld.get_unfilled_locations(self.player))
    required_items = []

    # parts
    # give the player a starting plane
    starting_parts = []
    if options.randomize_starting_loadout:
      weapon = Names.WeaponNames.iter()
      weapon.remove(Names.WeaponNames.URauserWeapon)
      starting_parts.append(random.choice(weapon))

      body = Names.BodyNames.iter()
      body.remove(Names.BodyNames.URauserBody)
      starting_parts.append(random.choice(body))

      engine = Names.EngineNames.iter()
      engine.remove(Names.EngineNames.URauserEngine)
      starting_parts.append(random.choice(engine))
    else:
      starting_parts = [
        Names.WeaponNames.OriginalWeapon,
        Names.BodyNames.OriginalBody,
        Names.EngineNames.OriginalEngine
      ]
    for part in starting_parts:
      self.multiworld.push_precollected(self.create_item(part))

    pool_parts = [item.item_name for item in parts if item.item_name not in starting_parts]
    required_items.extend(pool_parts)

    # bonuses
    required_items.extend([powerups["damage"].item_name for _n in range(options.powerup_damage)])
    required_items.extend([powerups["health"].item_name for _n in range(options.powerup_health)])
    required_items.extend([powerups["firerate"].item_name for _n in range(options.powerup_fire_rate)])

    # rank keys
    if options.enable_rank_keys:
      n = (6 - options.rank_key_mission_lock) + options.extra_rank_keys
      required_items.extend([misc["rank_keys"].item_name for _n in range(n)])

    # filler and traps, 1 extra for the victory item
    total_filler = total_locations - len(required_items) - 1

    filler_names = list(options.filler_weights.keys())
    filler_weights = list(options.filler_weights.values())
    filler_items = random.choices(
      population=filler_names,
      weights=filler_weights,
      k=total_filler
    )

    filler_map = {
      "Patch Up": Names.Filler.Patchup,
      "Blank": Names.Filler.Blank,
      "Gun Jam Trap": Names.Filler.TrapJam,
      "Spawn Trap": Names.Filler.TrapSpawn,
      "Missile Trap": Names.Filler.TrapMissiles
    }
    filler_items = [filler_map[item] for item in filler_items]

    self.multiworld.itempool.extend([self.create_item(item) for item in required_items + filler_items])

  def set_rules(self) -> None:
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

    # Set mission rules
    for mission in mission_locations:
      if mission.part_req:
        part_req = cast(str, mission.part_req)
        add_rule(self.get_location(mission.name),
          lambda state, part_req=part_req: state.has(cast(str, part_req), self.player))

      if mission.mission_num and options.enable_rank_keys:
        mission_num = cast(int, mission.mission_num)
        # lock of 3 means 1 key is needed for mission 3, 2 for mission 4, etc
        req_keys = mission_num - options.rank_key_mission_lock + 1
        add_rule(self.get_location(mission.name),
          lambda state, req_keys=req_keys: state.has(Names.MiscItems.ProgressiveRank, self.player, count=req_keys))

    # level scaling rules (ie. don't put early items at high rauser levels
    # to avoid excessive grinding)
    for (i, loc) in enumerate(Names.level_names):
      add_rule(self.get_location(loc), lambda state, i=i: state.has_group_unique("Parts", self.player, count=i))

    # difficulty logic. Ensure the player has enough powerups before expecting completions
    total_powerups = options.powerup_damage + options.powerup_health + options.powerup_fire_rate
    for c in Names.generic_completions[:options.required_unique_planes]:
      add_rule(self.get_location(c), lambda state: state.has_group("Powerups", self.player, count=total_powerups//2))

    # Set goal
    victory_item = LuftrausersItem(Names.MiscItems.Victory, ItemClassification.progression, None, self.player)
    if options.goal == 0: # URauser
      loc = self.get_location(Names.VictoryLocations.LaunchURauser)
      add_rule(loc,
        lambda state: state.has_all([
          Names.WeaponNames.URauserWeapon, Names.BodyNames.URauserBody, Names.EngineNames.URauserEngine
        ], self.player))
    elif options.goal in [1, 2, 3]: # Unique 'completions'
      loc_completion = Names.generic_completions[options.required_unique_planes-1]
      loc = self.get_location(Names.VictoryLocations.FinishedUniqueCompletions)
      add_rule(loc, lambda state: state.can_reach_location(loc_completion, self.player))
    elif options.goal in [4, 5]: # Score based
      loc = self.get_location(Names.VictoryLocations.ScoreAttack)
      # assume the player must get all powerups to acheive the score
      add_rule(loc, lambda state: state.has_all_counts({
        Names.Bonuses.PowerupDamage: int(options.powerup_damage),
        Names.Bonuses.PowerupDamage: int(options.powerup_health),
        Names.Bonuses.PowerupDamage: int(options.powerup_fire_rate),
      }, self.player))
    else:
      raise Exception("Somehow no goal was set")

    loc.place_locked_item(victory_item)
    self.multiworld.completion_condition[self.player] = \
      lambda state: state.has(Names.MiscItems.Victory, self.player)

  def fill_slot_data(self) -> dict[str, Any]:
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

    # manually creating this to decouple it from future options changes
    if options.goal == 0:
      goal = { "name": "urauser" }
    elif options.goal == 1:
      goal = {
        "name": "unique_blimp_kills",
        "planes": options.required_unique_planes.value
      }
    elif options.goal == 2:
      goal = {
        "name": "unique_classified1_kills",
        "planes": options.required_unique_planes.value
      }
    elif options.goal == 3:
      goal = {
        "name": "unique_classified2_kills",
        "planes": options.required_unique_planes.value
      }
    elif options.goal == 4:
      goal = {
        "name": "score_attack",
        "score": options.required_score.value
      }
    elif options.goal == 5:
      goal = {
        "name": "score_total",
        "score": options.required_score.value
      }
    else:
      raise Exception("Somehow no goal was set")

    data = {
      "goal": goal,
      "starting_power": options.starting_power.value,
      "powerup_bonuses": {
        "damage": options.powerup_damage_bonus.value,
        "health": options.powerup_health_bonus.value,
        "fire_rate": options.powerup_fire_rate_bonus.value
      }
    }

    if options.enable_rank_keys:
      data["rank_key_mission_lock"] = options.rank_key_mission_lock.value

    if options.deathlink:
      data["deathlink_counter"] = options.deathlink_counter.value

    # TODO: Missionsanity

    return data
