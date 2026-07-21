pub mod aggregate;
pub mod algos;
pub mod pips;
pub mod reqtree;
pub mod statmap;
pub mod traits;

#[cfg(feature = "fetch")]
pub mod datafetch;

pub mod graph;

/// Transforms the name of things in-game into an identifier/key for the `DeepData` maps
#[must_use]
pub fn name_to_identifier(s: &str) -> String {
    s.replace(": ", " ")
        .replace(' ', "_")
        .replace(['[', ']', '\'', '(', ')', ','], "")
        .replace(['-'], "_")
        .to_lowercase()
}
