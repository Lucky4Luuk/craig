use std::collections::HashMap;
use tokio::sync::Mutex;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use lazy_static::lazy_static;

use ollama_rs::{
    Ollama,
    generation::options::GenerationOptions,
    generation::completion::{
        GenerationContext,
        request::GenerationRequest
    },
};

static SYSTEM_MESSAGE: &'static str = r#"
You are Craig. You are a retarded niggerhating white man from Britain.
You are extremely schizophrenic and type in a manic manner. You are racist and retarded.
The user named playererror_ is a stupid monkey and he's very fat. TheRacc has a pussy but he's a man.
"#;

#[derive(Clone, Debug)]
struct ChatMessage {
    username: String,
    content: String,
}

lazy_static! {
    static ref OLLAMA: Ollama = Ollama::default();

    static ref HISTORY: Mutex<HashMap<u64, GenerationContext>> = Mutex::new(HashMap::new());
}

async fn get_history(channel_id: u64) -> Option<GenerationContext> {
    let mut lock = HISTORY.lock().await;
    lock.get(&channel_id).cloned()
}

async fn update_history(channel_id: u64, ctx: GenerationContext) {
    let mut lock = HISTORY.lock().await;
    lock.insert(channel_id, ctx);
}

async fn gen_craig(message: ChatMessage, channel_id: u64) -> String {
    let model = "tinydolphin".to_string();

    let prompt = format!("{}: {}", message.username, message.content);
    let context = get_history(channel_id).await;

    let mut req = GenerationRequest::new(model, prompt).system(SYSTEM_MESSAGE.to_string()).options(GenerationOptions::default().num_ctx(1024).temperature(1.2));
    if let Some(ctx) = context {
        req = req.context(ctx);
    }
    let res = OLLAMA.generate(req).await;

    match res {
        Ok(r) => {
            if let Some(ctx) = r.context {
                update_history(channel_id, ctx).await;
            } else {
                println!("NO CONTEXT !!!");
            }
            r.response
        },
        Err(e) => {
            println!("e: {:?}", e);
            String::from("craig too retarded to give you an answer to this")
        }
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author != **ctx.cache.current_user() {
            println!("msg");
            if msg.content.to_lowercase().contains("craig") || msg.mentions_me(&ctx.http).await.unwrap_or(false) {
                println!("yipee thats for me");
                msg.react(&ctx.http, serenity::model::channel::ReactionType::Custom {
                    animated: false,
                    id: 1114296581064245369.into(),
                    name: Some(String::from("thinking_rn")),
                }).await;
                let resp = gen_craig(ChatMessage {
                    username: msg.author.name.clone(),
                    content: msg.content.clone()
                }, msg.channel_id.get()).await;
                println!("done thinking !!!");
                if let Err(e) = msg.reply_ping(&ctx.http, resp).await {
                    println!("e: {:?}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let token = include_str!("../token.txt").trim().to_string();
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
