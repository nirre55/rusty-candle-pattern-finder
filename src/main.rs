mod backtest;
mod candle;
mod config;
mod error;
mod fetcher;
mod pattern;

use std::path::PathBuf;

use clap::Parser;

use std::io::Write;

use crate::backtest::{backtest, export_results};
use crate::candle::to_color_sequence;
use crate::config::Config;
use crate::pattern::extract_patterns;

#[derive(Parser)]
#[command(name = "rusty-candle-pattern-finder")]
#[command(about = "Download crypto candles, extract patterns, and backtest them")]
struct Cli {
    /// Path to the TOML config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    println!(
        "Symbol: {} | Interval: {} | Pattern length: {}..{}",
        config.symbol, config.interval, config.pattern_min_length, config.pattern_max_length
    );

    // 1. Fetch candles
    let start_ms = config.start_time_ms()?;
    let end_ms = config.end_time_ms()?;
    let candles =
        fetcher::fetch_all_candles(&config.symbol, &config.interval, start_ms, end_ms).await?;

    // 2. Build color sequence (skip dojis)
    let color_seq = to_color_sequence(&candles);
    let colors: Vec<_> = color_seq.iter().map(|(_, c)| *c).collect();
    println!(
        "Color sequence: {} candles ({} dojis skipped)",
        colors.len(),
        candles.len() - colors.len()
    );

    // 3. Extract all patterns
    let patterns = extract_patterns(&colors, config.pattern_min_length, config.pattern_max_length);
    println!("Unique patterns found: {}", patterns.len());

    // 4. Sort patterns by occurrence (descending) and backtest each
    let mut sorted_patterns: Vec<_> = patterns.into_iter().collect();
    sorted_patterns.sort_by(|a, b| b.1.cmp(&a.1));

    let results: Vec<_> = sorted_patterns
        .iter()
        .filter_map(|(pattern_str, count)| {
            let pat = pattern::parse_pattern(pattern_str)?;
            Some(backtest(&pat, &colors, *count))
        })
        .collect();

    // 5. Export to CSV
    let csv_path = PathBuf::from(format!(
        "results/{}_{}_backtest.csv",
        config.symbol, config.interval
    ));
    export_results(&csv_path, &results)?;
    println!("Backtest results exported to {}", csv_path.display());

    // 6. Export to TXT
    let txt_path = PathBuf::from(format!(
        "results/{}_{}_backtest.txt",
        config.symbol, config.interval
    ));
    let mut txt_file = std::fs::File::create(&txt_path)?;
    writeln!(
        txt_file,
        "Symbol: {} | Interval: {} | Pattern length: {}..{}",
        config.symbol, config.interval, config.pattern_min_length, config.pattern_max_length
    )?;
    writeln!(
        txt_file,
        "Color sequence: {} candles ({} dojis skipped)\n",
        colors.len(),
        candles.len() - colors.len()
    )?;
    writeln!(txt_file, "--- Backtest Results (sorted by occurrence) ---\n")?;
    for result in &results {
        writeln!(txt_file, "Occurrences: {}", result.occurrences)?;
        write!(txt_file, "{result}")?;
    }
    println!("Backtest report exported to {}", txt_path.display());

    Ok(())
}
