use chrono::Local;
use std::process::exit;
use teloxide::{net::Download, prelude::Requester, types::Message, Bot};
use tokio::fs::{self, DirBuilder, File};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting tgvmp...");

    let token = match std::env::var("BOT_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            log::error!("BOT_TOKEN not set");
            exit(1)
        }
    };

    let bot = Bot::new(token);
    teloxide::repl(bot, move |bot: Bot, message: Message| async move {
        if let Some(voice) = message.voice() {
            let dir = format!("./data/{}", message.chat.id);
            if let Err(e) = DirBuilder::new().recursive(true).create(&dir).await {
                log::error!("Failed to create directory \"{dir}\": {e}");
                return Ok(());
            }
            let mut file_number = 0;
            let date = Local::now().format("%Y_%m_%d_%H_%M_%S").to_string();
            let mut file_path = create_file_path(&dir, &date, file_number);
            loop {
                match fs::try_exists(&file_path).await {
                    Ok(exists) => {
                        if exists {
                            file_number += 1;
                            file_path = create_file_path(&dir, &date, file_number);
                            continue;
                        }

                        break;
                    }
                    Err(e) => {
                        log::error!("Failed to check if file \"{file_path}\" exists: {e}");
                        return Ok(());
                    }
                }
            }
            let tg_file = match bot.get_file(&voice.file.id).await {
                Ok(tg_file) => tg_file,
                Err(e) => {
                    log::error!("Failed to get Telegram file {}: {e}", voice.file.id);
                    return Ok(());
                }
            };
            let mut local_file = match File::create(&file_path).await {
                Ok(local_file) => local_file,
                Err(e) => {
                    log::error!("Failed to create file \"{file_path}\": {e}");
                    return Ok(());
                }
            };
            if let Err(e) = bot.download_file(&tg_file.path, &mut local_file).await {
                log::error!(
                    "Failed to download file {} to \"{file_path}\": {e}",
                    voice.file.id
                );
                return Ok(());
            }
        }
        Ok(())
    })
    .await;
}

fn create_file_path(dir: &str, date: &str, num: i32) -> String {
    format!("{dir}/{date}_{num}.ogg")
}
