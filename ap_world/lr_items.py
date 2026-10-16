import typing

from BaseClasses import Item, ItemClassification

from .lr_names import BodyNames, Bonuses, EngineNames, Filler, MiscItems, WeaponNames


class LuftrausersItem(Item):
    game = "Luftrausers"


class LuftrausersItemData(typing.NamedTuple):
    code: int
    item_name: str
    progression: ItemClassification


parts: list[LuftrausersItemData] = [
    LuftrausersItemData(0x0000, WeaponNames.OriginalWeapon, ItemClassification.progression),
    LuftrausersItemData(0x0001, WeaponNames.Laser, ItemClassification.progression),
    LuftrausersItemData(0x0002, WeaponNames.Spread, ItemClassification.progression),
    LuftrausersItemData(0x0003, WeaponNames.Missles, ItemClassification.progression),
    LuftrausersItemData(0x0004, WeaponNames.Cannon, ItemClassification.progression),
    LuftrausersItemData(0x0005, WeaponNames.URauserWeapon, ItemClassification.progression),
    LuftrausersItemData(0x0020, BodyNames.OriginalBody, ItemClassification.progression),
    LuftrausersItemData(0x0021, BodyNames.Armor, ItemClassification.progression),
    LuftrausersItemData(0x0022, BodyNames.Melee, ItemClassification.progression),
    LuftrausersItemData(0x0023, BodyNames.Nuke, ItemClassification.progression),
    LuftrausersItemData(0x0024, BodyNames.Bomb, ItemClassification.progression),
    LuftrausersItemData(0x0025, BodyNames.URauserBody, ItemClassification.progression),
    LuftrausersItemData(0x0040, EngineNames.OriginalEngine, ItemClassification.progression),
    LuftrausersItemData(0x0041, EngineNames.Superboost, ItemClassification.progression),
    LuftrausersItemData(0x0042, EngineNames.Gungine, ItemClassification.progression),
    LuftrausersItemData(0x0043, EngineNames.Underwater, ItemClassification.progression),
    LuftrausersItemData(0x0044, EngineNames.Hover, ItemClassification.progression),
    LuftrausersItemData(0x0045, EngineNames.URauserEngine, ItemClassification.progression),
]

powerups: dict[str, LuftrausersItemData] = {
    "damage": LuftrausersItemData(0x0080, Bonuses.PowerupDamage, ItemClassification.progression),
    "health": LuftrausersItemData(0x0081, Bonuses.PowerupHealth, ItemClassification.progression),
    "firerate": LuftrausersItemData(0x0082, Bonuses.PowerupFireRate, ItemClassification.progression),
}

misc: dict[str, LuftrausersItemData] = {
    "rank_keys": LuftrausersItemData(0x00C0, MiscItems.ProgressiveRank, ItemClassification.progression),
    "completion_token": LuftrausersItemData(0x00C1, MiscItems.CompletionToken, ItemClassification.progression),
}

filler: list[LuftrausersItemData] = [
    LuftrausersItemData(0x0100, Filler.Patchup, ItemClassification.filler),
    LuftrausersItemData(0x0101, Filler.Blank, ItemClassification.filler),
]

traps: list[LuftrausersItemData] = [
    LuftrausersItemData(0x0140, Filler.TrapJam, ItemClassification.trap),
    LuftrausersItemData(0x0140, Filler.TrapSpawn, ItemClassification.trap),
    LuftrausersItemData(0x0140, Filler.TrapMissiles, ItemClassification.trap),
]

item_groups: dict[str, set[str]] = {
    "Parts": {n.item_name for n in parts},
    "Powerups": {n.item_name for n in powerups.values()},
    "Misc": {n.item_name for n in misc.values()},
    "Filler": {n.item_name for n in filler},
    "Traps": {n.item_name for n in traps},
}

all_items: list[LuftrausersItemData] = parts + list(powerups.values()) + list(misc.values()) + filler + traps
item_table: dict[str, LuftrausersItemData] = {item.item_name: item for item in all_items}
items_by_id: dict[int, LuftrausersItemData] = {item.code: item for item in all_items}
