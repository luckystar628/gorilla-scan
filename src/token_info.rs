use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    pub data: TokenInfoData,
}

#[derive(Default, Debug, Clone,  Serialize, Deserialize)]
pub struct TokenInfoData {
    #[serde(rename = "totalSupply")]
    pub total_supply: f64,
    #[serde(default)]
    pub mcap: Option<f64>,
    pub fdv: f64,
    pub holders: i32,
  
}


// {
//     "statusCode": 200,
//     "data": {
//         "totalSupply": 7999396.840523477,
//         "mcap": null,
//         "fdv": 7376285.548029669,
//         "holders": 1943
//     }
// }

