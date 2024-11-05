use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TokenPriceHistory { 
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    pub data: TokenPriceHistoryData,
}

#[derive(Default, Debug, Clone,  Serialize, Deserialize)]
pub struct TokenPriceHistoryData {
    pub price: f64,
    #[serde(rename = "priceChain")]
    pub price_chain: Option<f64>,
    #[serde(rename = "price5m")]
    pub price_5m: Option<f64>,
    #[serde(rename = "priceChain5m")]
    pub price_chain_5m: Option<f64>,
    #[serde(rename = "variation5m")]
    pub variation_5m: Option<f64>,
    #[serde(rename = "variationChain5m")]
    pub variation_chain_5m: Option<f64>,
    #[serde(rename = "price1h")]
    pub price_1h: Option<f64>,
    #[serde(rename = "priceChain1h")]
    pub price_chain_1h: Option<f64>,
    #[serde(rename = "variation1h")]
    pub variation_1h: Option<f64>,
    #[serde(rename = "variationChain1h")]
    pub variation_chain_1h: Option<f64>,
    #[serde(rename = "price6h")]
    pub price_6h: Option<f64>,
    #[serde(rename = "priceChain6h")]
    pub price_chain_6h: Option<f64>,
    #[serde(rename = "variation6h")]
    pub variation_6h: Option<f64>,
    #[serde(rename = "variationChain6h")]
    pub variation_chain_6h: Option<f64>,
    #[serde(rename = "price24h")]
    pub price_24h: Option<f64>,
    #[serde(rename = "priceChain24h")]
    pub price_chain_24h: Option<f64>,
    #[serde(rename = "variation24h")]
    pub variation_24h: Option<f64>,
    #[serde(rename = "variationChain24h")]
    pub variation_chain_24h: Option<f64>,   
}



// {
//     "statusCode": 200,
//     "data": {
//       "price": 0.5495434572627633,
//       "priceChain": 0.0002243328014354464,
//       "price5m": 0.5490491153551353,
//       "priceChain5m": 0.00022397417386279105,
//       "variation5m": 0.0900360084012286,
//       "variationChain5m": 0.16012005601817592,
//       "price1h": 0.5449468548050113,
//       "priceChain1h": 0.0002221451861823698,
//       "variation1h": 0.8434955477258033,
//       "variationChain1h": 0.9847682457907014,
//       "price6h": 0.5393629876684607,
//       "priceChain6h": 0.00021925420605107835,
//       "variation6h": 1.887498739635518,
//       "variationChain6h": 2.3163046565158885,
//       "price24h": 0.5256786401094369,
//       "priceChain24h": 0.0002162710538325091,
//       "variation24h": 4.539811080845557,
//       "variationChain24h": 3.727612854367801
//     }
//   }