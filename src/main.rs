use std::{env, io::Write};

use color_eyre::{Result, eyre::WrapErr};

use grammers_client::{
    Client, Config as GrammersConfig, InputMessage, Update,
    grammers_tl_types::types::MessageMediaDice,
    session::Session,
    types::{Media, User, media::Dice},
};
use shabby::logging;
use tokio::signal;

#[allow(
    unused_imports,
    reason = "these will just constantly be added and removed otherwise"
)]
use tracing::{debug, error, info, warn};

const SESSION_FILENAME: &str = "shabby.session";

async fn handle_update(client: &Client, update: Update, me: &User) -> Result<bool> {
    match update {
        // Because we're making a userbot, we only care about messages sent by ourselves
        Update::NewMessage(message) if message.sender().is_some_and(|s| s.id() == me.id()) => {
            let chat = message.chat();
            let chat_name = chat.name();

            if let Some(Media::Dice(ref dice)) = message.media() {
                if dice.raw.value != 6 {
                    let reply_to = message.reply_to_message_id();

                    message.delete().await?;

                    let dice_media = Media::Dice(Dice {
                        raw: MessageMediaDice {
                            emoticon: "".to_string(),
                            value: 0,
                        },
                    });

                    let dice_msg = InputMessage::text("")
                        .reply_to(reply_to)
                        .copy_media(&dice_media)
                        .silent(true);

                    client.send_message(&chat, dice_msg).await?;
                    return Ok(false);
                }
            }

            info!(
                "Message in {} ({}): {}",
                chat_name,
                chat.id(),
                message.text()
            );

            let quit_command = message.text().trim().to_lowercase() == "!quit";

            if quit_command {
                info!("Received quit command");
                if let Err(err) = message.delete().await {
                    warn!("Failed to delete quit command message: {}", err);
                }
            }

            Ok(quit_command)
        }
        // Update::Raw(raw) => {
        //     debug!("Raw: {:?}", raw);
        //     Ok(false)
        // }
        _ => Ok(false),
    }
}

async fn handle_updates(client: Client, me: User) -> Result<()> {
    loop {
        let update = client.next_update().await?;
        let quit = handle_update(&client, update, &me).await?;

        if quit {
            break;
        }
    }

    Ok(())
}

/// The entry point for shabby.
///
/// Great things will eventually happen here.
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _log_state =
        logging::init(logging::LogLevel::default()).wrap_err("Failed to init logging")?;

    let cwd = env::current_dir().wrap_err("Failed to get current working directory")?;

    info!("Initializing in {}", cwd.display());

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

    let me = client.get_me().await.wrap_err("Failed to get self")?;

    info!("Successfully connected and authorized");

    println!("Press Ctrl+C to exit");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, exiting");
        }
        update_result = handle_updates(client.clone(), me) => {
            match update_result {
                Ok(_) => info!("Disconnected from Telegram gracefully"),
                Err(e) => error!("Error while handling updates: {}", e),
            }
        }
    }

    info!("Saving session file and exiting");
    client
        .session()
        .save_to_file(SESSION_FILENAME)
        .wrap_err("Failed to save session on exit")?;

    Ok(())
}

// async fn debug_topics(client: Client) -> Result<()> {
//     let topic_channel_id = env::var("SHABBY_TG_TOPIC_CHANNEL")
//         .wrap_err("Topic channel ID not set")?
//         .parse::<i64>()
//         .wrap_err("Topic channel ID must be a number")?;

//     let mut dialogs = client.iter_dialogs();

//     while let Some(dialog) = dialogs.next().await? {
//         let chat = dialog.chat();
//         let id = chat.id();
//         if id == topic_channel_id {
//             println!("{} ({})", chat.name(), chat.id());
//             let mut messages = client.iter_messages(chat);

//             while let Some(message) = messages.next().await? {
//                 if let Some(MessageAction::TopicCreate(topic)) = message.action() {
//                     println!("[{}] <topic> {}", message.id(), topic.title);
//                 }
//                 if message.pinned() {
//                     if let Some(MessageReplyHeader::Header(header)) = &message.raw.reply_to {
//                         if header.forum_topic {
//                             println!("[{}] <topic pin> {}", message.id(), message.text());
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     Ok(())
// }

fn get_from_env_or_stdin(var_name: &str) -> Result<String> {
    env::var(var_name).or_else(|_| {
        print!("{}: ", var_name);
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    })
}
