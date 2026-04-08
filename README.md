# Rusty Candle Pattern Finder

A Rust CLI tool that downloads crypto candlestick history from the Binance API, extracts color patterns (green/red), and backtests them.

## How it works

1. **Fetch** - Downloads full candle history from Binance with pagination, caches to CSV for resuming
2. **Classify** - Each candle is labeled V (green: close > open) or R (red: close < open). Dojis (close == open) are skipped
3. **Extract** - Finds all unique patterns of configurable length (e.g. VR, VRV, RRVV...)
4. **Backtest** - Repeats each pattern cyclically over the history and compares predictions to actual candle colors

## Configuration

Edit `config.toml`:

```toml
symbol = "BTCUSDT"
interval = "5m"
pattern_min_length = 2
pattern_max_length = 6

# Optional: restrict date range (format: "YYYY-MM-DD")
# Omit to download the full history
start_date = "2026-01-01"
end_date = "2026-04-01"
```

**Supported intervals**: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M

## Usage

```bash
# Default config (config.toml)
cargo run --release

# Custom config
cargo run --release -- -c my_config.toml
```

## Output

Results are exported to the `results/` folder:

- `BTCUSDT_5m_backtest.csv` - raw data for analysis
- `BTCUSDT_5m_backtest.txt` - human-readable report

## Metrics

| Metric | Description |
|--------|-------------|
| **Candle win rate** | % of candles correctly predicted by cycling the pattern |
| **Pattern windows** | Number of fixed-size windows (total_candles / pattern_length) |
| **Pattern wins/losses** | Windows with at least 1 correct prediction vs none |
| **Max consec. losses** | Longest streak of consecutive wrong candle predictions |
| **Win/Loss ratio** | Total wins / total losses |
| **First match attempts** | Pattern applied sequentially, reset on first match |
| **First match wins/losses** | Attempts where a match was found vs pattern exhausted |
| **First match win rate** | % of attempts that found a match |
| **First match max c.loss** | Longest streak of consecutive exhausted patterns |

### First Match explained

The pattern is walked position by position. On the first correct prediction, the attempt counts as a WIN and the pattern resets. If the entire pattern is traversed without any match, it counts as a LOSS.

## Project Structure

```
src/
  main.rs       - CLI entry point
  config.rs     - TOML config parsing
  fetcher.rs    - Binance API client + CSV cache
  candle.rs     - Candle types and color classification
  pattern.rs    - Pattern extraction
  backtest.rs   - Backtest engine and metrics
  error.rs      - Error types (thiserror)
```

## Requirements

- Rust 1.80+
- Internet connection (Binance API)
