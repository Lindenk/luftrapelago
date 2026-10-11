import pkgutil, random
from typing import Any, TextIO, cast, ClassVar

import orjson

from BaseClasses import Item, ItemClassification, MultiWorld, Tutorial, Region
from worlds.AutoWorld import WebWorld, World

import Names
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

    if options.goal != 0: # launch urauser
      self.excluded_location.append(Names.MiscLocations.LaunchURauser)
    if options.goal not in [4, 5]: # score attack
      self.excluded_locations.append(Names.MiscLocations.ScoreAttack)

  def create_regions(self) -> None:
    """
    This game doesn't really have regions, so just make a menu region. Regions
    could be used for mission tiers or part missions, but since they overlap
    I'd prefer to handle both requirements in the same way
    """
    menu = Region("Menu", self.player, self.multiworld)

    menu.locations.extend([LuftrausersLocation(self.player, loc.name, self.base_id + loc.code)
      for loc in all_locations
      if loc.name not in self.exluded_locations])

    self.multiworld.regions.append(menu)


  def create_item(self, name: str) -> Item:
    item = item_table[name]
    return LuftrausersItem(name, item.progression, self.base_id + item.code, self.player)

  def create_items(self) -> None:
    options: LuftrausersOptions = cast(LuftrausersOptions, self.options)
    total_locations = len(self.multiworld.get_unfilled_locations(self.player))
    required_items = []

    # parts
    required_items.extend([item.item_name for item in parts])

    # bonuses
    required_items.extend([powerups["damage"].item_name for _n in range(options.powerup_damage)])
    required_items.extend([powerups["health"].item_name for _n in range(options.powerup_health)])
    required_items.extend([powerups["firerate"].item_name for _n in range(options.powerup_fire_rate)])

    # rank keys
    if options.enable_rank_keys:
      n = (6 - options.rank_key_mission_lock) + options.extra_rank_keys
      required_items.extend([misc["rank_keys"].item_name for _n in range(n)])

    # filler and traps
    total_filler = len(required_items) - total_locations

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
      "Missle Trap": Names.Filler.TrapMissles
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

    # misc rules
    add_rule(self.get_location(Names.MiscLocations.LaunchURauser),
      lambda state: state.has_all([
        Names.WeaponNames.URauserWeapon, Names.BodyNames.URauserBody, Names.EngineNames.URauserEngine
      ], self.player))

    # Set goal
    victory_item = LuftrausersItem(Names.MiscItems.Victory, ItemClassification.progression, None, self.player)
    if options.goal == 0: # URauser
      loc = self.get_location(Names.MiscLocations.LaunchURauser)
    elif options.goal in [1, 2, 3]: # Unique 'completions'
      loc = self.get_location(Names.generic_completions[options.required_unique_planes-1])
    elif options.goal in [4, 5]: # Score based
      loc = self.get_location(Names.MiscLocations.ScoreAttack)
    else:
      raise Exception("Somehow no goal was set")

    loc.place_locked_item(victory_item)
    self.multiworld.completion_condition[self.player] = \
      lambda state: state.has(Names.MiscItems.Victory, self.player)
