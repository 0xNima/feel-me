extern crate dotenv;
extern crate feel_me;
extern crate redis;

use teloxide::{
    prelude::*, 
    types::{ReplyMarkup, InlineKeyboardMarkup, InlineKeyboardButton, InlineKeyboardButtonKind, InputFile},
    dispatching::DispatcherHandler
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::{error::Error, env};
use dotenv::dotenv;
use feel_me::{DBManager, Res, HELPTEXT, LOG, FEEDBACK, REPORT};
use redis::{Commands, RedisResult};



fn handle_text(
    text: &str, 
    dbm: &DBManager, 
    chat_id: i64, 
    redis_con: &mut redis::Connection) -> Res {  
        let response;
        if text.starts_with('/') {
            let splited: Vec<&str> = text.split(' ').collect();
            match splited[0] {
                "/start" => {
                    if splited.len() > 1 {
                        let nickname = splited[1];
                        let user = dbm.get_user_by_nickname(nickname);
                        if let Some(user) = user {
                            let _: () = redis::cmd("SET")
                                        .arg(chat_id)
                                        .arg(user.chat_id)
                                        .query(redis_con)
                                        .unwrap();
                            return Res {
                                text: Some(format!("Please Send the music that you want to share with {}", user.name)),
                                show_cancel_btn: true,
                                to_id: None,
                                msg_id: None,
                                file_unique_id: None
                            }
                        } else {
                            response = Some(String::from("Oops!\nInvalid link"));
                        }
                    } else {
                        response = Some(HELPTEXT.clone());
                    }
                },
                "/help" => {
                    response = Some(HELPTEXT.clone());
                }
                _ => { 
                    response = Some(String::from("Invalid Command"));
                }
            } 
        } else {
            let result: RedisResult<String> = redis::cmd("GET")
            .arg(chat_id)
            .query(redis_con);

            if let Ok(_text) = result {
                let splited: Vec<&str> = _text.split("_").collect();
                if splited.len() == 3 {
                    let _: () = redis::cmd("DEL")
                    .arg(chat_id)
                    .query(redis_con)
                    .unwrap();
                    return Res {
                        text: Some(String::from("Feedback is sent")),
                        show_cancel_btn: false,
                        to_id: Some(String::from(splited[0])),
                        msg_id: Some(splited[1].parse::<i32>().unwrap()),
                        file_unique_id: Some(String::from(splited[2]))
                    }
                }
            }
            response = None;
        }
    Res { text: response, show_cancel_btn: false, to_id: None, msg_id: None, file_unique_id: None }
}

async fn callback(
    cx: UpdateWithCx<AutoSend<Bot>, Message>
) -> Result<(), Box<dyn Error + Send + Sync>> {


    use feel_me::schema::users::dsl::*;
    
    let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
    let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");
    let db_manager = DBManager::new(&databse_url);

    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;

    let r: String; 

    if let Some(text) = cx.update.text() {
        let res = handle_text(text, &db_manager, cx.chat_id(), &mut con);
        if let Some(response) = res.text {
            if res.show_cancel_btn {
                cx.answer(response.clone()).reply_markup(ReplyMarkup::InlineKeyboard(
                    InlineKeyboardMarkup::new(
                        vec![vec![InlineKeyboardButton::callback("Cancel".into(), "cancel".into())]]
                    )
                )).await?;
                return Ok(())
            } else if let Some(to_id) = res.to_id {
                let msg_id = res.msg_id.unwrap();
                let file_unique_id = res.file_unique_id;
                cx.requester.send_message(to_id.clone(), text).reply_to_message_id(msg_id).await?;
                db_manager.set_history(cx.chat_id(), to_id.parse::<i64>().unwrap(), *FEEDBACK, msg_id, file_unique_id);
                cx.requester.send_message(cx.chat_id(), response).await?;
                return Ok(())
            } else {
                r = response;
            }
        } else {
            r = String::from("Invalid Message");
        }
    } else if let Some(audio) = cx.update.audio() {
        let target_id: RedisResult<i64> = redis::cmd("GET")
        .arg(cx.chat_id())
        .query(&mut con);

        if let Ok(to_id) = target_id {
            let au = InputFile::file_id(&audio.file_id);

            cx.requester.send_audio(to_id, au).reply_markup(ReplyMarkup::InlineKeyboard(
                InlineKeyboardMarkup::new(
                    vec![
                        vec![
                            InlineKeyboardButton::callback("Feedback".into(), format!("feedback_{}_{}_{}", cx.chat_id(), cx.update.id, audio.file_unique_id)),
                            InlineKeyboardButton::callback("Report User".into(), format!("report_{}", cx.chat_id()))
                        ]
                    ]
                )
            )).await?;
            db_manager.set_history(cx.chat_id(), to_id, *LOG, cx.update.id, Some(audio.file_unique_id.clone()));
            cx.requester.send_message(cx.chat_id(), "Music Has Been send").await?;
            let _: () = redis::cmd("DEL")
                               .arg(cx.chat_id())
                               .query(&mut con)
                               .unwrap();
            return Ok(())
        } else {
            r = String::from("The target is unknown\nClick on target user's link and then send the music");
        }
    } else {
        r = String::from("Invalid message format\n");
    }

    cx.answer(r).reply_markup(ReplyMarkup::InlineKeyboard(
        InlineKeyboardMarkup::new(
            vec![
                vec![
                    InlineKeyboardButton::callback("Help".into(), "help".into()),
                    InlineKeyboardButton::callback("My Link".into(), "get_link".into())
                ]
            ]
        )
    )).await?;

    Ok(())
}

async fn q_callback(cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>) -> Result<(), Box<dyn Error + Send + Sync>> {

    if let Some(data) = cx.update.data {

        let id = cx.update.from.id;
        let name_ = cx.update.from.full_name();
        let username_ = cx.update.from.username;
        let response;

        if data == "get_link" {
            use feel_me::schema::users::dsl::*;
            let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
            let db_manager = DBManager::new(&databse_url);

            let user_;
            if let Some(user) = db_manager.get_user_by_id(id) {
                user_ = user
            } else {
                user_ = db_manager.create_user(id, name_, username_);
            }
            response = format!("https://t.me/FeelMeBot?start={}", user_.nickname);

        } else if data == "cancel" {
            let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");
            let client = redis::Client::open(redis_url)?;
            let mut con = client.get_connection()?;
            let _: () = redis::cmd("DEL")
            .arg(id)
            .query(&mut con)
            .unwrap();
            cx.requester.send_message(id, String::from("Cancel sendig music")).await?;
            return Ok(())

        } else if data.starts_with("feedback_") {
            let splited: Vec<&str> = data.split("_").collect();

            let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
            let db_manager = DBManager::new(&databse_url);

            if db_manager.history_exists(id, splited[1].parse::<i64>().unwrap(), splited[2].parse::<i32>().unwrap(), String::from(splited[3]), *FEEDBACK) {
                response = String::from("Feedback is sent before");
            } else {
                let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");
                let client = redis::Client::open(redis_url)?;
                let mut con = client.get_connection()?;
                let _: () = redis::cmd("SET")
                .arg(id)
                .arg(splited[1..].join("_"))
                .query(&mut con)
                .unwrap();
                response = String::from("Send Feedback Text");
            }
            cx.requester.send_message(id, response).await?;
            return Ok(())

        } else if data.starts_with("report_") {
            response = "get report".into();

        } else {
            response = String::from(HELPTEXT.clone());
        }

        cx.requester.send_message(id, response)
                    .reply_markup(ReplyMarkup::InlineKeyboard(
                        InlineKeyboardMarkup::new(
                            vec![
                                vec![
                                    InlineKeyboardButton::callback("Help".into(), "help".into()),
                                    InlineKeyboardButton::callback("My Link".into(), "get_link".into())
                                ]
                            ]
                        )
        )).await?;
    }
    Ok(())
}


#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    
    dotenv().ok();
    let bot_token = env::var("BOT_TOKEN").expect("Error on getting BOT_TOKEN from environment variabls");

    teloxide::enable_logging!();
    log::info!("Starting bot...");

    let bot = Bot::new(bot_token).auto_send();

    // teloxide::repl(bot, callback).await;

    Dispatcher::new(bot)
        .callback_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |message| async move {
                q_callback(message).await;                
            })
        })
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |message| async move {
                callback(message).await;
            })
        })
        .dispatch()
        .await;
}