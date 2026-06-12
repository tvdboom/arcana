# Overview of the build system
------------------------------

## Traits
Traits is the umbrella term for modifiers and effects. They are the passive or active
bonuses a player has.


### Modifiers
Modifiers are static bonuses that apply to the player self. They are provided by weapons
and perks. Modifiers can also be negative. They apply when:
 - On perk: Always.
 - On weapon: When equipped.

Examples and corresponding description:
 - AttributeModifier(Strength, 5) → +5 strength
 - AttackModifier(-2) → -2 attack
 - PetDefenseModifier(5) → +5 pet defense
 - MaxHealthModifier(10) → +10 max health
 - KindPowerMultiplier(Fire, 10) → +10% fire damage
 - KindResistanceMultiplier(Shadow, 5) → +5% shadow resistance
 - CategoryPowerMultiplier(Melee, 5) → +5% melee damage
 - CategoryResistanceMultiplier(Range, 5) → +5% range resistance
 - LifeSteal(5) → +5% life steal
 - HealingMultiplier(5) → +5% healing


### Effects
Effects are either instant or temporary bonuses. They are provided by weapons and abilities.
They apply when:
 - On equipped weapon of categories melee, range, magical, finesse: When hitting the enemy with a basic attack.
 - On equipped weapon of categories shield, book: When hit by an enemy's basic attack.
 - On ability with `on_self=true`: Apply on the player casting.
 - On ability with `on_self=false`: Apply on the enemy if the ability hit the target.

Effects only apply during combat. They are stronger than modifiers of the same level, but
temporary.

Effects can only be applied to self, for example "Empower", and others only to enemies,
for example "Burn".

Examples and corresponding description:
 - Blind {miss_pct: 20, duration: 5} → Blind: +20% chance to miss basic attacks for 5s
 - Burn {damage: 5, duration: 10} → Burn: +5 damage/s for 10s
 - Freeze {attack_speed_pct: -10, duration: 5} → Freeze: -10% attack speed for 5s
 - ManaFlow {amount: 10, duration: 5} → Mana Flow: +10 mana/s for 5s
 - MonarchShield {duration: 5} → Monarch Shield: Total invulnerability for 5s. Cannot cast nor attack during this time.



## Build
Build is the umbrella term for perks, abilities and gear. They are the items/components a
player has in their inventory.


### Perks
Perks are static modifiers that a player has. A perk carries one or more modifiers
(never zero). A perk can have negative modifiers as well, as long as the positive net effect
is stronger, for example a perk with +3 strength and -1 wisdom. Lower level perks usually
have 1 modifier (or two of which one is negative), while a higher level perk can have up to
3 modifiers. Normal modifiers are, for example, +1 to an attribute per level of the perk, or
+5% to a kind of damage per level (doesn't have to be always linear with the level). The
strength of a perk is determined by the modifiers it has, which are stronger at higher levels.
The strength of a perk can also be determined by the number of modifiers it has, as well as
the presence of negative modifiers (a perk with more positive modifiers and fewer negative
modifiers is stronger than a perk with fewer positive modifiers and more negative modifiers).

Note the following:
 - Not two perks can have the exact same modifiers, but they can have some modifiers in
   common. For example, two perks can both have +5% fire damage, but one of them has also
   -5% shadow damage, while the other has -5% shadow resistance.

### Abilities
Abilities are active skills that a player can use during combat. They always have a cooldown
and a mana cost. Abilities are attributed to a kind (physical, fire, nature, ice, shadow, holy).
Assassins and warrior mostly use physical abilities while druids and mages use magical abilities
(the rest fo the kinds).

Abilities carry one or more effects (never zero). There are no negative effects. The strength
of an ability is determined by the effects it has, which are stronger at higher levels. The
higher the level, the (usually) more mana an ability costs and the longer the cooldown. An
ability's cooldown must always be longer than the longest duration in one of its effects, for
example an ability that applies burn for 5s must have a cooldown of at least 6s (usually more,
like 10).

Abilities apply on the player when `on_self=true`. It must only contain effects that apply to
self, for example "Empower". Abilities that apply on the enemy when `on_self=false` must only
contain effects that apply to enemies, for example "Burn". An ability cannot have effects that
apply both to self and to enemies. An ability can have multiple effects, but they must all
apply to the same target (self or enemy).

Note the following:
 - Abilities of type "holy" are the only ones that apply healing.
 - Magical abilities usually cost more mana than physical.
 - The ratio of total number of physical/magical abilities should be around 40/60.

Some combinations of kind + effects are common (not always per se, and the effects can also
be used by other kinds):
 - Physical abilities apply "Pierce", "Cleave", "Immobilize", "Lifesteal", etc...
 - Fire abilities apply "Burn".
 - Ice abilities apply "Freeze".
 - Shadow abilities apply "Paranoia".
 - Nature abilities apply "Poison".
 - Holy abilities apply "Heal", "Purge", etc...


### Equipment
Equipment is the umbrella term for anything that can be equipped, i.e., weapons and wearables.

#### Weapons
Weapons are items that a player can carry in one of its two hand slots. During combat, every
weapon attacks with basic attacks, which have a certain attack speed, attack, and crit chance
(unless attack=0, like with books and shields).

Weapons are of the same kinds as abilities (physical, fire, nature, ice, shadow, holy) and
of 5 categories: finesse, magical, melee, range, shield, book. By far the most weapons are
of kind physical, but there are also some weapons of other kinds, for example a fire wand
or a shadow dagger (usually at higher levels).

A weapon can have modifiers and effects. Modifiers apply when the weapon is equipped, while
effects apply when hitting an enemy with a basic attack (for melee, range, magical, finesse)
or when hit by an enemy's basic attack (for shield, book).

Note the following:
 - Finesse weapons usually have more crit chance and lower attack than melee/range.
 - Range weapons have lower crit chance and lower attack speed, but high attack.
 - Magical weapons have lower attack and attack speed, zero crit chance, but always have
   modifiers and/or effects.
 - Books and shields have always zero attack, attack speed, and crit chance.
 - Shields always have at least one modifier that gives defense (DefenseModifier).
 - Books always have at least one (magical) effect or modifier.
 - A player cannot wear two shields, two books, or a shield and a book at the same time.
 - Two-handed weapons must be considerable stronger than one-handed weapons of the same 
   level (around twice as strong).

#### Wearables
Wearables are all non-weapon equipment (helmet, chestplate, gloves, boots, accessories, etc...).
Wearables always have one or more have modifiers and/or effects (never zero). Modifiers apply
when the wearable is equipped, while effects apply when hit by an enemy's basic attack.


### General remarks
Same logic as for perks and abilities applies to modifiers and effects of equipment: the higher
the level, the stronger the modifiers and effects, as well as the more modifiers and effects.
The strength of equipment is determined by its modifiers and effects, as well as its other stats
(attack, attack speed, crit chance) for weapons.

Abilities, perks and equipment all have a level field. Levels go from 1 to 20. The higher the
level, the stronger the item. The strength of an item is determined by its stats as well as the
modifiers and effects it has, which are also stronger at higher levels. Lower level items usually
have fewer modifiers and effects, while higher level items have more.
