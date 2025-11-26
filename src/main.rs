use kryptex::config::Config;
use kryptex::signals::{IndicatorInput, MacdSignal, SignalGenerator};
use kryptex::db::SignalDatabase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let generator = SignalGenerator::new(config);
    let db = SignalDatabase::new("kryptex_signals.db")?;

    let input1 = IndicatorInput {
        macd: MacdSignal {
            macd: 0.5,
            signal: 0.3,
            histogram: 0.2,
        },
        rsi: 25.0,
        funding_rate: -0.0002,
        price: 45000.0,
        symbol: Some("BTC".to_string()),
    };

    let signal1 = generator.generate_signal(&input1);
    println!("Signal 1:");
    print_signal(&signal1);
    db.store_signal(&signal1)?;
    println!();

    let input2 = IndicatorInput {
        macd: MacdSignal {
            macd: -0.3,
            signal: -0.1,
            histogram: -0.2,
        },
        rsi: 75.0,
        funding_rate: 0.0003,
        price: 45500.0,
        symbol: Some("BTC".to_string()),
    };

    let signal2 = generator.generate_signal(&input2);
    println!("Signal 2:");
    print_signal(&signal2);
    db.store_signal(&signal2)?;
    println!();

    let input3 = IndicatorInput {
        macd: MacdSignal {
            macd: 0.1,
            signal: 0.05,
            histogram: 0.05,
        },
        rsi: 50.0,
        funding_rate: 0.00005,
        price: 45200.0,
        symbol: Some("BTC".to_string()),
    };

    let signal3 = generator.generate_signal(&input3);
    println!("Signal 3:");
    print_signal(&signal3);
    db.store_signal(&signal3)?;

    Ok(())
}

fn print_signal(signal: &kryptex::signals::SignalOutput) {
    println!("  Symbol: {}", signal.symbol);
    println!("  Direction: {:?}", signal.direction);
    println!("  Confidence: {:.2}%", signal.confidence * 100.0);
    println!("  Price: ${:.2}", signal.price);
    println!("  Recommended SL: {:.2}%", signal.recommended_sl_pct * 100.0);
    println!("  Recommended TP: {:.2}%", signal.recommended_tp_pct * 100.0);
    println!("  Reasons:");
    for (i, reason) in signal.reasons.iter().enumerate() {
        println!("    {}. {} (weight: {:.2})", i + 1, reason.description, reason.weight);
    }
}
