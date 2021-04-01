#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;

pub mod schema;
pub mod models;
pub mod strings;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid;
use models::{User, History, NewHistory, Blacklist, NewBlacklist};
use strings::{
    HELPTEXT, 
    ERROR_ON_GETTING_USER, 
    DB_CONN_ERROR, 
    USER_CREATING, 
    HISTORY_CREATING, 
    SET_BL_ERR, 
    SEARCH_BL_ERR,
    WRONG,
    SEND_MSG_TO_SELF,
    SEND_MUSIC_TEXT,
    BLOCKED_TEXT1,
    BLOCKED_TEXT2,
    INVALID_LINK,
    INVALID_CMD,
    FEEDBACK_IS_SENT
};

use redis::RedisResult;

pub struct DBManager {
    connection: PgConnection,
}

type Result<T> = std::result::Result<T, ()>;

impl DBManager {
    pub fn new(db_url: &str) -> Result<DBManager> {
        let conn = PgConnection::establish(db_url);
        if conn.is_err(){
            // log &format!("{} {}", *DB_CONN_ERROR, db_url)
            return Err(())
        }
        Ok(DBManager {connection: conn.unwrap()})
    }

    pub fn create_user(&self, id: i64, name_: String, username_: Option<String>) -> Result<User>{
        use schema::users;
        let user = User { 
            chat_id: id, 
            nickname: create_nickname(),
            name: name_,
            username: username_
        };
        let row = diesel::insert_into(users::table)
        .values(&user)
        .get_result(&self.connection);
        if row.is_err() {
            // log *USER_CREATING
            return Err(())
        }
        Ok(row.unwrap())
    }

    pub fn get_users(&self) -> Result<Vec<User>>{
        use schema::users::dsl::*;
        let rows = users.load::<User>(&self.connection);
        if rows.is_err() {
            // log *ERROR_ON_GETTING_USER
            return Err(())
        }
        Ok(rows.unwrap())
    }

    pub fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        use schema::users::dsl::*;
        let user = users
        .filter(chat_id.eq(id))
        .load::<User>(&self.connection);
        if user.is_err(){
            // log *ERROR_ON_GETTING_USER
            return Err(())
        }
        let user = user.unwrap();
        if user.len() != 0 {
            let user = user.get(0).unwrap();
            return Ok(
                Some(User {
                    chat_id: user.chat_id, 
                    nickname: user.nickname.clone(),
                    name: user.name.clone(),
                    username: user.username.clone(),
                })
            )
        }
        Ok(None)
    }

    pub fn get_user_by_nickname(&self, nn: &str) -> Result<Option<User>> {
        use schema::users::dsl::*;
        let user = users
        .filter(nickname.eq(nn))
        .load::<User>(&self.connection);
        if user.is_err(){
            // log *ERROR_ON_GETTING_USER
            return Err(())
        }
        let user = user.unwrap();
        if user.len() != 0 {
            let user = user.get(0).unwrap();
            return Ok(
                Some(User {
                    chat_id: user.chat_id, 
                    nickname: user.nickname.clone(),
                    name: user.name.clone(),
                    username: user.username.clone(),
                })
            )
        }
        Ok(None)
    }

    pub fn set_history(&self, from_id: i64, to_id: i64, history_type: i16, msg_id: i32, file_id: Option<String>) 
    -> Result<()>{
        use schema::histories;
        let history = NewHistory {
            from_id,
            to_id,
            history_type,
            msg_id,
            file_id
        };
        let res = diesel::insert_into(histories::table)
        .values(&history)
        .execute(&self.connection);
        if res.is_err(){
            // log *HISTORY_CREATING
            return Err(())
        }
        Ok(())
    }

    pub fn history_exists(&self, f_id: i64, t_id: i64, m_id: i32, fi_id: String, h_type: i16) -> Result<bool> {
        use schema::histories::dsl::*;

        let history = histories
        .filter(from_id.eq(f_id))
        .filter(to_id.eq(t_id))
        .filter(msg_id.eq(m_id))
        .filter(file_id.eq(fi_id))
        .filter(history_type.eq(h_type))
        .load::<History>(&self.connection);
        if history.is_err() {
            // log *ERROR_ON_GETTING_USER
            return Err(())
        }
        if history.unwrap().len() > 0 {
            return Ok(true)
        }
        Ok(false)
    }

    pub fn set_to_blacklist(&self, rter_id: i64, rted_id: i64) -> Result<()>{
        use schema::blacklists;
        let blacklist = NewBlacklist {
            reporter_id: rter_id,
            reported_id: rted_id,
            is_active: true
        };
        let res = diesel::insert_into(blacklists::table)
        .values(&blacklist)
        .execute(&self.connection);
        if res.is_err() {
            // log *SET_BL_ERR
            return Err(())
        }
        Ok(())
    }

    pub fn is_in_blacklist(&self, reporter: &User, rted_id: i64) -> Result<bool> {
        use schema::blacklists::dsl::*;

        let blacklist = Blacklist::belonging_to(reporter)
        .filter(reported_id.eq(rted_id))
        .load::<Blacklist>(&self.connection);

        if blacklist.is_err() {
            // log *SEARCH_BL_ERR
            return Err(())
        }
        Ok(blacklist.unwrap().len() > 0)
    }
}

