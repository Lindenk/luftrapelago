from typing import Any, ClassVar, cast

from BaseClasses import Item, ItemClassification, MultiWorld, Region, Tutorial
from worlds.AutoWorld import WebWorld, World
from worlds.generic.Rules import add_rule

from . import lr_names
from .lr_items import LuftrausersItem, item_groups, item_table, misc, parts, powerups
from .lr_locations import (
    LuftrausersLocation,
    LuftrausersLocationData,
    all_locations,
    generic_completion_locations,
    location_groups,
    location_table,
    mission_locations,
    unique_completion_locations,
)
from .lr_options import LuftrausersOptions, luftrausers_option_groups


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


BASE_ID = 0x1f700000


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
        self.unique_completion_locations: list[LuftrausersLocationData] = []
        self.generic_completion_locations: list[LuftrausersLocationData] = []

    def generate_early(self) -> None:
        options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

        # Remove excluded locations from the pool

        if options.use_unique_completion_checks:
            # don't use generic locations
            self.excluded_locations.extend(lr_names.generic_completions)

            # add in random unique completions up to the set amount
            completions = list(unique_completion_locations)
            self.random.shuffle(completions)
            self.unique_completion_locations.extend(completions[:options.required_completions])
            self.excluded_locations.extend([c.name for c in completions[options.required_completions:]])
        else:
            # allow all unique completions (because they'll just give tokens used for
            #   the real checks)
            self.unique_completion_locations.extend(list(unique_completion_locations))
            # add in generic completions up to what's set in options
            self.generic_completion_locations.extend(generic_completion_locations[:options.required_completions])
            self.excluded_locations.extend(lr_names.generic_completions[options.required_completions :])

        # for now remove all the urauser mission checks
        self.excluded_locations.extend([name for ((part, _), name) in lr_names.weapon_missions.items()
          if part == lr_names.WeaponNames.URauserWeapon])
        self.excluded_locations.extend([name for ((part, _), name) in lr_names.body_missions.items()
          if part == lr_names.BodyNames.URauserBody])
        self.excluded_locations.extend([name for ((part, _), name) in lr_names.engine_missions.items()
          if part == lr_names.EngineNames.URauserEngine])

    def create_regions(self) -> None:
        """
        This game doesn't really have regions, so just make a menu region. Regions
        could be used for mission tiers or part missions, but since they overlap
        I'd prefer to handle both requirements in the same way
        """
        options: LuftrausersOptions = cast(LuftrausersOptions, self.options)
        menu = Region("Menu", self.player, self.multiworld)

        # exclude locations
        locations = [
            LuftrausersLocation(self.player, loc.name, self.base_id + loc.code)
            for loc in all_locations
            if loc.name not in self.excluded_locations
        ]

        # victory locations
        if options.goal == 0:  # launch urauser
            locations.append(LuftrausersLocation(self.player, lr_names.VictoryLocations.LaunchURauser, None))
        elif options.goal in [1, 2, 3]:  # Unique 'completions'
            locations.append(
                LuftrausersLocation(self.player, lr_names.VictoryLocations.FinishedUniqueCompletions, None)
            )
        elif options.goal in [4, 5]:  # score attack
            locations.append(LuftrausersLocation(self.player, lr_names.VictoryLocations.ScoreAttack, None))

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
            weapon = list(lr_names.WeaponNames)
            weapon.remove(lr_names.WeaponNames.URauserWeapon)
            starting_parts.append(self.random.choice(weapon))

            body = list(lr_names.BodyNames)
            body.remove(lr_names.BodyNames.URauserBody)
            starting_parts.append(self.random.choice(body))

            engine = list(lr_names.EngineNames)
            engine.remove(lr_names.EngineNames.URauserEngine)
            starting_parts.append(self.random.choice(engine))
        else:
            starting_parts = [
                lr_names.WeaponNames.OriginalWeapon,
                lr_names.BodyNames.OriginalBody,
                lr_names.EngineNames.OriginalEngine,
            ]
        for part in starting_parts:
            self.multiworld.push_precollected(self.create_item(part))

        pool_parts = [item.item_name for item in parts if item.item_name not in starting_parts]
        required_items.extend(pool_parts)

        # bonuses
        required_items.extend([powerups["health"].item_name for _n in range(options.powerup_health)])
        required_items.extend([powerups["firerate"].item_name for _n in range(options.powerup_fire_rate)])
        required_items.extend([powerups["bonusbullets"].item_name for _n in range(options.powerup_bullets)])

        # rank keys
        if options.enable_rank_keys:
            n = (6 - options.rank_key_mission_lock) + options.extra_rank_keys
            required_items.extend([misc["rank_keys"].item_name for _n in range(n)])

        # prefill the unique plane completions to give tokens
        # generic completions triggered by tokens should give actual items
        if not options.use_unique_completion_checks:
            for loc_data in unique_completion_locations:
                loc = self.get_location(loc_data.name)
                loc.place_locked_item(self.create_item(lr_names.MiscItems.CompletionToken))

        # filler and traps, 1 extra for the victory item
        total_filler = total_locations - len(required_items) - 1
        if not options.use_unique_completion_checks:
          # removed locked item locations
          total_filler -= len(unique_completion_locations)

        filler_names = list(options.filler_weights.keys())
        filler_weights = list(options.filler_weights.values())
        filler_items = self.random.choices(population=filler_names, weights=filler_weights, k=total_filler)

        filler_map = {
            "Patch Up": lr_names.Filler.Patchup,
            #"Blank": lr_names.Filler.Blank,
            "Gun Jam Trap": lr_names.Filler.TrapJam,
            "Engine Stall Trap": lr_names.Filler.TrapStall,
        }
        filler_items = [filler_map[item] for item in filler_items]

        self.multiworld.itempool.extend([self.create_item(item) for item in required_items + filler_items])

    def set_rules(self) -> None:
        options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

        # Set mission rules
        for mission in (m for m in mission_locations if m.name not in self.excluded_locations):
            if mission.part_req:
                part_req = cast(str, mission.part_req)
                add_rule(
                    self.get_location(mission.name),
                    lambda state, part_req=part_req: state.has(cast(str, part_req), self.player),
                )

            if mission.mission_num and options.enable_rank_keys:
                mission_num = cast(int, mission.mission_num)
                # lock of 3 means 1 key is needed for mission 3, 2 for mission 4, etc
                req_keys = mission_num - options.rank_key_mission_lock + 1
                if req_keys > 0:
                    add_rule(
                        self.get_location(mission.name),
                        lambda state, req_keys=req_keys: state.has(
                            lr_names.MiscItems.ProgressiveRank, self.player, count=req_keys
                        ),
                    )

        # plane completions
        weapon_names = list(lr_names.WeaponNames)
        body_names = list(lr_names.BodyNames)
        engine_names = list(lr_names.EngineNames)
        for name, build in lr_names.unique_planes.items():
            # skip any that aren't in this world
            if name not in self.unique_completion_locations:
              continue

            add_rule(
                self.get_location(name),
                lambda state, build=build: state.has_all(
                    [weapon_names[build["weapon"]], body_names[build["body"]], engine_names[build["engine"]]],
                    self.player,
                ),
            )

        # handle progressive completions. have unique locations produce an
        # event token, and require each generic location to obtain that many
        # completion tokens
        if not options.use_unique_completion_checks:
            for i, loc_data in enumerate(self.generic_completion_locations):
                add_rule(
                    self.get_location(loc_data.name),
                    lambda state, i=i: state.has(lr_names.MiscItems.CompletionToken, self.player, count=i + 1),
                )

        # level scaling rules (ie. don't put early items at high rauser levels
        # to avoid excessive grinding). Put 1 in logic to start, unlocked by more parts
        for i, loc in enumerate(lr_names.level_names):
            add_rule(self.get_location(loc), lambda state, i=i: state.has_group_unique("Parts", self.player, count=i+3))

        # difficulty logic. Ensure the player has enough powerups before expecting completions
        total_powerups = options.powerup_health + options.powerup_fire_rate + options.powerup_bullets
        for c in self.unique_completion_locations:
            add_rule(
                self.get_location(c.name), lambda state:
                  state.has_group("Powerups", self.player, count=total_powerups // 2)
            )

        # Set goal
        victory_item = LuftrausersItem(lr_names.MiscItems.Victory, ItemClassification.progression, None, self.player)
        if options.goal == 0:  # URauser
            loc = self.get_location(lr_names.VictoryLocations.LaunchURauser)
            add_rule(
                loc,
                lambda state: state.has_all(
                    [
                        lr_names.WeaponNames.URauserWeapon,
                        lr_names.BodyNames.URauserBody,
                        lr_names.EngineNames.URauserEngine,
                    ],
                    self.player,
                ),
            )
        elif options.goal == 1:  # Unique completions
            loc = self.get_location(lr_names.VictoryLocations.FinishedUniqueCompletions)
            if options.use_unique_completion_checks:
              for c in self.unique_completion_locations:
                add_rule(loc, lambda state, c=c: state.can_reach_location(c.name, self.player))
            else:
              loc_completion = self.generic_completion_locations[-1]
              add_rule(loc, lambda state: state.can_reach_location(loc_completion.name, self.player))
        else:
            raise Exception("Somehow no goal was set")

        loc.place_locked_item(victory_item)
        self.multiworld.completion_condition[self.player] = lambda state: state.has(
            lr_names.MiscItems.Victory, self.player
        )

    def fill_slot_data(self) -> dict[str, Any]:
        options: LuftrausersOptions = cast(LuftrausersOptions, self.options)

        # manually creating this to decouple it from future options changes
        if options.goal == 0:
            goal = "urauser"
        elif options.goal == 1:
            goal = "completions"
        else:
            raise Exception("Somehow no goal was set")

        if options.completion_condition == 0: # blimp kill
          completion_condition = "blimp"
        elif options.completion_condition == 10:
          completion_condition = "score"
        else:
          raise Exception("Somehow no valid completion condition was set")

        data = {
            "goal": goal,
            "completion_condition": completion_condition,
            "completion_locations": [c.code for c in self.unique_completion_locations],
            "completion_score": options.required_score.value,
            "send_generic_completions": not options.use_unique_completion_checks,
            "starting_health": options.starting_health.value,
            "powerup_bonuses": {
                "health": options.powerup_health_bonus.value,
            },
            "enable_sfmt": options.enable_sfmt.value > 0
        }

        if options.enable_rank_keys:
            data["rank_key_mission_lock"] = options.rank_key_mission_lock.value

        if options.deathlink:
            data["deathlink_counter"] = options.deathlink_counter.value

        # TODO: Missionsanity

        return data
