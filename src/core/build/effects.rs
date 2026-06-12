use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Effect {
    /// Enrages your pet, boosting its stats.
    BeastFrenzy {
        attack_pct: f32,
        attack_speed_pct: f32,
        duration: f32,
    },

    /// Increases the attack of the target.
    Berserk {
        attack_pct: f32,
        duration: f32,
    },

    /// Increases the damage of the next basic attack.
    Bleed {
        damage_pct: f32,
    },

    /// Causes the target to have an increased chance to miss basic attacks.
    Blind {
        miss_pct: f32,
        duration: f32,
    },

    /// Deals damage over time.
    Burn {
        damage: u32,
        duration: f32,
    },

    /// Grants a brief window where abilities cost less mana to cast.
    Clearcasting {
        reduction_pct: f32,
        duration: f32,
    },

    /// Deals a percentage of the damage to a second target.
    Cleave {
        damage_pct: f32,
        duration: f32,
    },

    /// Places a mark that deals damage after some time.
    Curse {
        damage: u32,
        timer: u32,
    },

    /// Instant damage
    Pierce {
        damage: u32,
    },

    /// Grants a low percentage chance on hitting to instantly reset all active ability cooldowns.
    EchoStruck {
        reset_pct: f32,
    },

    /// Increases overall damage.
    Empower {
        damage_pct: f32,
        duration: f32,
    },

    /// Increases critical strike chance.
    Focus {
        crit_chance_pct: f32,
        duration: f32,
    },

    /// Increases armor.
    Fortify {
        armor_pct: f32,
        duration: f32,
    },

    /// Reduces attack speed.
    Freeze {
        attack_speed_pct: f32,
        duration: f32,
    },

    /// Increases initiative.
    Haste {
        initiative_pct: f32,
        duration: f32,
    },

    /// Instantly heal a percentage of missing health.
    Heal {
        heal_pct: u32,
    },

    /// Prevents the target from dodging attacks or abilities.
    Immobilize {
        duration: f32,
    },

    /// Restores a flat percentage of health on inflicted damage.
    Lifesteal {
        percentage: f32,
        duration: f32,
    },

    /// Destroys a portion of the target's current mana pool.
    ManaBurn {
        amount: u32,
    },

    /// Recover a flat amount of mana per sec.
    ManaFlow {
        amount: u32,
        duration: f32,
    },

    /// Siphons a percentage of the target's current mana pool into your own.
    Manasteal {
        percentage: f32,
    },

    /// Grants a brief window of total invulnerability where the target cannot take damage or be debuffed.
    /// The target cannot cast any ability nor attack in this period.
    MonarchShield {
        duration: f32,
    },

    /// Deals damage over time.
    Poison {
        damage: u32,
        duration: f32,
    },

    /// Reduces initiative.
    Paranoia {
        initiative_pct: f32,
        duration: f32,
    },

    /// Removes all negative effects on the target.
    Purge,

    /// Recover a flat amount of health per sec.
    Regen {
        heal: u32,
        duration: f32,
    },

    /// Preventing the target from casting any abilities.
    Silence {
        duration: f32,
    },

    /// Redirects a portion of all incoming damage taken by the player directly into their pet.
    SoulLink {
        damage_transfer_pct: f32,
        duration: f32,
    },

    /// Freezes the target's ability cooldown recovery.
    Stun {
        duration: f32,
    },

    /// Forces enemies to prioritize attacking the target's pet instead of the player.
    Taunt {
        duration: f32,
    },

    /// Spikes out thorny roots that reflect a flat amount of physical damage back to anyone who strikes you.
    Thorns {
        damage_reflected_pct: f32,
        duration: f32,
    },

    /// Amplifies all incoming damage on the target.
    Vulnerability {
        damage_pct: f32,
        duration: f32,
    },
}

