//! Deepwoken MAGIC NUMBERS

pub const MAX_TOTAL: i64 = 330;

pub const STAT_CAP: i64 = 100;

pub const MAX_LEVEL: u32 = 20;

pub const POINTS_PER_LEVEL: i64 = 15;

pub const TRAIT_CAP: i64 = 6;

pub const TRAIT_TOTAL_CAP: i64 = 12;

/* ---------- starting values ---------- */

/// Starting stat values (see wiki)
/// TODO! use a static/lazy initiliazed map or smth,
/// but tbh at this scale a linear search is probably faster 😹😹😹😹😹😹😹😹
pub const STARTING_FLAT: &[(&str, f64)] = &[
    ("Health", 220.0),
    ("Posture", 20.0),
    ("Ether", 120.0),
    ("Tempo", 120.0),
    ("Sanity", 80.0),
    ("Carry Load", 100.0),
];

/* ---------- requirements ---------- */

/// Khan lowers every equipment and weapon stat requirement (aside from power req).
pub const KHAN_REQ_REDUCTION: i64 = 3;

/// Silentheart lowers weapon-category requirements (not core attributes)
pub const SILENTHEART_REQ_REDUCTION: i64 = 25;

/// The talent an oath is represented by, since oaths live in the talent list
pub const SILENTHEART: &str = "Oath: Silentheart";

/// Failing a weapon's requirements costs at most this percent of damage
pub const REQUIREMENT_PENALTY: f64 = 0.25;

/* ---------- weapon damage ---------- */

/// Stat scaling contributes `SCALING_FACTOR * \sum(stat * coeff) / SCALING_DIVISOR`.
pub const SCALING_FACTOR: f64 = 0.75;
pub const SCALING_DIVISOR: f64 = 1000.0;

/// A rank-k scaling ring contributes `RING_FACTOR * investment / (2^(k-1) * 1000)`.
pub const RING_FACTOR: f64 = 1.2;

/// Proficiency raises the weapon scaling term
pub const PROFICIENCY_PER_POINT: f64 = 0.065;

/// Songchant raises the mantra scaling term
pub const SONGCHANT_PER_POINT: f64 = 0.075;

/// Proficiency and Songchant each add this much pen per point
pub const PEN_PER_TRAIT_POINT: f64 = 2.5;

/// A weapon whose damage types include Bleed, bleeds for this percent of scaled damage
pub const INNATE_BLEED_RATE: f64 = 0.15;

/* ---------- per attribute stuff ---------- */

pub const PEN_PER_STRENGTH: f64 = 0.1;

pub const HEALTH_PER_LEVEL: f64 = 4.0;

/// Fortitude gives half a point of health per point, halving again past the knee
pub const HEALTH_PER_FORTITUDE: f64 = 0.5;
pub const HEALTH_PER_FORTITUDE_PAST_KNEE: f64 = 0.25;
pub const FORTITUDE_HEALTH_KNEE: i64 = 50;

pub const HEALTH_PER_VITALITY: f64 = 10.0;

pub const ETHER_PER_INTELLIGENCE: f64 = 2.0;
pub const ETHER_PER_CHARISMA: f64 = 1.5;
pub const ETHER_PER_ERUDITION: f64 = 25.0;

pub const SANITY_PER_WILLPOWER: f64 = 3.0;

pub const TEMPO_PER_WILLPOWER: f64 = 0.5;
pub const TEMPO_PER_ERUDITION: f64 = 5.0;
pub const TEMPO_GAIN_PER_ERUDITION: f64 = 5.0;

pub const STEALTH_PER_AGILITY: f64 = 0.2;

pub const CARRY_LOAD_PER_STRENGTH: f64 = 0.5;
pub const CARRY_LOAD_PER_FORTITUDE: f64 = 0.5;
/// Neither Strength nor Fortitude can contribute more carry load than this, which they
/// hit at 100 investment
pub const CARRY_LOAD_PER_STAT_CAP: f64 = 50.0;

/* ---------- boons, flaws & aspects ---------- */

pub const PACKMULE_CARRY_LOAD: f64 = 50.0;

/// The fully upgraded Carrying Capacity echo unlock. Account progression rather than a
/// build property, so it is assumed to be maxed.
pub const ECHO_CARRY_LOAD: f64 = 30.0;

/// Ganymede multiplies sanity gained above the base
pub const GANYMEDE_SANITY_MULT: f64 = 1.2;

/// Felinor multiplies stealth, then adds its own flat amount after
pub const FELINOR_STEALTH_MULT: f64 = 1.2;
pub const FELINOR_STEALTH_FLAT: f64 = 20.0;

/* ---------- caps ---------- */

/// Penetration ceiling, and the raised ceiling once a cap breaker is held.
pub const PEN_CAP: f64 = 0.5;
pub const PEN_CAP_LIFTED: f64 = 1.0;

/// Talents that lift the penetration ceiling rather than just adding to it.
pub const PEN_CAP_BREAKERS: &[&str] = &["Million Ton Piercer", "Ether Overdrive"];

/// Damage modifier caps as `(soft, hard)` fractions.
///
/// Entering any combat drops the soft cap. Only player combat drops the hard cap.
pub const DAMAGE_CAPS_OUT_OF_COMBAT: (f64, f64) = (0.50, 0.75);
pub const DAMAGE_CAPS_PVE: (f64, f64) = (0.25, 0.75);
pub const DAMAGE_CAPS_PVP: (f64, f64) = (0.25, 0.50);

/// A resistance from any one source is clamped here before it is applied
pub const MAX_SINGLE_RESIST: f64 = 99.0;

/* ---------- shrines ---------- */

pub const SHRINE_ORDER_MAX_LOSS: f64 = 25.0;

pub const SHRINE_MASTERY_LIMIT: i64 = 3;
