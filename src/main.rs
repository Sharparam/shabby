use std::{env, io::Write};

use color_eyre::{Result, eyre::WrapErr};

use grammers_client::{
    Client, Config as GrammersConfig,
    grammers_tl_types::enums::{MessageAction, MessageReplyHeader},
    session::Session,
};
use shabby::logging;
use tracing::info;

const SESSION_FILENAME: &str = "shabby.session";

/// The entry point for shabby.
///
/// Great things will eventually happen here.
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _log_state =
        logging::init(logging::LogLevel::default()).wrap_err("Failed to init logging")?;

    info!("Initializing");

    let api_id = get_from_env_or_stdin("SHABBY_TG_API_ID")?;
    let api_hash = get_from_env_or_stdin("SHABBY_TG_API_HASH")?;
    let phone_number = get_from_env_or_stdin("SHABBY_TG_PHONE_NUMBER")?;

    let client = Client::connect(GrammersConfig {
        api_id: api_id.parse().wrap_err("Failed to parse API ID")?,
        api_hash,
        session: Session::load_file_or_create(SESSION_FILENAME)
            .wrap_err("Failed to load or create session")?,
        params: Default::default(),
    })
    .await
    .wrap_err("Failed to connect to Telegram")?;

    if !client
        .is_authorized()
        .await
        .wrap_err("Failed to check authorization")?
    {
        info!("Requesting token SMS");
        let token = client
            .request_login_code(&phone_number)
            .await
            .wrap_err("Failed to request login code")?;

        print!("Enter the code you received: ");
        std::io::stdout().flush()?;
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;

        let code = code.trim().to_string();

        let user = match client
            .sign_in(&token, &code)
            .await
            .wrap_err("Failed to sign in")
        {
            Ok(user) => {
                if let Err(err) = client
                    .session()
                    .save_to_file(SESSION_FILENAME)
                    .wrap_err("Failed to save session")
                {
                    client.sign_out().await.wrap_err("Failed to sign out")?;
                    return Err(err);
                };

                user
            }
            Err(err) => {
                return Err(err);
            }
        };

        info!(
            "Successfully signed in as {} (ID: {})",
            user.username().unwrap_or("<no username>"),
            user.id()
        );
    }

    info!("Successfully connected and authorized");

    let topic_channel_id = env::var("SHABBY_TG_TOPIC_CHANNEL")
        .wrap_err("Topic channel ID not set")?
        .parse::<i64>()
        .wrap_err("Topic channel ID must be a number")?;

    let mut dialogs = client.iter_dialogs();

    while let Some(dialog) = dialogs.next().await? {
        let chat = dialog.chat();
        let id = chat.id();
        if id == topic_channel_id {
            println!("{} ({})", chat.name(), chat.id());
            let mut messages = client.iter_messages(chat);

            while let Some(message) = messages.next().await? {
                if let Some(MessageAction::TopicCreate(topic)) = message.action() {
                    println!("[{}] <topic> {}", message.id(), topic.title);
                }
                if message.pinned() {
                    if let Some(MessageReplyHeader::Header(header)) = &message.raw.reply_to {
                        if header.forum_topic {
                            println!("[{}] <topic pin> {}", message.id(), message.text());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_from_env_or_stdin(var_name: &str) -> Result<String> {
    env::var(var_name).or_else(|_| {
        print!("{}: ", var_name);
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    })
}
