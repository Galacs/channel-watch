use std::collections::HashMap;
use std::env;

use serenity::async_trait;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::GuildChannel;
use serenity::prelude::*;

use std::time::SystemTime;

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

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::default())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}