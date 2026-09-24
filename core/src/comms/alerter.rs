use teloxide::prelude::*;
use teloxide::Bot;

pub struct Alerter {
    bot: Bot,
}

impl Alerter {
    pub fn new(token: String) -> Self {
        Self {
            bot: Bot::new(token),
        }
    }

    pub async fn notify(&self, message: &str, chat_id: teloxide::types::ChatId) {
        let _ = self.bot.send_message(chat_id, message).await;
    }
}
