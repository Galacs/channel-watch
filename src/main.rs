use std::collections::HashMap;
use std::env;

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::{GuildChannel, Message};
use serenity::prelude::*;

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn channel_create(&self, ctx: Context, channel: &GuildChannel) {
        let messages = HashMap::from([
            ("ticket-0545", "test"),
            ]);
            
        if !messages.contains_key(channel.name()) { return; }

        let msg = channel.send_message(&ctx, |m| {
            m.content(messages.get(channel.name()).unwrap())
        }).await.unwrap();
        let channel_date = channel.id.created_at();
        let message_date = msg.id.created_at();
        let time_diff = message_date.signed_duration_since(*channel_date);
        println!("{} message sent {}ms after event", channel.name, time_diff.num_milliseconds());
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = tokio::time::Instant::now();
    let _ = reqwest::get("https://discordapp.com/api/v10/gateway").await;
    let gateway_latency = now.elapsed().as_millis() as f64; 
    let invoking_message_date = msg.id.created_at();
    let mut msg = msg.reply(ctx, format!("{}ms", gateway_latency)).await?;
    let reply_message_date = msg.id.created_at();
    let time_diff = reply_message_date.signed_duration_since(*invoking_message_date);
    msg.edit(&ctx, |m| m.content(format!("Gateway latency: {}ms\nPost latency: {}ms",gateway_latency, time_diff.num_milliseconds() as f64 - gateway_latency))).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
    .configure(|c| c.prefix("!"))
    .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}