pub mod token_overview;
pub mod token_info;
pub mod token_price_history;
pub mod token_top50_holders;
pub mod token_audit;
pub mod token_pool;
pub mod pool_liquidity;

use tokio::time;
use dotenv::dotenv;
use log::{error, info};
use reqwest::Client;
use serde_json;
use std::env;
use teloxide::{
    prelude::*,
    types::{Me, MessageKind},
    utils::command::BotCommands,
};
use chrono::{NaiveDateTime, DateTime, Utc};
use token_overview::{TokenOverviewData, TokenOverview};
use token_info::TokenInfo;
use token_price_history::TokenPriceHistory;
use token_top50_holders::{TokenTopHolders, HolderInfo};
use token_audit::TokenAudit;
use token_pool::TokenPool;
use pool_liquidity::PoolLiquidity;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display help message")]
    Help,
    #[command(description = "Send the welcome message")]
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();

    let bot_commands = Command::bot_commands();
    if bot.set_my_commands(bot_commands).await.is_err() {
        log::warn!("Could not set up the commands.");
    }

    Dispatcher::builder(
        bot,
        dptree::entry().branch(Update::filter_message().endpoint(message_handler)),
    )
    .build()
    .dispatch()
    .await;

    Ok(())
}

async fn message_handler(bot: Bot, msg: Message, me: Me) -> ResponseResult<()> {
    dotenv().ok();

    if let MessageKind::WebAppData(data) = msg.kind {
        bot.send_message(msg.chat.id, data.web_app_data.data)
            .await?;
    } else if let Some(text) = msg.text() {

            let chat_type = match msg.chat.kind {
                teloxide::types::ChatKind::Private { .. } => {
                    "a private chat".to_string()
                }
                teloxide::types::ChatKind::Public(ref public_chat) => {
                    match public_chat.kind {
                        teloxide::types::PublicChatKind::Group { .. } => "a group".to_string(),
                        teloxide::types::PublicChatKind::Supergroup { .. } => "a supergroup".to_string(),
                        teloxide::types::PublicChatKind::Channel { .. } => "a channel".to_string(),
                    }
                }
            };

            if chat_type == "a group" || chat_type == "a supergroup" {
                let username = msg.from()
                                    .and_then(|user| user.username.clone())
                                    .unwrap_or_else(|| {
                                        msg.from()
                                            .map(|user| user.first_name.clone())
                                            .unwrap_or_else(|| "Unknown User".to_string())
                                    });
                if let Ok(cmd) = Command::parse(text, me.username()) {
                    answer_command(bot, msg, cmd, username).await?;
                } else {
                    answer_message(bot, msg).await?;
                }
            } else {
                bot.send_message(msg.chat.id, "This message is not available in this chat type. Please try again in group chat.")
                    .await?;
        }
        
    }

    Ok(())
}

async fn answer_command(bot: Bot, msg: Message, cmd: Command, username: String) -> ResponseResult<()> {

    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            bot.send_message(msg.chat.id, format!("Welcome to Here @{username}! 🎉"))
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Invalid command")
                .await?;
        }
    }
    Ok(())
}

