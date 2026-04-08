use std::path::Path;

use serde::Serialize;

use crate::candle::CandleColor;
use crate::error::Result;
use crate::pattern::pattern_to_string;

/// Result of backtesting a single pattern.
#[derive(Debug, Serialize)]
pub struct BacktestResult {
    pub pattern: String,
    pub occurrences: u32,
    pub total_candles: usize,
    pub wins: usize,
    pub losses: usize,
    pub candle_win_rate: f64,
    pub pattern_windows: usize,
    pub pattern_wins: usize,
    pub pattern_losses: usize,
    pub pattern_win_rate: f64,
    pub max_consecutive_losses: usize,
    pub win_loss_ratio: f64,
    pub first_match_attempts: usize,
    pub first_match_wins: usize,
    pub first_match_losses: usize,
    pub first_match_win_rate: f64,
    pub first_match_max_consec_losses: usize,
    pub first_match_max_consec_losses_count: usize,
}

impl std::fmt::Display for BacktestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Pattern: {} ===", self.pattern)?;
        writeln!(f, "  Total candles tested : {}", self.total_candles)?;
        writeln!(f, "  Wins                 : {}", self.wins)?;
        writeln!(f, "  Losses               : {}", self.losses)?;
        writeln!(f, "  Candle win rate       : {:.2}%", self.candle_win_rate * 100.0)?;
        writeln!(f, "  Pattern windows       : {}", self.pattern_windows)?;
        writeln!(f, "  Pattern wins          : {}", self.pattern_wins)?;
        writeln!(f, "  Pattern losses        : {}", self.pattern_losses)?;
        writeln!(f, "  Pattern win rate      : {:.2}%", self.pattern_win_rate * 100.0)?;
        writeln!(f, "  Max consec. losses    : {}", self.max_consecutive_losses)?;
        writeln!(f, "  Win/Loss ratio        : {:.2}", self.win_loss_ratio)?;
        writeln!(f, "  First match attempts  : {}", self.first_match_attempts)?;
        writeln!(f, "  First match wins      : {}", self.first_match_wins)?;
        writeln!(f, "  First match losses    : {}", self.first_match_losses)?;
        writeln!(f, "  First match win rate  : {:.2}%", self.first_match_win_rate * 100.0)?;
        writeln!(f, "  First match max c.loss: {}", self.first_match_max_consec_losses)?;
        writeln!(f, "  First match max c.loss count: {}", self.first_match_max_consec_losses_count)?;
        Ok(())
    }
}

/// Backtests a pattern against a color sequence.
///
/// The pattern is repeated cyclically over the entire sequence as prediction.
/// Each candle is compared: prediction == actual → WIN, else → LOSS.
///
/// Pattern win rate: a "window" of pattern_len candles is a WIN if at least
/// one candle in that window was correctly predicted.
pub fn backtest(pattern: &[CandleColor], actual: &[CandleColor], occurrences: u32) -> BacktestResult {
    let pattern_len = pattern.len();
    let total = actual.len();

    let mut wins: usize = 0;
    let mut losses: usize = 0;
    let mut consecutive_losses: usize = 0;
    let mut max_consecutive_losses: usize = 0;

    // Per-candle comparison
    let results: Vec<bool> = actual
        .iter()
        .enumerate()
        .map(|(i, actual_color)| {
            let predicted = &pattern[i % pattern_len];
            predicted == actual_color
        })
        .collect();

    for &is_win in &results {
        if is_win {
            wins += 1;
            consecutive_losses = 0;
        } else {
            losses += 1;
            consecutive_losses += 1;
            if consecutive_losses > max_consecutive_losses {
                max_consecutive_losses = consecutive_losses;
            }
        }
    }

    // Pattern window win rate
    let pattern_windows = total / pattern_len;
    let mut pattern_wins: usize = 0;
    for chunk in results.chunks(pattern_len) {
        if chunk.iter().any(|&w| w) {
            pattern_wins += 1;
        }
    }
    let pattern_losses = pattern_windows - pattern_wins;

    let candle_win_rate = if total > 0 {
        wins as f64 / total as f64
    } else {
        0.0
    };
    let pattern_win_rate = if pattern_windows > 0 {
        pattern_wins as f64 / pattern_windows as f64
    } else {
        0.0
    };
    let win_loss_ratio = if losses > 0 {
        wins as f64 / losses as f64
    } else {
        f64::INFINITY
    };

    // First-match: walk through the pattern sequentially, reset on first match.
    // WIN = found a match before exhausting the pattern, LOSS = no match in full pattern.
    let mut first_match_attempts: usize = 0;
    let mut first_match_wins: usize = 0;
    let mut first_match_consec_losses: usize = 0;
    let mut first_match_max_consec_losses: usize = 0;
    let mut first_match_max_consec_losses_count: usize = 0;
    let mut pattern_pos: usize = 0;
    for actual_color in actual {
        let predicted = &pattern[pattern_pos];
        if predicted == actual_color {
            first_match_attempts += 1;
            first_match_wins += 1;
            first_match_consec_losses = 0;
            pattern_pos = 0;
        } else {
            pattern_pos += 1;
            if pattern_pos >= pattern_len {
                first_match_attempts += 1;
                first_match_consec_losses += 1;
                if first_match_consec_losses > first_match_max_consec_losses {
                    first_match_max_consec_losses = first_match_consec_losses;
                    first_match_max_consec_losses_count = 1;
                } else if first_match_consec_losses == first_match_max_consec_losses {
                    first_match_max_consec_losses_count += 1;
                }
                pattern_pos = 0;
            }
        }
    }

    let first_match_win_rate = if first_match_attempts > 0 {
        first_match_wins as f64 / first_match_attempts as f64
    } else {
        0.0
    };

    BacktestResult {
        pattern: pattern_to_string(pattern),
        occurrences,
        total_candles: total,
        wins,
        losses,
        candle_win_rate,
        pattern_windows,
        pattern_wins,
        pattern_losses,
        pattern_win_rate,
        max_consecutive_losses,
        win_loss_ratio,
        first_match_attempts,
        first_match_wins,
        first_match_losses: first_match_attempts - first_match_wins,
        first_match_win_rate,
        first_match_max_consec_losses,
        first_match_max_consec_losses_count,
    }
}

