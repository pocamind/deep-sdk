pub mod reqtree;
pub mod statmap;
pub mod traits;

#[cfg(feature = "fetch")]
pub mod datafetch;

/// Transforms the name of things in-game into an identifier/key for the DeepData maps
pub fn name_to_identifier(s: &str) -> String {
    s.replace(' ', "_")
        .replace(['[', ']', '\'', ':', '(', ')', ','], "")
        .replace(['-'], "_")
}