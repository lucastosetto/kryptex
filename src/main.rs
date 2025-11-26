use kryptex::config::Config;
use kryptex::signals::{IndicatorInput, MacdSignal, SignalGenerator};
use kryptex::db::SignalDatabase;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut symbol = None;
    let mut macd = None;
    let mut signal_line = None;
    let mut histogram = None;
    let mut rsi = None;
    let mut funding_rate = None;
    let mut price = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--symbol" | "-s" => {
                if i + 1 < args.len() {
                    symbol = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --symbol requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--macd" | "-m" => {
                if i + 1 < args.len() {
                    macd = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --macd requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--signal" => {
                if i + 1 < args.len() {
                    signal_line = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --signal requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--histogram" => {
                if i + 1 < args.len() {
                    histogram = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --histogram requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--rsi" | "-r" => {
                if i + 1 < args.len() {
                    rsi = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --rsi requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--funding-rate" | "--funding_rate" | "-f" => {
                if i + 1 < args.len() {
                    funding_rate = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --funding-rate requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--price" | "-p" => {
                if i + 1 < args.len() {
                    price = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --price requires a value");
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                print_usage(&args[0]);
                std::process::exit(1);
            }
        }
    }

    if symbol.is_none() || macd.is_none() || signal_line.is_none() || histogram.is_none() 
        || rsi.is_none() || price.is_none() {
        eprintln!("Error: Missing required arguments");
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let config = Config::default();
    let generator = SignalGenerator::new(config);
    let db = SignalDatabase::new("kryptex_signals.db")?;

    let input = IndicatorInput {
        macd: MacdSignal {
            macd: macd.unwrap(),
            signal: signal_line.unwrap(),
            histogram: histogram.unwrap(),
        },
        rsi: rsi.unwrap(),
        funding_rate,
        price: price.unwrap(),
        symbol: Some(symbol.unwrap()),
    };

    let signal = generator.generate_signal(&input);
    print_signal(&signal);
    db.store_signal(&signal)?;

    Ok(())
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} [OPTIONS]", program);
    eprintln!("\nOptions:");
    eprintln!("  --symbol, -s <SYMBOL>        Trading symbol (e.g., BTC, ETH)");
    eprintln!("  --macd, -m <VALUE>            MACD line value");
    eprintln!("  --signal <VALUE>              MACD signal line value");
    eprintln!("  --histogram <VALUE>            MACD histogram value");
    eprintln!("  --rsi, -r <VALUE>             RSI value (0-100)");
    eprintln!("  --funding-rate, -f <VALUE>    Funding rate as decimal (optional, e.g., -0.0002)");
    eprintln!("  --price, -p <VALUE>            Current price");
    eprintln!("  --help, -h                     Show this help message");
    eprintln!("\nExample:");
    eprintln!("  {} --symbol BTC --macd 0.5 --signal 0.3 --histogram 0.2 --rsi 25.0 --funding-rate -0.0002 --price 45000.0", program);
}

fn print_signal(signal: &kryptex::signals::SignalOutput) {
    println!("  Symbol: {}", signal.symbol);
    let direction_str = match signal.direction {
        kryptex::signals::SignalDirection::Long => "Long",
        kryptex::signals::SignalDirection::Short => "Short",
        kryptex::signals::SignalDirection::None => "Neutral",
    };
    println!("  Direction: {}", direction_str);
    println!("  Confidence: {:.2}%", signal.confidence * 100.0);
    println!("  Price: ${:.2}", signal.price);
    
    if signal.direction != kryptex::signals::SignalDirection::None {
        println!("  Recommended SL: {:.2}%", signal.recommended_sl_pct * 100.0);
        println!("  Recommended TP: {:.2}%", signal.recommended_tp_pct * 100.0);
    } else {
        println!("  Recommended SL: N/A (no signal)");
        println!("  Recommended TP: N/A (no signal)");
    }
    
    println!("  Reasons:");
    for (i, reason) in signal.reasons.iter().enumerate() {
        println!("    {}. {} (contribution: {:.2}%)", i + 1, reason.description, reason.weight * 100.0);
    }
}
