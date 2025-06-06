use std::{env, io::Write};

use clap::Parser;
use color_eyre::{Result, eyre::WrapErr};
use grammers_client::{
    Client, Config as GrammersConfig, InputMessage, Update,
    grammers_tl_types::types::MessageMediaDice,
    session::Session,
    types::{Chat, Media, Message, User, media::Dice},
};
use tokio::signal;
use tracing::{error, info};

use self::{cli::Cli, config::Config, logging::LogState};

pub mod cli;
mod command;
pub mod config;
mod dirs;
pub mod logging;

struct Bot {
    client: Client,
    me: User,
}

struct Context {
    chat: Chat,
    message: Message,
}

pub async fn run() -> Result<LogState> {
    let cli = Cli::try_parse()?;
    let log_state = logging::init(cli.log_level().unwrap_or_default())?;

    let cwd = env::current_dir().wrap_err("Failed to get current working directory")?;

    info!(cwd = %cwd.display(), "Initializing");

    let config = Config::from_cli(&cli)?;
    if let Some(config_log_level) = config.log_level() {
        log_state.set_level_filter(config_log_level)?;
    }
    let session_path = config.session_filename();
    let session =
        Session::load_file_or_create(session_path).wrap_err("Failed to load or create session")?;

    info!("Using session file: {}", session_path.display());

    let client = Client::connect(GrammersConfig {
        api_id: config.api_id(),
        api_hash: config.api_hash().to_string(),
        session,
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
            .request_login_code(config.phone_number())
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
                    .save_to_file(config.session_filename())
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

    // let mut log_chat: Option<Chat> = None;

    // while let Some(dialog) = client.iter_dialogs().next().await? {
    //     if dialog.chat().id() == me.id() {
    //         log_chat = Some(dialog.chat().clone());
    //         break;
    //     }
    // }

    // if log_chat.is_none() {
    //     error!("Could not find a suitable chat for feedback");
    //     bail!("No suitable chat found for logging");
    // }

    // let log_chat = log_chat.unwrap();

    let bot = Bot {
        client: client.clone(),
        me,
    };

    println!("Press Ctrl+C to exit");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, exiting");
        }
        update_result = handle_updates(&bot) => {
            match update_result {
                Ok(_) => info!("Disconnected from Telegram gracefully"),
                Err(e) => error!("Error while handling updates: {}", e),
            }
        }
    }

    info!("Saving session file and exiting");
    client
        .session()
        .save_to_file(config.session_filename())
        .wrap_err("Failed to save session on exit")?;

    Ok(log_state)
}

async fn handle_updates(bot: &Bot) -> Result<()> {
    loop {
        let update = bot.client.next_update().await?;
        match handle_update(bot, update).await {
            Ok(quit) if quit => {
                break;
            }
            Err(err) => {
                error!(?err, "Error handling update");
            }
            Ok(_) => {}
        }
    }

    Ok(())
}

async fn handle_command(context: &Context) -> Result<bool> {
    let result = command::parse_chat_command(context).wrap_err("Failed to parse chat command")?;

    match result.response {
        Some(command::ActionResponse::Delete) => {
            if let Err(err) = context.message.delete().await {
                error!(?err, "Failed to delete command message");
            }
        }
        Some(command::ActionResponse::Edit(new_message)) => {
            if let Err(err) = context.message.edit(new_message).await {
                error!(?err, "Failed to edit command message");
            }
        }
        Some(command::ActionResponse::Reply(response)) => {
            if let Err(err) = context.message.reply(response).await {
                error!(?err, "Failed to reply to command message");
            }
        }
        None => {}
    };

    Ok(result.quit)

    // match command.action {
    //     BotAction::Quit => {
    //         info!("Received quit command");
    //         if let Err(err) = message.delete().await {
    //             warn!("Failed to delete quit command message: {}", err);
    //         }
    //         Ok(true)
    //     }
    // }
}

async fn handle_message(bot: &Bot, context: &Context) -> Result<bool> {
    let message = &context.message;

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

            bot.client.send_message(&context.chat, dice_msg).await?;
            return Ok(false);
        }
    }

    Ok(false)
}

async fn handle_update(bot: &Bot, update: Update) -> Result<bool> {
    match update {
        // Because we're making a userbot, we only care about messages sent by ourselves
        Update::NewMessage(message) if message.sender().is_some_and(|s| s.id() == bot.me.id()) => {
            let text = message.text().trim();

            let context = Context {
                chat: message.chat(),
                message: message.clone(),
            };

            if text.starts_with('!') {
                match handle_command(&context).await {
                    Err(err) => {
                        if let Some(clap_err) = err.root_cause().downcast_ref::<clap::Error>() {
                            let formatted = clap_err.to_string();
                            if context.chat.id() == bot.me.id() {
                                message.reply(formatted).await?;
                            } else {
                                message.delete().await?;
                                bot.client.send_message(&bot.me, formatted).await?;
                            }
                            Ok(false)
                        } else {
                            Err(err)
                        }
                    }
                    res => res,
                }
            } else {
                handle_message(bot, &context).await
            }

            // info!(
            //     "Message in {} ({}): {}",
            //     chat_name,
            //     chat.id(),
            //     message.text()
            // );
        }
        // Update::Raw(raw) => {
        //     debug!("Raw: {:?}", raw);
        //     Ok(false)
        // }
        _ => Ok(false),
    }
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
