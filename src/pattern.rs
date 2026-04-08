use std::collections::HashMap;

use crate::candle::CandleColor;

/// A pattern is a sequence of candle colors, e.g. [V, R] or [V, R, V].
pub type Pattern = Vec<CandleColor>;

/// Formats a pattern as a string, e.g. "VR" or "VRV".
pub fn pattern_to_string(pattern: &[CandleColor]) -> String {
    pattern.iter().map(|c| c.to_string()).collect()
}

/// Parses a string like "VRV" into a pattern.
pub fn parse_pattern(s: &str) -> Option<Pattern> {
    s.chars()
        .map(|c| match c {
            'V' => Some(CandleColor::V),
            'R' => Some(CandleColor::R),
            _ => None,
        })
        .collect()
}

/// Extracts all unique patterns of lengths [min_len..=max_len] from the color sequence.
/// Returns a map of pattern string → occurrence count.
pub fn extract_patterns(
    colors: &[CandleColor],
    min_len: usize,
    max_len: usize,
) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();

    for len in min_len..=max_len {
        for window in colors.windows(len) {
            let key = pattern_to_string(window);
            *counts.entry(key).or_insert(0) += 1;
        }
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_patterns_counts_correctly() {
        let colors = vec![
            CandleColor::V,
            CandleColor::R,
            CandleColor::V,
            CandleColor::R,
            CandleColor::V,
        ];
        let result = extract_patterns(&colors, 2, 3);

        assert_eq!(result.get("VR"), Some(&2));
        assert_eq!(result.get("RV"), Some(&2));
        assert_eq!(result.get("VRV"), Some(&2));
        assert_eq!(result.get("RVR"), Some(&1));
    }

    #[test]
    fn parse_pattern_valid() {
        let p = parse_pattern("VRV");
        assert_eq!(
            p,
            Some(vec![CandleColor::V, CandleColor::R, CandleColor::V])
        );
    }

    #[test]
    fn parse_pattern_invalid_returns_none() {
        assert_eq!(parse_pattern("VXR"), None);
    }
}
