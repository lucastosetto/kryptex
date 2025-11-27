//! Test subscription message serialization

use perptrix::services::hyperliquid::messages::{RequestMessage, Subscription};

fn main() {
    let subscription = Subscription::candle("BTC", "1m");
    let request = RequestMessage::Subscribe { subscription };
    
    let json = serde_json::to_string_pretty(&request).unwrap();
    println!("Subscription message:");
    println!("{}", json);
}

