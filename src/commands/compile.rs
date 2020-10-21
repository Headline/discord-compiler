use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{Args, CommandResult, macros::command, CommandError};

use crate::cache::{WandboxInfo, BotInfo, Stats};
use wandbox::*;

use crate::utls::parser::{Parser, ParserResult};
use crate::utls::discordhelpers::*;

#[command]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let success_id;
    let success_name;
    let loading_id;
    let loading_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
        loading_id = botinfo.get("LOADING_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

    // parse user input
    let result : ParserResult = Parser::get_components(&msg.content).await?;


    // build user input
    let mut builder = CompilationBuilder::new();
    builder.code(&result.code);
    builder.target(&result.target);
    builder.stdin(&result.stdin);
    builder.save(true);
    builder.options(result.options);


    // aquire lock to our wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxInfo>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from("Internal request failure\nWandbox cache is uninitialized, please file a bug."));
        }
    };
    let wbox = wandbox_lock.read().await;

    // build request
    match builder.build(&wbox) {
        Ok(()) => (),
        Err(e) => {
            return Err(CommandError::from(format!("An internal error has occurred while building request.\n{}", e)));
        }
    };

    // send out loading emote
    let reaction = match msg.react(&ctx.http, DiscordHelpers::build_reaction(loading_id, &loading_name)).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions?\n{}", e)));
        }
    };

    // dispatch our req
    let result = match builder.dispatch().await {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // remove our loading emote
    match msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await {
        Ok(()) => (),
        Err(_e) => {
            return Err(CommandError::from("Unable to remove reactions!\nAm I missing permission to manage messages?"));
        }
    }

    // Dispatch our request
    let emb = DiscordHelpers::build_compilation_embed( &msg.author, &result);
    let mut emb_msg = DiscordHelpers::embed_message(emb);
    let compilation_embed = msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

    // Success/fail react
    let reaction;
    if result.status == "0" {
        reaction = DiscordHelpers::build_reaction(success_id, &success_name);
    }
    else {
        reaction = ReactionType::Unicode(String::from("❌"));
    }
    compilation_embed.react(&ctx.http, reaction).await?;


    let data = ctx.data.read().await;
    let stats = data.get::<Stats>().unwrap().lock().await;
    if stats.should_track() {
        stats.compilation(&builder.lang, result.status == "1").await;
    }
    debug!("Command executed");
    Ok(())
}

