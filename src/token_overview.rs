use serde::Deserialize;
use serde::Serialize;

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct TokenOverview {
//     pub data: TokenOverviewData,
// }

#[derive(Default, Debug, Clone,  Serialize, Deserialize)]
pub struct TokenOverviewData {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub display_symbol: Option<String>,
    pub decimals: i32,
    pub price: f64,
    pub logo_url: String,
    pub protocol_id: String,
    pub price_24h_change: f64,
    pub credit_score: f64,
    pub time_at: f64,
    pub is_verified: bool,
    pub is_scam: bool,
    pub is_suspicious: bool,
    pub is_core: bool,
    pub is_wallet: bool,
    pub low_credit_score: bool,   
}
    // "id": "0x48b62137edfa95a428d35c09e44256a739f6b557",
    // "chain": "ape",
    // "name": "Wrapped ApeCoin",
    // "symbol": "WAPE",
    // "display_symbol": null,
    // "optimized_symbol": "WAPE",
    // "decimals": 18,
    // "logo_url": "https://static.debank.com/image/ape_token/logo_url/ape/2357165eac1453c46f526704b51a801b.png",
    // "protocol_id": "",
    // "price": 1.089705,
    // "price_24h_change": 0.022285285426145666,
    // "credit_score": 910069.1626024441,
    // "is_verified": true,
    // "is_scam": false,
    // "is_suspicious": false,
    // "is_core": true,
    // "is_wallet": true,
    // "time_at": 1728868118.0,
    // "low_credit_score": false




   

