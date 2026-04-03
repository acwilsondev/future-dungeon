import json

with open('content.json', 'r') as f:
    data = json.load(f)

data['monsters'].append({
    "name": "Garden Snake",
    "glyph": "s",
    "color": [0, 200, 0],
    "hp": 12,
    "defense": 1,
    "power": 4,
    "faction": "Animals",
    "personality": "Tactical",
    "viewshed": 6,
    "xp_reward": 20,
    "ranged": None,
    "spawn_chance": 0.4,
    "min_floor": 2,
    "max_floor": 999,
    "is_boss": False,
    "phases": None,
    "guaranteed_loot": None,
    "branches": ["Gardens"]
})

data['monsters'].append({
    "name": "Ice Elemental",
    "glyph": "E",
    "color": [100, 100, 255],
    "hp": 25,
    "defense": 3,
    "power": 6,
    "faction": "Animals",
    "personality": "Brave",
    "viewshed": 8,
    "xp_reward": 30,
    "ranged": None,
    "spawn_chance": 0.4,
    "min_floor": 2,
    "max_floor": 999,
    "is_boss": False,
    "phases": None,
    "guaranteed_loot": None,
    "branches": ["Vaults"]
})

data['items'].append({
    "name": "Frost Wand",
    "glyph": "/",
    "color": [100, 100, 255],
    "price": 50,
    "potion": None,
    "weapon": 3,
    "armor": None,
    "ranged": None,
    "ranged_weapon": None,
    "aoe": None,
    "confusion": None,
    "poison": None,
    "ammo": False,
    "consumable": False,
    "spawn_chance": 0.1,
    "min_floor": 2,
    "max_floor": 999,
    "obfuscated_name": "Cold Stick",
    "cursed": False,
    "slot": "Melee",
    "branches": ["Vaults"]
})

data['items'].append({
    "name": "Vine Whip",
    "glyph": "/",
    "color": [0, 200, 0],
    "price": 45,
    "potion": None,
    "weapon": 4,
    "armor": None,
    "ranged": None,
    "ranged_weapon": None,
    "aoe": None,
    "confusion": None,
    "poison": None,
    "ammo": False,
    "consumable": False,
    "spawn_chance": 0.1,
    "min_floor": 2,
    "max_floor": 999,
    "obfuscated_name": "Thorny Whip",
    "cursed": False,
    "slot": "Melee",
    "branches": ["Gardens"]
})

with open('content.json', 'w') as f:
    json.dump(data, f, indent=2)