impl Effect {
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        match self {
            Self::BeastFrenzy { attack_pct, attack_speed_pct, duration } => {
                match language {
                    Language::Spanish => format!("Frenesí de bestia: +{attack_pct:.0}% de ataque de mascota y +{attack_speed_pct:.0}% de velocidad de ataque de mascota durante {duration:.1}s"),
                    Language::Dutch => format!("Beestachtige Razernij: +{attack_pct:.0}% huisdier aanval en +{attack_speed_pct:.0}% huisdier aanvalsnelheid gedurende {duration:.1}s"),
                    Language::English => format!("Beast Frenzy: +{attack_pct:.0}% pet attack and +{attack_speed_pct:.0}% pet attack speed for {duration:.1}s"),
                }
            },
            Self::Berserk { attack_pct, duration } => {
                match language {
                    Language::Spanish => format!("Furia: +{attack_pct:.0}% de ataque durante {duration:.1}s"),
                    Language::Dutch => format!("Berserk: +{attack_pct:.0}% aanval gedurende {duration:.1}s"),
                    Language::English => format!("Berserk: +{attack_pct:.0}% attack for {duration:.1}s"),
                }
            },
            Self::Bleed { damage_pct } => {
                match language {
                    Language::Spanish => format!("Sangrado: +{damage_pct:.0}% de daño de ataque básico como daño a lo largo del tiempo"),
                    Language::Dutch => format!("Bloeden: +{damage_pct:.0}% basisaanvalschade als schade over tijd"),
                    Language::English => format!("Bleed: +{damage_pct:.0}% basic attack damage as damage over time"),
                }
            },
            Self::Blind { miss_pct, duration } => {
                match language {
                    Language::Spanish => format!("Ceguera: +{miss_pct:.0}% de probabilidad de fallar ataques básicos durante {duration:.1}s"),
                    Language::Dutch => format!("Blindheid: +{miss_pct:.0}% kans om basisaanvallen te missen gedurende {duration:.1}s"),
                    Language::English => format!("Blind: +{miss_pct:.0}% chance to miss basic attacks for {duration:.1}s"),
                }
            },
            Self::Burn { damage, duration } => {
                match language {
                    Language::Spanish => format!("Quemadura: +{damage} de daño/s durante {duration:.1}s"),
                    Language::Dutch => format!("Brand: +{damage} schade/s gedurende {duration:.1}s"),
                    Language::English => format!("Burn: +{damage} damage/s for {duration:.1}s"),
                }
            },
            Self::Clearcasting { reduction_pct, duration } => {
                match language {
                    Language::Spanish => format!("Lanzamiento libre: -{reduction_pct:.0}% de coste de maná de habilidades durante {duration:.1}s"),
                    Language::Dutch => format!("Helder Gieten: -{reduction_pct:.0}% manakosten van vaardigheden gedurende {duration:.1}s"),
                    Language::English => format!("Clearcasting: -{reduction_pct:.0}% mana costs of abilities for {duration:.1}s"),
                }
            },
            Self::Cleave { damage_pct, duration } => {
                match language {
                    Language::Spanish => format!("Hendedura: Inflige {damage_pct:.0}% de daño al segundo objetivo durante {duration:.1}s"),
                    Language::Dutch => format!("Klieven: Richt {damage_pct:.0}% schade toe aan een tweede doelwit gedurende {duration:.1}s"),
                    Language::English => format!("Cleave: Deals {damage_pct:.0}% damage to second target for {duration:.1}s"),
                }
            },
            Self::Curse { damage, timer } => {
                match language {
                    Language::Spanish => format!("Maldición: Inflige {damage} de daño después de {timer}s"),
                    Language::Dutch => format!("Vloek: Richt {damage} schade toe na {timer}s"),
                    Language::English => format!("Curse: Deals {damage} damage after {timer}s"),
                }
            },
            Self::Pierce { damage } => {
                match language {
                    Language::Spanish => format!("Perforación: Inflige instantáneamente {damage} de daño de perforación"),
                    Language::Dutch => format!("Doorboring: Richt direct {damage} doordringende schade toe"),
                    Language::English => format!("Pierce: Instantly deals {damage} piercing damage"),
                }
            },
            Self::EchoStruck { reset_pct } => {
                match language {
                    Language::Spanish => format!("Golpe de Eco: +{reset_pct:.0}% de probabilidad de reiniciar enfriamientos al golpear"),
                    Language::Dutch => format!("Echo-inslag: +{reset_pct:.0}% kans om alle afkoeltijden te resetten bij een treffer"),
                    Language::English => format!("Echo Struck: +{reset_pct:.0}% chance to reset all cooldowns on hit"),
                }
            },
            Self::Empower { damage_pct, duration } => {
                match language {
                    Language::Spanish => format!("Potenciar: +{damage_pct:.0}% de daño general durante {duration:.1}s"),
                    Language::Dutch => format!("Versterken: +{damage_pct:.0}% totale schade gedurende {duration:.1}s"),
                    Language::English => format!("Empower: +{damage_pct:.0}% overall damage for {duration:.1}s"),
                }
            },
            Self::Focus { crit_chance_pct, duration } => {
                match language {
                    Language::Spanish => format!("Enfoque: +{crit_chance_pct:.0}% de probabilidad de golpe crítico durante {duration:.1}s"),
                    Language::Dutch => format!("Focus: +{crit_chance_pct:.0}% kritieke slag kans gedurende {duration:.1}s"),
                    Language::English => format!("Focus: +{crit_chance_pct:.0}% critical strike chance for {duration:.1}s"),
                }
            },
            Self::Fortify { armor_pct, duration } => {
                match language {
                    Language::Spanish => format!("Fortificar: +{armor_pct:.0}% de armadura durante {duration:.1}s"),
                    Language::Dutch => format!("Versterken: +{armor_pct:.0}% bepantsering gedurende {duration:.1}s"),
                    Language::English => format!("Fortify: +{armor_pct:.0}% armor for {duration:.1}s"),
                }
            },
            Self::Freeze { attack_speed_pct, duration } => {
                match language {
                    Language::Spanish => format!("Congelación: -{attack_speed_pct:.0}% de velocidad de ataque durante {duration:.1}s"),
                    Language::Dutch => format!("Bevriezen: -{attack_speed_pct:.0}% aanvalsnelheid gedurende {duration:.1}s"),
                    Language::English => format!("Freeze: -{attack_speed_pct:.0}% attack speed for {duration:.1}s"),
                }
            },
            Self::Haste { initiative_pct, duration } => {
                match language {
                    Language::Spanish => format!("Prisa: +{initiative_pct:.0}% de iniciativa durante {duration:.1}s"),
                    Language::Dutch => format!("Haast: +{initiative_pct:.0}% initiatief gedurende {duration:.1}s"),
                    Language::English => format!("Haste: +{initiative_pct:.0}% initiative for {duration:.1}s"),
                }
            },
            Self::Heal { heal_pct } => {
                match language {
                    Language::Spanish => format!("Curación: Cura instantáneamente {heal_pct}% de la salud faltante"),
                    Language::Dutch => format!("Genezing: Geneest direct {heal_pct}% ontbrekende gezondheid"),
                    Language::English => format!("Heal: Instantly heals {heal_pct}% missing health"),
                }
            },
            Self::Immobilize { duration } => {
                match language {
                    Language::Spanish => format!("Inmovilizar: Evita que esquive durante {duration:.1}s"),
                    Language::Dutch => format!("Immobiliseren: Voorkomt ontwijken gedurende {duration:.1}s"),
                    Language::English => format!("Immobilize: Prevents dodging for {duration:.1}s"),
                }
            },
            Self::Lifesteal { percentage, duration } => {
                match language {
                    Language::Spanish => format!("Robo de vida: Restaura {percentage:.0}% del daño infligido como salud durante {duration:.1}s"),
                    Language::Dutch => format!("Levensroof: Herstelt {percentage:.0}% van de toegebrachte schade als gezondheid gedurende {duration:.1}s"),
                    Language::English => format!("Lifesteal: Restores {percentage:.0}% of damage dealt as health for {duration:.1}s"),
                }
            },
            Self::ManaBurn { amount } => {
                match language {
                    Language::Spanish => format!("Quemadura de maná: Quema instantáneamente {amount} de maná"),
                    Language::Dutch => format!("Manabrand: Brandt direct {amount} mana"),
                    Language::English => format!("Mana Burn: Instantly burns {amount} mana"),
                }
            },
            Self::ManaFlow { amount, duration } => {
                match language {
                    Language::Spanish => format!("Flujo de maná: +{amount} de maná/s durante {duration:.1}s"),
                    Language::Dutch => format!("Manastroom: +{amount} mana/s gedurende {duration:.1}s"),
                    Language::English => format!("Mana Flow: +{amount} mana/s for {duration:.1}s"),
                }
            },
            Self::Manasteal { percentage } => {
                match language {
                    Language::Spanish => format!("Robo de maná: Roba {percentage:.0}% del maná del objetivo"),
                    Language::Dutch => format!("Manaroof: Steelt {percentage:.0}% van de mana van het doelwit"),
                    Language::English => format!("Manasteal: Steals {percentage:.0}% of target's mana"),
                }
            },
            Self::MonarchShield { duration } => {
                match language {
                    Language::Spanish => format!("Escudo Monarca: Invulnerabilidad total durante {duration:.1}s. No se puede lanzar habilidades ni atacar durante este tiempo."),
                    Language::Dutch => format!("Monarch Schild: Totale onkwetsbaarheid gedurende {duration:.1}s. Kan in deze periode geen vaardigheden gebruiken of aanvallen."),
                    Language::English => format!("Monarch Shield: Total invulnerability for {duration:.1}s. Cannot cast nor attack during this time."),
                }
            },
            Self::Poison { damage, duration } => {
                match language {
                    Language::Spanish => format!("Veneno: +{damage} de daño/s durante {duration:.1}s"),
                    Language::Dutch => format!("Vergif: +{damage} schade/s gedurende {duration:.1}s"),
                    Language::English => format!("Poison: +{damage} damage/s for {duration:.1}s"),
                }
            },
            Self::Paranoia { initiative_pct, duration } => {
                match language {
                    Language::Spanish => format!("Paranoia: -{initiative_pct:.0}% de iniciativa durante {duration:.1}s"),
                    Language::Dutch => format!("Paranoia: -{initiative_pct:.0}% initiatief gedurende {duration:.1}s"),
                    Language::English => format!("Paranoia: -{initiative_pct:.0}% initiative for {duration:.1}s"),
                }
            },
            Self::Purge => {
                match language {
                    Language::Spanish => "Purga: Elimina todos los efectos negativos".to_string(),
                    Language::Dutch => "Zuiveren: Verwijdert alle negatieve effecten".to_string(),
                    Language::English => "Purge: Removes all negative effects".to_string(),
                }
            },
            Self::Regen { heal, duration } => {
                match language {
                    Language::Spanish => format!("Regeneración: +{heal} de salud/s durante {duration:.1}s"),
                    Language::Dutch => format!("Regeneratie: +{heal} gezondheid/s gedurende {duration:.1}s"),
                    Language::English => format!("Regen: +{heal} health/s for {duration:.1}s"),
                }
            },
            Self::Silence { duration } => {
                match language {
                    Language::Spanish => format!("Silencio: Evita lanzar habilidades durante {duration:.1}s"),
                    Language::Dutch => format!("Stilte: Voorkomt het gebruiken van vaardigheden gedurende {duration:.1}s"),
                    Language::English => format!("Silence: Prevents casting skills for {duration:.1}s"),
                }
            },
            Self::SoulLink { damage_transfer_pct, duration } => {
                match language {
                    Language::Spanish => format!("Enlace de alma: Redirige {damage_transfer_pct:.0}% del daño a la mascota durante {duration:.1}s"),
                    Language::Dutch => format!("Zielenband: Stuurt {damage_transfer_pct:.0}% van de schade door naar het huisdier gedurende {duration:.1}s"),
                    Language::English => format!("Soul Link: Redirects {damage_transfer_pct:.0}% of damage to pet for {duration:.1}s"),
                }
            },
            Self::Stun { duration } => {
                match language {
                    Language::Spanish => format!("Aturdimiento: Congela la recuperación de enfriamientos durante {duration:.1}s"),
                    Language::Dutch => format!("Verdoving: Bevriest het herstel van afkoeltijden gedurende {duration:.1}s"),
                    Language::English => format!("Stun: Freezes cooldown recovery for {duration:.1}s"),
                }
            },
            Self::Taunt { duration } => {
                match language {
                    Language::Spanish => format!("Provocación: Obliga a los enemigos a atacar a la mascota durante {duration:.1}s"),
                    Language::Dutch => format!("Provoceren: Dwingt vijanden om het huisdier aan te vallen gedurende {duration:.1}s"),
                    Language::English => format!("Taunt: Forces enemies to attack pet for {duration:.1}s"),
                }
            },
            Self::Thorns { damage_reflected_pct, duration } => {
                match language {
                    Language::Spanish => format!("Espinas: Refleja {damage_reflected_pct:.0}% del daño recibido como daño físico durante {duration:.1}s"),
                    Language::Dutch => format!("Doornen: Reflecteert {damage_reflected_pct:.0}% van de inkomende schade als fysieke schade gedurende {duration:.1}s"),
                    Language::English => format!("Thorns: Reflects {damage_reflected_pct:.0}% of incoming damage as physical damage for {duration:.1}s"),
                }
            },
            Self::Vulnerability { damage_pct, duration } => {
                match language {
                    Language::Spanish => format!("Vulnerabilidad: Amplifica el daño recibido en el objetivo un {damage_pct:.0}% durante {duration:.1}s"),
                    Language::Dutch => format!("Kwetsbaarheid: Versterkt inkomende schade op het doelwit met {damage_pct:.0}% gedurende {duration:.1}s"),
                    Language::English => format!("Vulnerability: Amplifies incoming damage on the target by {damage_pct:.0}% for {duration:.1}s"),
                }
            },
        }
    }
}