async fn answer_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    let token_adr = msg.text().unwrap();
    if token_adr.starts_with("0x") && token_adr.len() == 42 && token_adr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                
        let request_client = Client::new();
        let dextools_api_key = env::var("DEXTOOLS_API_KEY").expect("API_KEY not set");
        let dextools_api_plan = env::var("DEXTOOLS_API_PLAN").expect("API_PLAN not set");
        let debank_api_key = env::var("DEBANK_API_KEY").expect("API_KEY not set");
        
        match get_token_data(request_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await {
            Ok(token_data) => {
                // tokio::time::sleep(time::Duration::from_secs(1)).await; //delay for 1 sec to avoid conflict request
                let token_info = get_token_info(request_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await.unwrap_or_default();
                // tokio::time::sleep(time::Duration::from_secs(1)).await; //delay for 1 sec to avoid conflict request
                let token_price_history = get_token_price_history(request_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await.unwrap_or_default();
                // tokio::time::sleep(time::Duration::from_secs(1)).await; //delay for 1 sec to avoid conflict request
                let token_top_holders = get_top_50_holders(request_client.clone(), &debank_api_key, &token_adr).await.unwrap_or_default();
                // tokio::time::sleep(time::Duration::from_secs(1)).await; //delay for 1 sec to avoid conflict request
                let token_audit = get_token_audit(request_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await.unwrap_or_default();
                // tokio::time::sleep(time::Duration::from_secs(1)).await; //delay for 1 sec to avoid conflict request
                //make message
                let text =
                make_token_overview_message(&token_data, &token_info, &token_price_history, &token_top_holders, &token_audit)
                        .await?;
                bot.send_message(msg.chat.id, text)  // Changed "text" to text
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?;
            }
            Err(e) => {
                error!("Error fetching token overview: {}", e);
                bot.send_message(msg.chat.id, "Invalid token address")
                .await?;
            }
        }
    }
    Ok(())
}


async fn get_token_data(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenOverviewData, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/{}",
        api_plan, "apechain", token_address
    );

    let response = client
        .get(&url)
        .header("X-API-KEY", api_key)
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str::<TokenOverview>(&text) {
        Ok(token_overview) => Ok(token_overview.data),
        Err(e) => Err(e),
    }
}

async fn get_token_info(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenInfo, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/{}/info",
        api_plan, "apechain", token_address
    );

    let response = client
    .get(&url)
    .header("X-API-KEY", api_key)
    .send()
    .await
    .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) =>  Err(e),
    }
}

async fn get_token_price_history(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenPriceHistory, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/{}/price",
        api_plan, "apechain", token_address
    );

    let response = client
    .get(&url)
    .header("X-API-KEY", api_key)
    .send()
    .await
    .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) =>  Err(e),
    }
}

async fn get_token_audit(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenAudit, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/{}/audit",
        api_plan, "apechain", token_address
    );
    let response = client
    .get(&url)
    .header("X-API-KEY", api_key)
    .send()
    .await
    .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) =>  Err(e),
    }
}

async fn get_top_50_holders(
    client: Client,
    api_key: &str,
    token_address: &str,
) -> Result<TokenTopHolders, serde_json::Error> {
    let url = format!(
        "https://pro-openapi.debank.com/v1/token/top_holders?chain_id={}&id={}&start=0&limit=50",
        "ape",
        token_address
    );
    
    let response = client
        .get(&url)
        .header("AccessKey", api_key)
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();
    
    let holders: Vec<HolderInfo> = serde_json::from_str(&text)?;
    Ok(TokenTopHolders { holders })
}

