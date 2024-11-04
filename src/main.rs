pub mod token_overview;
pub mod token_top50_holders;

// use tokio::time;
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
use token_overview::TokenOverviewData;
use token_top50_holders::{TokenTopHolders, HolderInfo};


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
    #[command(description = "Get token overview\n\tEntry type: /s ****(token address)")]
    S,
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
        if let Ok(cmd) = Command::parse(text, me.username()) {
            answer(bot, msg, cmd).await?;
        }
    }

    Ok(())
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let username = msg.chat.username().unwrap();
    let message_text = msg.text().unwrap();

    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Start => {
            bot.send_message(msg.chat.id, format!("Welcome to Here {username}! 🎉"))
                .await?
        }
        Command::S => {
            let token_adr = message_text.replace("/s ", "");
            info!("Received command /s for token: {}", token_adr);
            
            let debank_client = Client::new();
            let dextools_api_key = env::var("DEXTOOLS_API_KEY").expect("API_KEY not set");
            let dextools_api_plan = env::var("DEXTOOLS_API_PLAN").expect("API_PLAN not set");

            match get_token_data(debank_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await {
                Ok(token_data) => {
                    // tokio::time::sleep(time::Duration::from_secs(3)).await; //delay for 3 sec to avoid conflict request
                    let token_holders = get_top_50_holders(debank_client.clone(), &dextools_api_key, &dextools_api_plan, &token_adr).await.unwrap();
                    let text =
                        make_token_overview_message(&token_data, &token_holders)
                            .await?;
                    bot.send_message(msg.chat.id, text)  // Changed "text" to text
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?
                }
                Err(e) => {
                    error!("Error fetching token overview: {}", e);
                    bot.send_message(msg.chat.id, "Invalid token address")
                        .await?
                }
            }
        }
    };

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
    match serde_json::from_str(&text) {
        Ok(obj) => Ok(obj),
        Err(e) =>  Err(e),
    }
}

async fn get_token_info(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenOverviewData, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/info",
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

async fn get_token_price_history(client: Client, api_key: &str, api_plan: &str, token_address: &str) -> Result<TokenOverviewData, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/price",
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
    api_plan: &str,
    token_address: &str,
) -> Result<TokenTopHolders, serde_json::Error> {
    let url = format!(
        "https://public-api.dextools.io/{}/v2/token/{}/top_holders?start=0&limit=50",
        api_plan, "apechain", token_address
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
    token_holders: &TokenTopHolders,
) -> Result<String, reqwest::Error> {
    //token Info
    let token_address = &token_data.id;
    let name = &token_data.name;  
    let symbol = &token_data.symbol;
    let price = token_data.price;
    //top holders Info
    let holders_count = token_holders.holders.len();
    let mut sum_usd_amount_top_10_holders = 0.0;
    let mut holders_text = String::from("\n");
    let mut top_num = 0;
    let mut index_on_a_line = 0;
    let mut num_whale = 0;
    let mut num_largefish = 0;
    let mut num_bigfish = 0;
    let mut num_smallfish = 0;
    let mut num_shrimp = 0;

    for holder in &token_holders.holders {
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
            holders_text = holders_text + &link + "\n";
            index_on_a_line = 0;
        } else {
            holders_text = holders_text + &link;
            index_on_a_line += 1;
        }

        if top_num == token_holders.holders.len() {
            holders_text += &format!("\n🐳 ( > $100K ) :  {num_whale}\n🦈 ( $50K - $100K ) :  {num_largefish}\n🐬 ( $10K - $50K ) :  {num_bigfish}\n🐟 ( $1K - $10K ) :  {num_smallfish}\n🦐 ( $0 - $1K ) :  {num_shrimp}\n");
        }
    }
    let sum_usd_amount_top_10_holders = controll_big_float(sum_usd_amount_top_10_holders);

    let text = format!("
⛓  APE

🪙 {name}  ({symbol})

{token_address}
➖➖➖➖➖➖

🏷  Price:  ${price}

🧳  Holders:  {holders_count}
        └ Top 10 Holders :  ${sum_usd_amount_top_10_holders}

{holders_text}
❎ <a href=\"https://twitter.com/search?q={token_address}=typed_query&f=live\"> Search on 𝕏 </a>

📈 <a href=\"https://apescan.io/token/{token_address}\"> APE Scan </a>
");

    Ok(text)
}




// async fn num_floating_point(num: &f64, length: i32) -> Result<f64, reqwest::Error> {
//     let num_floating = ((num * 10_f64.powi(length as i32)).round()) / 10_f64.powi(length as i32);
//     Ok(num_floating)
// }
fn controll_big_float(num: f64) -> String {
    if num > 1000000.0 {
        format!("{:.1}M", num / 1000000.0)
    } else if num > 1000.0 {
        format!("{:.2}K", num / 1000.0)
    } else {
        format!("{:.3}", num)
    }
}
