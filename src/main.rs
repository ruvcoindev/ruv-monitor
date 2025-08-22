use serde::Deserialize;
use reqwest;
use std::error::Error;
use std::time::Duration;
use csv::Writer;
use chrono::{Local, Utc};
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct LiquidityPool {
    id: String,
    reserves: Vec<Reserve>,
    total_trustlines: String,
}

#[derive(Deserialize, Debug)]
struct Reserve {
    asset: String,
    amount: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 ruv-monitor — старт (данные пишутся в ruv_history.csv)");

    let user_public_key = ""; // ← вставь G...
    let mut csv_writer = Writer::from_path("ruv_history.csv")?;
    csv_writer.write_record(&["date", "time", "ruv_rub", "ruv", "xlm", "xlm_usd", "usd_rub", "trustlines"])?;

    let fallback_xlm_usd = 0.45;
    let fallback_usd_rub = 95.0;

    loop {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        // 🔧 Прямой запрос к пулу RUV/XLM
        let pool_url = "https://horizon.stellar.org/liquidity_pools/bfad578200fecbf7210294aa939d68eeb1ea9057c05673f11fedc771089bac00";

        let pool_result = client.get(pool_url).send().await;
        let pool_json: Value = match pool_result {
            Ok(resp) if resp.status().is_success() => {
                match resp.json().await {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("❌ JSON error: {}", e);
                        Value::Null
                    }
                }
            }
            Ok(resp) => {
                eprintln!("❌ HTTP {}: {}", resp.status(), pool_url);
                Value::Null
            }
            Err(e) => {
                eprintln!("❌ Network: {}", e);
                Value::Null
            }
        };

        let mut pools: Vec<LiquidityPool> = Vec::new();

        if pool_json["id"].is_string() {
            if let Ok(pool) = serde_json::from_value(pool_json.clone()) {
                pools.push(pool);
            }
        } else {
            eprintln!("⚠️  Не удалось загрузить пул RUV/XLM");
        }

        // 🌍 Курс XLM/USD
        let xlm_usd: f64 = match client
            .get("https://api.coingecko.com/api/v3/simple/price?ids=stellar&vs_currencies=usd")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let json: Value = resp.json().await.unwrap_or_default();
                json["stellar"]["usd"].as_f64().unwrap_or(fallback_xlm_usd)
            }
            _ => fallback_xlm_usd,
        };

        // 🇷🇺 Курс USD/RUB через cbr-xml-daily.ru
        let usd_rub: f64 = match client
            .get("https://www.cbr-xml-daily.ru/latest.js")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let json: Value = resp.json().await.unwrap_or_default();
                let rate_rub_per_usd = json["rates"]["USD"].as_f64();
                rate_rub_per_usd.and_then(|r| if r > 0.0 { Some(1.0 / r) } else { None }).unwrap_or(fallback_usd_rub)
            }
            _ => fallback_usd_rub,
        };

        let now = Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let time = now.format("%H:%M:%S").to_string();

        println!("\n📊 [{}] Мониторинг RUV/XLM", time);
        println!("──────────────────────────────────────────────────");

        for pool in &pools {
            let mut ruv = 0.0;
            let mut xlm = 0.0;

            for reserve in &pool.reserves {
                let amount = reserve.amount.parse::<f64>().unwrap_or(0.0);
                if reserve.asset.starts_with("RUV:") {
                    ruv = amount;
                } else if reserve.asset == "native" {
                    xlm = amount;
                }
            }

            if ruv > 0.0 && xlm > 0.0 {
                let xlm_per_ruv = xlm / ruv;
                let ruv_rub = xlm_per_ruv * xlm_usd * usd_rub;

                println!("🔄 Пул: {} (RUV/XLM)", &pool.id[..8]);
                println!("   RUV: {:>8.0} | XLM: {:>8.2}", ruv, xlm);
                println!("   Участников: {}", pool.total_trustlines);
                println!("   → 1 RUV = {:.4} RUB", ruv_rub);

                csv_writer.write_record(&[
                    date.clone(),                 // ← Дата
                    time.clone(),                 // ← Время
                    format!("{:.4}", ruv_rub),
                    format!("{:.0}", ruv),
                    format!("{:.2}", xlm),
                    format!("{:.4}", xlm_usd),
                    format!("{:.2}", usd_rub),
                    pool.total_trustlines.clone(),
                ])?;
            }
        }

        // 🧑‍💼 Проверка позиции пользователя
        if !user_public_key.is_empty() {
            let url = format!("https://horizon.stellar.org/accounts/{}/liquidity_positions", user_public_key);
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    let json: Value = resp.json().await.unwrap_or_default();
                    if let Some(positions) = json["_embedded"]["records"].as_array() {
                        println!("\n👤 Ваша доля:");
                        for pos in positions {
                            let pool_id = pos["liquidity_pool"]["id"].as_str().unwrap_or("неизвестен");
                            let shares = pos["liquidity_pool"]["shares"].as_str().unwrap_or("0");
                            println!("   Пул {}: {} шар", &pool_id[..8], shares);
                        }
                    }
                }
            }
        }

        csv_writer.flush()?;
        println!("──────────────────────────────────────────────────");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
