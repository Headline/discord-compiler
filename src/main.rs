mod utls;
mod apis;
mod commands;
mod cache;
mod events;

use std::{
    collections::HashSet,
    env,
    error::Error
};

use serenity::{
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
};

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

/** Command Registration **/
use crate::commands::{
    ping::*,
    botinfo::*,
    compile::*,
    languages::*,
    compilers::*,
    help::*,
    asm::hide::*
};
use crate::apis::dbl::BotsListAPI;
use crate::cache::CacheFiller;

#[group]
#[commands(botinfo,compile,languages,compilers,ping,help,asm)]
struct General;


/** Spawn bot **/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN")?;
    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();

            owners.insert(info.owner.id);

            if let Some(team) = info.team {
                for member in &team.members {
                    owners.insert(member.user.id);
                }
            }

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    info!("Registering owner(s): {}", owners.iter().map(|o| format!("{}", o.0)).collect::<Vec<String>>().join(", "));

    let prefix = env::var("BOT_PREFIX")?;
    let framework = StandardFramework::new()
        .configure(|c| c
        .owners(owners)
        .prefix(&prefix))
        .group(&GENERAL_GROUP);
    let mut client = serenity::Client::new(token)
        .framework(framework)
        .event_handler(events::Handler)
        .await?;

    CacheFiller::fill(client.data.clone(), &prefix).await?;

    let dbl = BotsListAPI::new();
    if dbl.should_spawn() {
        dbl.spawn(client.cache_and_http.http.clone(), client.data.clone());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}