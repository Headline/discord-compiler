use serenity::{
    async_trait,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

use crate::cache::*;
use serenity::model::guild::{Guild, GuildUnavailable};
use crate::utls::discordhelpers::DiscordHelpers;
use chrono::{Duration, Utc, DateTime};

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx : &Context, data : &TypeMap, shards : &Vec<usize>);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx : &Context, data : &TypeMap, shards : &Vec<usize>) {
        let sum : usize = shards.iter().sum();

        // update stats
        let mut stats = data.get::<Stats>().unwrap().lock().await;
        if stats.should_track() {
            stats.post_servers(sum).await;
        }

        let presence_str = format!("{} servers | ;invite", sum);
        ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;
        info!("{} shard(s) ready", shards.len());
        debug!("Existing in {} guilds", sum);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        let now: DateTime<Utc> = Utc::now();
        if guild.joined_at + Duration::seconds(30) > now {
            let data = ctx.data.read().await;

            // publish new server to stats
            {
                let mut stats = data.get::<Stats>().unwrap().lock().await;
                if stats.should_track() {
                    stats.new_server().await;
                }
            }

            // post new server to join log
            {
                let info = data.get::<BotInfo>().unwrap().read().await;
                if let Some(log) = info.get("JOIN_LOG") {
                    if let Ok(id) = log.parse::<u64>() {
                        let emb = DiscordHelpers::build_join_embed(&guild);
                        DiscordHelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
                    }
                }
            }

            // update shard guild count & presence
            let sum : usize = {
                let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
                let index = ctx.shard_id as usize;
                shard_info[index] += 1;
                shard_info.iter().sum()
            };

            let presence_str = format!("{} servers | ;invite", sum);
            ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;

            info!("Joining {}", guild.name);
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable) {
        let data = ctx.data.read().await;
        let mut stats = data.get::<Stats>().unwrap().lock().await;
        if stats.should_track() {
            stats.leave_server().await;
        }

        let info = data.get::<BotInfo>().unwrap().read().await;
        if let Some(log) = info.get("JOIN_LOG") {
            if let Ok(id) = log.parse::<u64>() {
                let emb = DiscordHelpers::build_leave_embed(&incomplete.id);
                DiscordHelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
            }
        }

        // update shard guild count & presence
        let sum : usize = {
            let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
            let index = ctx.shard_id as usize;
            shard_info[index] -= 1;
            shard_info.iter().sum()
        };
        info!("Leaving {}", &incomplete.id);
        let presence_str = format!("{} servers | ;invite", sum);
        ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;
    }


    async fn ready(&self, ctx : Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);
        let data = ctx.data.read().await;
        let mut info = data.get::<BotInfo>().unwrap().write().await;
        info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());

        let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
        shard_info.push(ready.guilds.len());

        if shard_info.len() == ready.shard.unwrap()[1] as usize {
            self.all_shards_ready(&ctx, &data, &shard_info).await;
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

}
