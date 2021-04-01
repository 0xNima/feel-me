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
use feel_me::{DBManager, Res, handle_text, strings::*};
use redis::{Commands, RedisResult};


async fn callback(
    cx: UpdateWithCx<AutoSend<Bot>, Message>
) -> Result<(), Box<dyn Error + Send + Sync>> {

    use feel_me::schema::users::dsl::*;

    let answer_str: String; 
    
    let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
    
    let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");
    
    let db_manager = DBManager::new(&databse_url);
    
    let client = redis::Client::open(redis_url);

    if db_manager.is_err() || client.is_err() {
        cx.answer(*WRONG).await?;
        return Ok(())
    }

    let con = client.unwrap().get_connection();

    if con.is_err() {
        cx.answer(*WRONG).await?;
        return Ok(())
    }

    let db_manager = db_manager.unwrap();

    let mut con = con.unwrap();
    
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
            }
             
            if let Some(to_id) = res.to_id {
                let msg_id = res.msg_id.unwrap();

                cx.requester.send_message(to_id.clone(), text).reply_to_message_id(msg_id).await?;

                let res = db_manager.set_history(cx.chat_id(), to_id.parse::<i64>().unwrap(), *FEEDBACK, msg_id, res.file_unique_id);

                if res.is_err() {
                    // log error
                }

                cx.requester.send_message(cx.chat_id(), response).await?;
                return Ok(())
            }
            answer_str = response;
        } else {
            answer_str = String::from(*INVALID_MSG);
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
                            InlineKeyboardButton::callback("Report User".into(), format!("report_{}_{}", cx.chat_id(), cx.update.id))
                        ]
                    ]
                )
            )).await?;

            db_manager.set_history(cx.chat_id(), to_id, *LOG, cx.update.id, Some(audio.file_unique_id.clone()));

            cx.requester.send_message(cx.chat_id(), "Music Has Been send").await?;

            let del: RedisResult<()> = redis::cmd("DEL")
                               .arg(cx.chat_id())
                               .query(&mut con);
            if del.is_err() {
                // log error
            }
            return Ok(())
        } else {
            answer_str = String::from(*UNKNOWN_TARGET);
        }
    } else {
        answer_str = String::from(*INVALID_MSG_FORMAT);
    }

    cx.answer(answer_str).reply_markup(ReplyMarkup::InlineKeyboard(
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
        let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
        let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");

        let id = cx.update.from.id;
        let name_ = cx.update.from.full_name();
        let username_ = cx.update.from.username;
        let response;

        if data == "get_link" {
            use feel_me::schema::users::dsl::*;

            let db_manager = DBManager::new(&databse_url);

            if db_manager.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let db_manager = db_manager.unwrap();

            let _user = db_manager.get_user_by_id(id);

            if _user.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let user_;
            
            if let Some(user) = _user.unwrap() {
                user_ = user
            } else {
                let _user = db_manager.create_user(id, name_, username_);
             
                if _user.is_err() {
                    cx.requester.send_message(id, *WRONG).await?;
                    return Ok(())
                }

                user_ = _user.unwrap();
            }
            response = format!("{}{}", *BOT_URL, user_.nickname);

        } else if data == "cancel" {
            let redis_url = env::var("REDIS_URL").expect("Error on getting REDIS_URL from environment variabls");
            
            let client = redis::Client::open(redis_url);

            if client.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }
        
            let con = client.unwrap().get_connection();
            
            if con.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let mut con = con.unwrap();

            let del: RedisResult<()> = redis::cmd("DEL")
            .arg(id)
            .query(&mut con);

            if del.is_err() {
                // log here
            }
            
            cx.requester.send_message(id, *CANCEL_SENDING).await?;
            
            return Ok(())

        } else if data.starts_with("feedback_") {
            let splited: Vec<&str> = data.split("_").collect();

            let to_id = splited[1].parse::<i64>().unwrap();
            
            let db_manager = DBManager::new(&databse_url);
            
            if db_manager.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let db_manager = db_manager.unwrap();

            let target_user = db_manager.get_user_by_id(to_id);

            if target_user.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let target_user = target_user.unwrap();

            let is_in_blacklist;

            if let Some(t_user) = target_user {
                is_in_blacklist = db_manager.is_in_blacklist(&t_user, id);  

                if is_in_blacklist.is_err() {
                    cx.requester.send_message(id, *WRONG).await?;
                    return Ok(())
                }
            } else {
                is_in_blacklist = Ok(false);
            }
        
            if is_in_blacklist.unwrap() {
                response = String::from(*FEEDBACK_BLOCKED);
            } else {
                let history_exists = db_manager
                .history_exists(id, to_id, splited[2].parse::<i32>().unwrap(), String::from(splited[3]), *FEEDBACK);
                

                if history_exists.is_err() {
                    cx.requester.send_message(id, *WRONG).await?;
                    return Ok(())
                }

                if history_exists.unwrap() {
                    response = String::from(*FEEDBACK_SENT);
                } else {
                    let client = redis::Client::open(redis_url);
                 
                    if client.is_err() {
                        cx.requester.send_message(id, *WRONG).await?;
                        return Ok(())
                    }

                    let con = client.unwrap().get_connection();

                    if con.is_err() {
                        cx.requester.send_message(id, *WRONG).await?;
                        return Ok(())
                    }

                    let mut con = con.unwrap();

                    let res: RedisResult<()> = redis::cmd("SET")
                    .arg(id)
                    .arg(splited[1..].join("_"))
                    .query(&mut con);

                    if res.is_err() {
                        cx.requester.send_message(id, *WRONG).await?;
                        return Ok(())
                    }

                    response = String::from(*FEEDBACK_SEND);
                }
            }
            cx.requester.send_message(id, response).await?;
            return Ok(())

        } else if data.starts_with("report_") {
            let splited: Vec<&str> = data.split("_").collect();

            let to_id = splited[1].parse::<i64>().unwrap();

            let msg_id = splited[2].parse::<i32>().unwrap();

            let db_manager = DBManager::new(&databse_url);

            if db_manager.is_err() {
                cx.requester.send_message(id, *WRONG).await?;
                return Ok(())
            }

            let db_manager = db_manager.unwrap();

            let set_to_blacklist = db_manager.set_to_blacklist(id, to_id);

            let set_history = db_manager.set_history(id, to_id, *REPORT, msg_id, None);


            if set_to_blacklist.is_err() || set_history.is_err() {
                // log error
            }

            response = String::from(*USER_REPORTED_BLOCKED);

        } else {
            response = String::from(*HELPTEXT);
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
