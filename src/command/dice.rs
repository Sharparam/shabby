use clap::Args;
use grammers_client::{
    InputMessage,
    grammers_tl_types::types::MessageMediaDice,
    types::{Media, media::Dice},
};

use super::{ActionResult, BotCommandError};

#[derive(Args, Debug)]
pub struct DiceArgs {
    pub emoji: String,
}

impl DiceArgs {
    pub fn handle(&self) -> Result<ActionResult, BotCommandError> {
        let dice_media = Media::Dice(Dice {
            raw: MessageMediaDice {
                emoticon: self.emoji.clone(),
                value: 0,
            },
        });

        Ok(ActionResult::reply(
            InputMessage::text("").copy_media(&dice_media).silent(true),
        ))
    }
}
