use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};
use crate::cache::{WandboxInfo, BotInfo};
use crate::utls::discordhelpers::DiscordHelpers;
use serenity_utils::menu::*;
use serenity::framework::standard::CommandError;

#[command]
pub async fn languages(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxInfo>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from("Internal request failure\nWandbox cache is uninitialized, please file a bug."));
        }
    };
    let wbox = wandbox_lock.read().await;

    let mut items : Vec<String> = Vec::new();
    let langs = wbox.get_languages();
    for lang in langs {
        items.push( lang.name);
    }

    let avatar;
    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        avatar = botinfo.get("BOT_AVATAR").unwrap().clone();
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }


    let options = DiscordHelpers::build_menu_controls();
    let pages = DiscordHelpers::build_menu_items(items, 15, "Supported Languages", &avatar, &msg.author.tag());
    let menu = Menu::new(ctx, msg, &pages, options);
    match menu.run().await {
        Ok(m) => m,
        Err(e) => {
            if e.to_string() == "Unknown Message" {
                match msg.react(&ctx.http, DiscordHelpers::build_reaction(success_id, &success_name)).await {
                    Ok(r) => r,
                    Err(_e) => {
                        // No need to fail here - this case is handled above
                        return Ok(());
                    }
                };


                return Ok(())
            }

            return Err(CommandError::from(format!("Failed to build languages menu\n{}", e)));
        }
    };

    debug!("Command executed");
    Ok(())
}