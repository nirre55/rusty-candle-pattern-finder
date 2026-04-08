use std::path::{Path, PathBuf};

use crate::candle::Candle;
use crate::error::{AppError, Result};

const BINANCE_API: &str = "https://api.binance.com/api/v3/klines";
const BATCH_LIMIT: u16 = 1000;

pub fn csv_path(symbol: &str, interval: &str, start: Option<i64>, end: Option<i64>) -> PathBuf {
    let start_tag = start.map_or_else(|| "all".to_owned(), |s| s.to_string());
    let end_tag = end.map_or_else(|| "now".to_owned(), |e| e.to_string());
    PathBuf::from(format!("data/{symbol}_{interval}_{start_tag}_{end_tag}.csv"))
}

pub async fn fetch_all_candles(
    symbol: &str,
    interval: &str,
    config_start: Option<i64>,
    config_end: Option<i64>,
) -> Result<Vec<Candle>> {
    let path = csv_path(symbol, interval, config_start, config_end);

    // Resume from last candle if CSV exists
    let resumed_start = load_last_close_time(&path).map(|t| {
        println!("Resuming from close_time {t}");
        t + 1
    });

    let client = reqwest::Client::new();
    let mut all_candles = if resumed_start.is_some() {
        load_from_csv(&path)?
    } else {
        Vec::new()
    };

    // Use resumed position if available, otherwise config start_date
    let mut current_start = resumed_start.or(config_start);

    loop {
        let mut url = format!(
            "{BINANCE_API}?symbol={symbol}&interval={interval}&limit={BATCH_LIMIT}"
        );
        if let Some(start) = current_start {
            url.push_str(&format!("&startTime={start}"));
        }
        if let Some(end) = config_end {
            url.push_str(&format!("&endTime={end}"));
        }

        println!("Fetching {symbol} {interval} (start={current_start:?})...");

        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(AppError::Binance(body));
        }

        let data: Vec<Vec<serde_json::Value>> = resp.json().await?;
        if data.is_empty() {
            break;
        }

        let batch: Vec<Candle> = data
            .iter()
            .map(|row| parse_kline(row))
            .collect::<Result<Vec<_>>>()?;

        let batch_len = batch.len();

        let last_close_time = batch
            .last()
            .map(|c| c.close_time)
            .unwrap_or(0);

        all_candles.extend(batch);

        if batch_len < BATCH_LIMIT as usize {
            break;
        }

        current_start = Some(last_close_time + 1);
    }

    save_to_csv(&path, &all_candles)?;
    println!(
        "Total candles: {} — saved to {}",
        all_candles.len(),
        path.display()
    );

    Ok(all_candles)
}

fn parse_kline(row: &[serde_json::Value]) -> Result<Candle> {
    let parse_f64 = |v: &serde_json::Value| -> Result<f64> {
        v.as_str()
            .ok_or_else(|| AppError::Binance("Expected string field".to_owned()))?
            .parse::<f64>()
            .map_err(|e| AppError::Binance(format!("Float parse error: {e}")))
    };

    Ok(Candle {
        open_time: row[0]
            .as_i64()
            .ok_or_else(|| AppError::Binance("Missing open_time".to_owned()))?,
        open: parse_f64(&row[1])?,
        high: parse_f64(&row[2])?,
        low: parse_f64(&row[3])?,
        close: parse_f64(&row[4])?,
        volume: parse_f64(&row[5])?,
        close_time: row[6]
            .as_i64()
            .ok_or_else(|| AppError::Binance("Missing close_time".to_owned()))?,
    })
}

fn load_last_close_time(path: &Path) -> Option<i64> {
    if !path.exists() {
        return None;
    }
    let mut reader = csv::Reader::from_path(path).ok()?;
    let mut last: Option<i64> = None;
    for candle in reader.deserialize::<Candle>().flatten() {
        last = Some(candle.close_time);
    }
    last
}

fn load_from_csv(path: &Path) -> Result<Vec<Candle>> {
    let mut reader = csv::Reader::from_path(path)?;
    let candles: Vec<Candle> = reader
        .deserialize()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(candles)
}

fn save_to_csv(path: &Path, candles: &[Candle]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    for candle in candles {
        writer.serialize(candle)?;
    }
    writer.flush()?;
    Ok(())
}