fn create_nickname() -> String{
    return Uuid::new_v4().to_string()
}

pub struct Res{
    pub text: Option<String>,
    pub show_cancel_btn: bool,
    pub to_id: Option<String>,
    pub msg_id: Option<i32>,
    pub file_unique_id: Option<String>
}

pub fn handle_text(
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
                        if user.is_err() {
                            return Res {
                                text: Some(String::from(*WRONG)),
                                show_cancel_btn: false,
                                to_id: None,
                                msg_id: None,
                                file_unique_id: None
                            }
                        } 
                        if let Some(user) = user.unwrap() {
                            if user.chat_id == chat_id {
                                response = Some(String::from(*SEND_MSG_TO_SELF));
                            } else {
                                let is_in_blacklist = dbm.is_in_blacklist(&user, chat_id);
                                if is_in_blacklist.is_err() {
                                    return Res {
                                        text: Some(String::from(*WRONG)),
                                        show_cancel_btn: false,
                                        to_id: None,
                                        msg_id: None,
                                        file_unique_id: None
                                    }
                                }
                                if !is_in_blacklist.unwrap() {
                                    let _: () = redis::cmd("SET")
                                        .arg(chat_id)
                                        .arg(user.chat_id)
                                        .query(redis_con)
                                        .unwrap();
                                    return Res {
                                        text: Some(format!("{} {}", *SEND_MUSIC_TEXT, user.name)),
                                        show_cancel_btn: true,
                                        to_id: None,
                                        msg_id: None,
                                        file_unique_id: None
                                    }
                                }
                                response = Some(format!("{} {}\n{}", *BLOCKED_TEXT1, user.name, *BLOCKED_TEXT2));
                            }
                        } else {
                            response = Some(String::from(*INVALID_LINK));
                        }
                    } else {
                        response = Some(String::from(*HELPTEXT));
                    }
                },
                "/help" => {
                    response = Some(String::from(*HELPTEXT));
                }
                _ => { 
                    response = Some(String::from(*INVALID_CMD));
                }
            } 
        } else {
            let result: RedisResult<String> = redis::cmd("GET")
            .arg(chat_id)
            .query(redis_con);

            if let Ok(_text) = result {
                let splited: Vec<&str> = _text.split("_").collect();
                if splited.len() == 3 {
                    let r: RedisResult<i64> = redis::cmd("DEL")
                    .arg(chat_id)
                    .query(redis_con);
                    if r.is_ok() {
                        return Res {
                            text: Some(String::from(*FEEDBACK_IS_SENT)),
                            show_cancel_btn: false,
                            to_id: Some(String::from(splited[0])),
                            msg_id: Some(splited[1].parse::<i32>().unwrap()),
                            file_unique_id: Some(String::from(splited[2]))
                        }
                    }
                }
            }
            response = None;
        }
    Res { text: response, show_cancel_btn: false, to_id: None, msg_id: None, file_unique_id: None }
}


#[cfg(test)]
mod tests {
    extern crate dotenv;
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn is_in_blacklist() {
        dotenv().ok();
        let databse_url = env::var("DATABASE_URL").expect("Error on getting DATABASE_URL from environment variabls");
        let db_manager = DBManager::new(&databse_url).unwrap();
        let res = db_manager.is_in_blacklist(&User { 
            chat_id: 100000, 
            nickname: "nickname".into(), 
            name: "name".into(), 
            username: None
        }, 100000).unwrap();
        assert_eq!(res, false);
    }

    #[test]
    fn set_to_blacklist() {
        assert_eq!(false, true);
    }
}