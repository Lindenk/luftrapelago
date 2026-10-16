from dataclasses import dataclass

from BaseClasses import Location

from .lr_names import body_missions, engine_missions, generic_completions, level_names, unique_planes, weapon_missions


@dataclass
class LuftrausersLocationData:
    name: str
    code: int
    part_req: str | None = None
    mission_num: int | None = None


class LuftrausersLocation(Location):
    game: str = "Luftrausers"


mission_locations = (
    [
        LuftrausersLocationData(name, i + 0x0000, part, mission_no)
        for (i, ((part, mission_no), name)) in enumerate(weapon_missions.items())
    ]
    + [
        LuftrausersLocationData(name, i + 0x0100, part, mission_no)
        for (i, ((part, mission_no), name)) in enumerate(body_missions.items())
    ]
    + [
        LuftrausersLocationData(name, i + 0x0200, part, mission_no)
        for (i, ((part, mission_no), name)) in enumerate(engine_missions.items())
    ]
)

level_locations = [LuftrausersLocationData(name, i + 0x300) for i, name in enumerate(level_names)]

generic_completion_locations = [LuftrausersLocationData(name, i + 0x400) for i, name in enumerate(generic_completions)]
unique_completion_locations = [LuftrausersLocationData(name, i + 0x600) for i, name in enumerate(unique_planes.keys())]

location_groups: dict[str, set[str]] = {
    "Missions": {loc.name for loc in mission_locations},
    "Levels": {loc.name for loc in level_locations},
    "GenericCompletions": {loc.name for loc in generic_completion_locations},
    "UniqueCompletions": {loc.name for loc in unique_completion_locations},
}

all_locations: list[LuftrausersLocationData] = \
  mission_locations \
  + level_locations \
  + generic_completion_locations \
  + unique_completion_locations
scoutable_locations: list[LuftrausersLocationData] = mission_locations
location_table: dict[str, LuftrausersLocationData] = {loc.name: loc for loc in all_locations}
