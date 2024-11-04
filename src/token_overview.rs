use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenOverview {
    pub status_code: i32,
    pub data: TokenOverviewData,
}

#[derive(Default, Debug, Clone,  Serialize, Deserialize)]
pub struct TokenOverviewData {
    pub address: String,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "logo")]
    pub logo_url: String,
    pub description: String,
    pub decimals: i32,
    pub social_info: SocialInfo,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SocialInfo {
    pub email: Option<String>,
    pub bitbucket: Option<String>,
    pub discord: Option<String>,
    pub facebook: Option<String>,
    pub github: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub medium: Option<String>,
    pub reddit: Option<String>,
    pub telegram: Option<String>,
    pub tiktok: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub youtube: Option<String>,
}

// {
//     "statusCode": 200,
//     "data": {
//         "address": "0x48b62137edfa95a428d35c09e44256a739f6b557",
//         "name": "Wrapped ApeCoin",
//         "symbol": "WAPE",
//         "logo": "https://www.dextools.io/resources/tokens/logos/3/apechain/0x48b62137edfa95a428d35c09e44256a739f6b557.jpg?1729568016",
//         "description": "",
//         "decimals": 18,
//         "socialInfo": {
//             "email": "",
//             "bitbucket": "",
//             "discord": "",
//             "facebook": "",
//             "github": "",
//             "instagram": "",
//             "linkedin": "",
//             "medium": "",
//             "reddit": "https://www.reddit.com",
//             "telegram": "",
//             "tiktok": "",
//             "twitter": "https://twitter.com/apecoin",
//             "website": "https://apecoin.com/",
//             "youtube": ""
//         }
//     }
// }