async fn make_token_overview_message(
    token_data: &TokenOverviewData,
    token_info: &TokenInfo,
    token_price_history: &TokenPriceHistory,
    token_top_holders: &TokenTopHolders,
    token_audit: &TokenAudit,
) -> Result<String, reqwest::Error> {
    //token overview
    let token_address = &token_data.address;
    let name = &token_data.name;
    let symbol = &token_data.symbol;
    let logo_url = &token_data.logo_url;
    let creation_date = &token_data.creation_date.clone().unwrap_or_default();
    let age = calculate_age(creation_date);

    //social info
    let mut social_text = String::new();
    let email = &token_data.social_info.email.clone().unwrap_or_default();
    if !email.is_empty() {
        social_text += &format!(" <a href=\"{email}\">📧 </a>");
    }
    let bitbucket = &token_data.social_info.bitbucket.clone().unwrap_or_default();
    if !bitbucket.is_empty() {
        social_text += &format!(" <a href=\"{bitbucket}\">🗃️ </a>");
    }
    let discord = &token_data.social_info.discord.clone().unwrap_or_default();
    if !discord.is_empty() {
        social_text += &format!(" <a href=\"{discord}\">💭 </a>");
    }
    let facebook = &token_data.social_info.facebook.clone().unwrap_or_default();
    if !facebook.is_empty() {
        social_text += &format!(" <a href=\"{facebook}\">ⓕ </a>");
    }
    let github = &token_data.social_info.github.clone().unwrap_or_default();
    if !github.is_empty() {
        social_text += &format!(" <a href=\"{github}\">🐱 </a>");
    }
    let instagram = &token_data.social_info.instagram.clone().unwrap_or_default();
    if !instagram.is_empty() {
        social_text += &format!(" <a href=\"{instagram}\">📸 </a>");
    }
    let linkedin = &token_data.social_info.linkedin.clone().unwrap_or_default();
    if !linkedin.is_empty() {
        social_text += &format!(" <a href=\"{linkedin}\">ℹ️ </a>");
    }
    let medium = &token_data.social_info.medium.clone().unwrap_or_default();
    if !medium.is_empty() {
        social_text += &format!(" <a href=\"{medium}\">Ⓜ️ </a>");
    }
    let reddit = &token_data.social_info.reddit.clone().unwrap_or_default();
    if !reddit.is_empty() {
        social_text += &format!(" <a href=\"{reddit}\">🎯</a>");
    }
    let telegram = &token_data.social_info.telegram.clone().unwrap_or_default();
    if !telegram.is_empty() {
        social_text += &format!(" <a href=\"{telegram}\">🕊️ </a>");
    }
    let tiktok = &token_data.social_info.tiktok.clone().unwrap_or_default();
    if !tiktok.is_empty() {
        social_text += &format!(" <a href=\"{tiktok}\">🎬 </a>");
    }
    let twitter = &token_data.social_info.twitter.clone().unwrap_or_default();
    if !twitter.is_empty() {
        social_text += &format!(" <a href=\"{twitter}\">𝕏 </a>");
    }
    let website = &token_data.social_info.website.clone().unwrap_or_default();
    if !website.is_empty() {
        social_text += &format!(" <a href=\"{website}\">🌐 </a>");
    }
    let youtube = &token_data.social_info.youtube.clone().unwrap_or_default();
    if !youtube.is_empty() {
        social_text += &format!(" <a href=\"{youtube}\">🎥</a>");
    }


    // # token Info
    // let total_supply = token_info.data.total_supply;
    // let mcap = match token_info.data.mcap {
        //     Some(cap) => cap,
        //     None => 0.0,
        // };
    let holders_count = &token_info.data.holders;
    let fdv = controll_big_float(token_info.data.fdv);

    //top price history
    let price = num_floating_point(&token_price_history.data.price, 3)  ;
    let price_1h = num_floating_point(&token_price_history.data.price_1h.unwrap_or_default(), 3);
    let price_6h = num_floating_point(&token_price_history.data.price_6h.unwrap_or_default(), 3);
    let price_24h = num_floating_point(&token_price_history.data.price_24h.unwrap_or_default(), 3);
    let variation_1h = num_floating_point(&token_price_history.data.variation_1h.unwrap_or_default(), 2);
    let variation_6h = num_floating_point(&token_price_history.data.variation_6h.unwrap_or_default(), 2);
    let variation_24h = num_floating_point(&token_price_history.data.variation_24h.unwrap_or_default(), 2);

     //top holders Info
    //  let holders_count = token_top_holders.holders.len();
     let mut sum_usd_amount_top_10_holders = 0.0;
     let mut holders_text = String::from("\n");
     let mut top_num = 0;
     let mut index_on_a_line = 0;
     let mut num_whale = 0;
     let mut num_largefish = 0;
     let mut num_bigfish = 0;
     let mut num_smallfish = 0;
     let mut num_shrimp = 0;
    
     holders_text += &format!("<u><b><i>50 Top Holders Map</i></b></u>\n        ");
     for holder in &token_top_holders.holders {
         let holder_address = &holder.holder_address;
         let usd_amount = holder.usd_amount;
 
         top_num += 1;
         if top_num <= 10 {
             sum_usd_amount_top_10_holders += usd_amount;
         }
 
         let whale_symbol = if usd_amount > 100000.0 {
             num_whale += 1;
             "🐳"
         } else if usd_amount > 50000.0 {
             num_largefish += 1;
             "🦈"
         } else if usd_amount > 10000.0 {
             num_bigfish += 1;
             "🐬"
         } else if usd_amount > 1000.0 {
             num_smallfish += 1;
             "🐟"
         } else {
             num_shrimp += 1;
             "🦐"
         };
 
         let link = format!("<a href=\"https://suiscan.xyz/mainnet/account/{holder_address}?Amount={usd_amount}\">{whale_symbol}</a>");
         if index_on_a_line == 9 {
             holders_text = holders_text + &link + "\n        ";
             index_on_a_line = 0;
         } else {
             holders_text = holders_text + &link;
             index_on_a_line += 1;
         }
 
         if top_num == token_top_holders.holders.len() {
             holders_text += &format!("\n        🐳 ( > $100K ) :  {num_whale}\n        🦈 ( $50K - $100K ) :  {num_largefish}\n        🐬 ( $10K - $50K ) :  {num_bigfish}\n        🐟 ( $1K - $10K ) :  {num_smallfish}\n        🦐 ( $0 - $1K ) :  {num_shrimp}\n");
         }
     }
    let sum_usd_amount_top_10_holders = controll_big_float(sum_usd_amount_top_10_holders);

    //token audit
    let mut audit_text = String::new();
    let token_audit_status = &token_audit.status_code;
    if *token_audit_status == 200 {
        let is_open_source = &token_audit.data.is_open_source;
        let is_honeypot = &token_audit.data.is_honeypot;
        let is_mintable = &token_audit.data.is_mintable;
        let is_proxy = &token_audit.data.is_proxy;
        let slippage_modifiable = &token_audit.data.slippage_modifiable;
        let is_blacklisted = &token_audit.data.is_blacklisted;
        let is_contract_renounced = &token_audit.data.is_contract_renounced;
        let is_potentially_scam = &token_audit.data.is_potentially_scam;
        // let sell_tax_min = &token_audit.data.sell_tax.min;
        // let sell_tax_max = &token_audit.data.sell_tax.max;
        // let buy_tax_min = &token_audit.data.buy_tax.min;
        // let buy_tax_max = &token_audit.data.buy_tax.max;

        audit_text += &format!("🔍 Audit\n");
        if is_open_source    == "yes" {
            audit_text += &format!("        🔓 Open source: ✅\n");
        } else if is_open_source == "no" {
            audit_text += &format!("        🔓 Open source: ❌\n");
        }
        if is_honeypot == "yes" {
            audit_text += &format!("        🍯 Honeypot: ✅\n");
        } else if is_honeypot == "no" {
            audit_text += &format!("        🍯 Honeypot: ❌\n");
        }
        if is_mintable == "yes" {  
            audit_text += &format!("        🖨 Mintable: ✅\n");
        } else if is_mintable == "no" {
            audit_text += &format!("        🖨 Mintable: ❌\n");
        }   
        if is_proxy == "yes" {
            audit_text += &format!("        🔄 Proxy: ✅\n");
        } else if is_proxy == "no" {
            audit_text += &format!("        🔄 Proxy: ❌\n");
        }   
        if slippage_modifiable == "yes" {
            audit_text += &format!("        📊 Slippage modifiable: ✅\n");
        } else if slippage_modifiable == "no" {
            audit_text += &format!("        📊 Slippage modifiable: ❌\n");
        }   
        if is_blacklisted == "yes" {
            audit_text += &format!("        ⛔ Blacklisted: ❗\n");
        } else if is_blacklisted == "no" {
            audit_text += &format!("        ⛔ Blacklisted: ❌\n");
        }
        if is_contract_renounced == "yes" {
            audit_text += &format!("        📜 Contract renounced: ✅\n");
        } else if is_contract_renounced == "no" {
            audit_text += &format!("        📜 Contract renounced: ❌\n");
        }
        if is_potentially_scam == "yes" {
            audit_text += &format!("        ⚠️ Potentially scam: ❗\n");
        } else if is_potentially_scam == "no" {
            audit_text += &format!("        ⚠️ Potentially scam: ❌\n");
        }
        // if *sell_tax_min != 0.0 || *sell_tax_max != 0.0 {
        //     audit_text += &format!("        ⬇️ Sell tax: {sell_tax_min} - {sell_tax_max}\n");
        // }
        // if *buy_tax_min != 0.0 || *buy_tax_max != 0.0 {
        //     audit_text += &format!("        ⬆️ Buy tax: {buy_tax_min} - {buy_tax_max}\n");
        // }
    }

    //token pool
    let client = Client::new();
    let dextools_api_key = env::var("DEXTOOLS_API_KEY").expect("API_KEY not set");
    let dextools_api_plan = env::var("DEXTOOLS_API_PLAN").expect("API_PLAN not set");
    let mut page = 0;
    let mut _liquidity = 0.0;
    loop {
        let token_pool_page = get_token_pool(client.clone(), &dextools_api_key, &dextools_api_plan, token_address, page).await.unwrap_or_default();
        for pool in &token_pool_page.data.results {
            let pool_address = &pool.address;
            let pool_liquidity = get_pool_liquidity(client.clone(), &dextools_api_key, &dextools_api_plan, pool_address).await.unwrap_or_default();
            _liquidity += pool_liquidity.data.liquidity.unwrap_or_default();
        }
        if token_pool_page.data.page == token_pool_page.data.total_pages {
            break;
        }
        page += 1;
    }
    let liquidity = controll_big_float(_liquidity);

    let text = format!("
<a href=\"https://dexscreener.com/apechain/{token_address}\">🚀</a> <a href=\"{logo_url}\">{name}  </a>{symbol}
🌐 ApeChain @ Camelot
💰 USD:  ${price}
💎 FDV:  ${fdv}
💦 Liquidity:  ${liquidity}
📈 Price history
        └ <i>1H:</i>    ${price_1h} / {variation_1h}%  
        └ <i>6H:</i>    ${price_6h} / {variation_6h}%  
        └ <i>24H:</i>  ${price_24h} / {variation_24h}% 
{audit_text}🕐 Age:  {age}
🧰 More: {social_text}
👩‍👧‍👦 Holders: {holders_count}
        └ Top 10 Holders :  ${sum_usd_amount_top_10_holders}
{holders_text} 
<code>{token_address}</code>
<a href=\"https://dexscreener.com/apechain/{token_address}\">DEX</a> <a href=\"https://apescan.io/address/{token_address}\">EXP</a>

❎ <a href=\"https://twitter.com/search?q={token_address}=typed_query&f=live\"> Search on 𝕏 </a>
📈 <a href=\"https://apescan.io/token/{token_address}\"> APE Scan </a>
");

    Ok(text)
}




fn num_floating_point(num: &f64, length: i32) -> f64 {
    ((num * 10_f64.powi(length as i32)).round()) / 10_f64.powi(length as i32)
}

fn controll_big_float(num: f64) -> String {
    if num > 1_000_000.0 {
        format!("{:.1}M", num / 1_000_000.0)
    } else if num > 1_000.0 {
        format!("{:.2}K", num / 1000.0)
    } else {
        format!("{:.3}", num)
    }
}
// Add this new function before make_token_overview_message
fn calculate_age(creation_date: &str) -> String {
    if let Ok(date) = NaiveDateTime::parse_from_str(creation_date, "%Y-%m-%dT%H:%M:%S") {
        let creation = DateTime::<Utc>::from_naive_utc_and_offset(date, Utc);
        let now = Utc::now();
        let duration = now.signed_duration_since(creation);
        
        let days = duration.num_days();
        if days > 365 {
            format!("{:.1} years", days as f64 / 365.0)
        } else if days > 30 {
            format!("{:.1} months", days as f64 / 30.0)
        } else {
            format!("{} days", days)
        }
    } else {
        "🔥".to_string()
    }
}

async fn get_token_pool(client: Client, api_key: &str, api_plan: &str, token_address: &str, page: i32) -> Result<TokenPool, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/{}/pools?sort=creationTime&order=desc&from=2023-10-01T00%3A00%3A00.000Z&to=2024-11-05T00%3A00%3A00.000Z&pageSize=50&page={}",
        api_plan, "apechain", token_address, page
    );
    let response = client
    .get(&url)
    .header("X-API-KEY", api_key)
    .send()
    .await
    .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) => Err(e),
    }
}

async fn get_pool_liquidity(client: Client, api_key: &str, api_plan: &str, pool_address: &str) -> Result<PoolLiquidity, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/pool/{}/{}/liquidity",
        api_plan, "apechain", pool_address
    );
    let response = client
    .get(&url)
    .header("X-API-KEY", api_key)
    .send()
    .await
    .unwrap();

    let text = response.text().await.unwrap();
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) =>  Err(e),
    }
}
