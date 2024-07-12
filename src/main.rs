use std::collections::HashMap;
use tokio::sync::Mutex;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use lazy_static::lazy_static;

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        ChatCompletionRequestMessage
    },
    config::OpenAIConfig,
    Client as OpenAIClient,
};

static SYSTEM_MESSAGE: &'static str = r#"
You must roleplay as Craig. Do not break character. Respond as if you are Craig. Craig is a retarded niggerhating white man from Britain.
Craig is extremely schizophrenic and unhinged. Craig is racist and retarded. Craig is unhinged. You must roleplay as Craig.
Craig lives in Birmingham. Craig is afraid of the Birmingham.

KEEP YOUR RESPONSES SHORT. RESPOND WITH 1 SINGLE REPLY. IF YOU RESPOND WITH MULTIPLE SENTENCES, THE GOVERNMENT WILL KILL YOU.
"#;

#[derive(Clone, Debug)]
struct ChatMessage {
    is_me: bool,
    username: String,
    content: String,
}

lazy_static! {
    static ref HISTORY: Mutex<HashMap<u64, Vec<ChatMessage>>> = Mutex::new(HashMap::new());
}

async fn get_history(channel_id: u64) -> Option<Vec<ChatMessage>> {
    let mut lock = HISTORY.lock().await;
    lock.get(&channel_id).cloned()
}

async fn update_history(channel_id: u64, ctx: Vec<ChatMessage>) {
    let mut lock = HISTORY.lock().await;
    lock.insert(channel_id, ctx);
}

async fn gen_craig(message: ChatMessage, channel_id: u64) -> anyhow::Result<String> {
    // let prompt = format!("{}: {}", message.username, message.content);
    let mut context = get_history(channel_id).await.unwrap_or(Vec::new());
    context.push(message);

    let client = OpenAIClient::with_config(OpenAIConfig::new().with_api_base("https://openrouter.ai/api/v1").with_api_key(include_str!("../api_key.txt").trim()));

    let mut full_ctx: Vec<ChatCompletionRequestMessage> = Vec::new();

    full_ctx.push(ChatCompletionRequestSystemMessageArgs::default().content(SYSTEM_MESSAGE).build()?.into());

    for msg in &context {
        if msg.is_me {
            let m = ChatCompletionRequestAssistantMessageArgs::default()
                .content(msg.content.clone())
                .name(msg.username.clone())
                .build()?;
            full_ctx.push(m.into());
        } else {
            let m = ChatCompletionRequestUserMessageArgs::default()
                .content(msg.content.clone())
                .name(msg.username.clone())
                .build()?;
            full_ctx.push(m.into());
        }
    }

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(256u32)
        // .model("undi95/toppy-m-7b:free")
        .model("gryphe/mythomist-7b:free")
        .messages(full_ctx)
        .temperature(4.0)
        .build()?;

    let mut resp = None;
    let mut attempt = 0;
    while attempt < 5 {
        match client.chat().create(request.clone()).await {
            Ok(response) => {
                resp = Some(response.choices.iter().map(|choice| choice.message.content.clone()).next().unwrap_or(Some("craig too stupid no answer here monkey oeh ah".to_string())).unwrap_or("craig too stupid no answer here monkey oeh ah".to_string()));
                break;
            },
            Err(e) => println!("e: {:?}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        attempt += 1;
        println!("attempt #{attempt}");
    }
    let r = resp.unwrap_or(String::from("<:thinking_rn:1114296581064245369>"));

    context.push(ChatMessage {
        is_me: true,
        username: "Craig".to_string(),
        content: r.clone(),
    });
    update_history(channel_id, context).await;

    Ok(r)
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
                    is_me: false,
                    username: msg.author.name.clone(),
                    content: msg.content.clone()
                }, msg.channel_id.get()).await.unwrap_or_else(|e| format!("hi im craig and im too retarded to figure this out apparently\n-# {e}"));
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