/// Exports backtest results to a CSV file.
pub fn export_results(path: &Path, results: &[BacktestResult]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    for result in results {
        writer.serialize(result)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_perfect_match() {
        let pattern = vec![CandleColor::V, CandleColor::R];
        let actual = vec![
            CandleColor::V,
            CandleColor::R,
            CandleColor::V,
            CandleColor::R,
        ];
        let result = backtest(&pattern, &actual, 0);
        assert_eq!(result.wins, 4);
        assert_eq!(result.losses, 0);
        assert!((result.candle_win_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn backtest_no_match() {
        let pattern = vec![CandleColor::V, CandleColor::R];
        let actual = vec![
            CandleColor::R,
            CandleColor::V,
            CandleColor::R,
            CandleColor::V,
        ];
        let result = backtest(&pattern, &actual, 0);
        assert_eq!(result.wins, 0);
        assert_eq!(result.losses, 4);
        assert_eq!(result.max_consecutive_losses, 4);
    }

    #[test]
    fn backtest_partial_match() {
        let pattern = vec![CandleColor::V, CandleColor::R];
        let actual = vec![
            CandleColor::V,
            CandleColor::V,
            CandleColor::V,
            CandleColor::R,
        ];
        let result = backtest(&pattern, &actual, 0);
        assert_eq!(result.wins, 3);  // V==V, V==V, R==R
        assert_eq!(result.losses, 1); // R!=V
        assert_eq!(result.pattern_wins, 2); // each window has at least 1 win
    }

    #[test]
    fn first_match_resets_on_win() {
        // Pattern VRV, Market RRVRVV
        let pattern = vec![CandleColor::V, CandleColor::R, CandleColor::V];
        let actual = vec![
            CandleColor::R, // predict V → L, advance to pos 1
            CandleColor::R, // predict R → W → attempt 1 WIN, reset
            CandleColor::V, // predict V → W → attempt 2 WIN, reset
            CandleColor::R, // predict V → L, advance to pos 1
            CandleColor::V, // predict R → L, advance to pos 2
            CandleColor::V, // predict V → W → attempt 3 WIN, reset
        ];
        let result = backtest(&pattern, &actual, 0);
        assert_eq!(result.first_match_attempts, 3);
        assert_eq!(result.first_match_wins, 3);
    }

    #[test]
    fn first_match_counts_loss_when_pattern_exhausted() {
        // Pattern VR, Market RV
        // pos 0: predict V, actual R → L, advance to pos 1
        // pos 1: predict R, actual V → L, pattern exhausted → attempt 1 LOSS
        let pattern = vec![CandleColor::V, CandleColor::R];
        let actual = vec![CandleColor::R, CandleColor::V];
        let result = backtest(&pattern, &actual, 0);
        assert_eq!(result.first_match_attempts, 1);
        assert_eq!(result.first_match_wins, 0);
    }
}
