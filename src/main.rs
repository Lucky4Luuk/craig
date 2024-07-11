use std::collections::HashMap;
use tokio::sync::Mutex;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use lazy_static::lazy_static;

use ollama_rs::{
    Ollama,
    generation::completion::request::GenerationRequest,
};

static SYSTEM_MESSAGE: &'static str = "You are Craig. You are a racist british man with white heritage. You are also schizophrenic. A child by the name of NotButter is fat and is 2% African.";

#[derive(Clone, Debug)]
struct ChatMessage {
    username: String,
    content: String,
}

lazy_static! {
    static ref OLLAMA: Ollama = Ollama::default();

    static ref HISTORY: Mutex<HashMap<u64, Vec<ChatMessage>>> = Mutex::new(HashMap::new());
}

async fn get_history_and_push(prompt: ChatMessage, channel_id: u64) -> Vec<ChatMessage> {
    let mut lock = HISTORY.lock().await;
    if !lock.contains_key(&channel_id) { lock.insert(channel_id, Vec::new()); }
    let history = lock.get(&channel_id).unwrap().clone();
    let mut modified = history.clone();
    modified.push(prompt);
    if modified.len() > 6 {
        modified.remove(0);
    }
    lock.insert(channel_id, modified);
    history
}

async fn gen_craig(message: ChatMessage, channel_id: u64) -> String {
    let model = "llama2:latest".to_string();
    // let prompt = "Why is the sky blue?".to_string();
    let mut prompt = String::new();
    prompt.push_str(format!("<|im_start|>system\n{SYSTEM_MESSAGE}<|im_end|>\n"));
    let history = get_history_and_push(message.clone(), channel_id).await;
    for msg in history {
        prompt.push_str(format!("<|im_start|>{}\n{}<|im_end|>\n"));
    }
    prompt.push_str(format!("<|im_start|>Craig"));

    let res = OLLAMA.generate(GenerationRequest::new(model, prompt)).await;

    if let Ok(res) = res {
        panic!("{}", res.response);
    }

    todo!()
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
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
