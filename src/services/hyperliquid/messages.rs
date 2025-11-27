//! Hyperliquid WebSocket message types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum RequestMessage {
    #[serde(rename = "subscribe")]
    Subscribe { subscription: Subscription },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { subscription: Subscription },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Subscription {
    Candle {
        #[serde(rename = "type")]
        sub_type: String,
        coin: String,
        interval: String,
    },
    AllMids {
        #[serde(rename = "type")]
        sub_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dex: Option<String>,
    },
    Notification {
        #[serde(rename = "type")]
        sub_type: String,
        user: String,
    },
}

impl Subscription {
    pub fn candle(coin: &str, interval: &str) -> Self {
        Subscription::Candle {
            sub_type: "candle".to_string(),
            coin: coin.to_string(),
            interval: interval.to_string(),
        }
    }

    pub fn all_mids(dex: Option<String>) -> Self {
        Subscription::AllMids {
            sub_type: "allMids".to_string(),
            dex,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResponse {
    pub channel: String,
    pub data: SubscriptionResponseData,
    #[serde(default, rename = "isSnapshot")]
    pub is_snapshot: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResponseData {
    pub method: String,
    pub subscription: Subscription,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WebSocketMessage {
    SubscriptionResponse(SubscriptionResponse),
    CandleData(CandleData),
    AllMidsData(AllMidsData),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandleData {
    #[serde(rename = "channel")]
    pub channel: String,
    #[serde(rename = "data")]
    pub data: CandleUpdate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandleUpdate {
    #[serde(rename = "t")]
    pub start_time: u64,
    #[serde(rename = "T")]
    pub end_time: u64,
    #[serde(rename = "s")]
    pub coin: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "n", default)]
    pub trades: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllMidsData {
    pub channel: String,
    pub data: Vec<MidPrice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MidPrice {
    pub coin: String,
    pub px: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessage {
    pub channel: String,
    pub data: ErrorData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorData {
    pub error: String,
}

