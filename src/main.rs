use std::collections::HashMap;
use std::env;

use notify::Watcher;
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::{GuildChannel, Message};
use serenity::prelude::*;
use tokio::sync::watch;
use tokio::fs;
use tokio::io::AsyncBufReadExt;

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn channel_create(&self, ctx: Context, channel: &GuildChannel) {
        let data = ctx.data.read().await;
        let messages = data.get::<MessagesData>().unwrap();
            
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

pub struct MessagesData;

impl serenity::prelude::TypeMapKey for MessagesData {
    type Value = HashMap<String, String>;
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
    .configure(|c| c.prefix("!"))
    .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let messages = load_messages().await.expect("no messages");

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    client.data.write().await.insert::<MessagesData>(messages);
    let (tx, mut rx) = watch::channel(false);
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        match res {
           Ok(event) => {
               if let notify::EventKind::Modify(_) = event.kind {
                   tx.send(true).unwrap();
               };
            },
           Err(e) => println!("watch error: {:?}", e),
        }
    }).expect("can't set up watcher");
    watcher.watch(std::path::Path::new("messages.txt"), notify::RecursiveMode::NonRecursive).expect("can't set up watcher");
    
    let data = client.data.clone();
    let _file_watchdog = tokio::task::spawn(async move {
        loop {
            if rx.changed().await.is_ok() {
                let messages = load_messages().await.unwrap();
                data.write().await.insert::<MessagesData>(messages);
            }
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn load_messages() -> Option<HashMap<String, String>> {
    let filename = "messages.txt";
    let Ok(buf) = fs::read(filename).await else {
        panic!("Could not read file `{}`", filename);
    };
    let mut lines = buf.lines();

    let mut messages: HashMap<String, String> = HashMap::new();
    let mut key = String::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(previous_entry) = messages.get_mut(&key) {
                while previous_entry.chars().last().unwrap() == '\n' {
                    previous_entry.pop();
                }
            };
            key = line[1..line.len() - 1].to_owned();
            messages.insert(key.clone(), String::new());
        } else {
            let entry = messages.get_mut(&key).unwrap();
            entry.push_str(&line);
            entry.push('\n');
        }
    }

    let last_entry = messages.get_mut(&key).unwrap();
    while last_entry.chars().last().unwrap() == '\n' {
        last_entry.pop();
    }

    Some(messages)
}