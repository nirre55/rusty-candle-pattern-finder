use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandleColor {
    V, // Verte (green): close > open
    R, // Rouge (red): close < open
}

impl std::fmt::Display for CandleColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V => write!(f, "V"),
            Self::R => write!(f, "R"),
        }
    }
}

impl Candle {
    pub fn color(&self) -> Option<CandleColor> {
        if self.close > self.open {
            Some(CandleColor::V)
        } else if self.close < self.open {
            Some(CandleColor::R)
        } else {
            None // doji — ignored
        }
    }
}

/// Converts candles to a color sequence, skipping dojis.
pub fn to_color_sequence(candles: &[Candle]) -> Vec<(usize, CandleColor)> {
    candles
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.color().map(|color| (i, color)))
        .collect()
}
