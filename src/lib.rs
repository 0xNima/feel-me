#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid;
use models::{User, History, NewHistory, Blacklist, NewBlacklist};

pub struct DBManager {
    connection: PgConnection,
}

impl DBManager {
    pub fn new(db_url: &str) -> DBManager {
        DBManager {
            connection: PgConnection::establish(db_url)
            .expect(&format!("Error connecting to {}", db_url))
        }
    }

    pub fn create_user(&self, id: i64, name_: String, username_: Option<String>) -> User{
        use schema::users;
        let user = User { 
            chat_id: id, 
            nickname: create_nickname(),
            name: name_,
            username: username_
        };
        diesel::insert_into(users::table)
        .values(&user)
        .get_result(&self.connection)
        .expect("Error on creating new user")
    }

    pub fn get_users(&self) -> Vec<User>{
        use schema::users::dsl::*;
        users.load::<User>(&self.connection).expect("Erro on getting Users")
    }

    pub fn get_user_by_id(&self, id: i64) -> Option<User> {
        use schema::users::dsl::*;
        let user = users
        .filter(chat_id.eq(id))
        .load::<User>(&self.connection)
        .expect("Erro on getting Users");

        if user.len() != 0 {
            let user = user.get(0).unwrap();
            Some(User {
                chat_id: user.chat_id, 
                nickname: user.nickname.clone(),
                name: user.name.clone(),
                username: user.username.clone(),
            })
        } else {
            None
        }
    }

    pub fn get_user_by_nickname(&self, nn: &str) -> Option<User> {
        use schema::users::dsl::*;
        let user = users
        .filter(nickname.eq(nn))
        .load::<User>(&self.connection)
        .expect("Erro on getting Users");

        if user.len() != 0 {
            let user = user.get(0).unwrap();
            Some(User {
                chat_id: user.chat_id, 
                nickname: user.nickname.clone(),
                name: user.name.clone(),
                username: user.username.clone(),
            })
        } else {
            None
        }
    }

    pub fn set_history(&self, from_id: i64, to_id: i64, history_type: i16, msg_id: i32, file_id: Option<String>) {
        use schema::histories;
        let history = NewHistory {
            from_id,
            to_id,
            history_type,
            msg_id,
            file_id
        };
        diesel::insert_into(histories::table)
        .values(&history)
        .execute(&self.connection)
        .expect("Error while storing history");
    }

    pub fn history_exists(&self, f_id: i64, t_id: i64, m_id: i32, fi_id: String, h_type: i16) -> bool {
        use schema::histories::dsl::*;

        let history = histories
        .filter(from_id.eq(f_id))
        .filter(to_id.eq(t_id))
        .filter(msg_id.eq(m_id))
        .filter(file_id.eq(fi_id))
        .filter(history_type.eq(h_type))
        .load::<History>(&self.connection)
        .expect("Erro on getting Users");
 
        if history.len() > 0 {
            return true
        }
        false
    }

    pub fn set_to_blacklist(&self, rter_id: i64, rted_id: i64) {
        use schema::blacklists;
        let blacklist = NewBlacklist {
            reporter_id: rter_id,
            reported_id: rted_id,
            is_active: true
        };
        diesel::insert_into(blacklists::table)
        .values(&blacklist)
        .execute(&self.connection)
        .expect("Error on set to blacklist");
    }

    pub fn is_in_blacklist(&self, reporter: &User, rted_id: i64) -> bool {
        use schema::blacklists::dsl::*;

        let blacklist = Blacklist::belonging_to(reporter)
        .filter(reported_id.eq(rted_id))
        .load::<Blacklist>(&self.connection)
        .unwrap_or(Vec::new());

        blacklist.len() > 0
    }
}

fn create_nickname() -> String{
    return Uuid::new_v4().to_string()
}

lazy_static! {
    pub static ref HELPTEXT: String = String::from("
        Send Music: You can send your music by clicking on user's uniqu link\
        \nGet your own link: You can get your link by click on `Get Link Button`\
        \nAvailable commands:\
            \n\t/start: Start the bot and send music or get your link\
            \n\t/help: See this Desctiption again :)
    ");

    pub static ref LOG: i16 = 0;

    pub static ref FEEDBACK: i16 = 1;

    pub static ref REPORT: i16 = 2;

    pub static ref UNREPORT: i16 = 3;
}

pub struct Res{
    pub text: Option<String>,
    pub show_cancel_btn: bool,
    pub to_id: Option<String>,
    pub msg_id: Option<i32>,
    pub file_unique_id: Option<String>
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
        let db_manager = DBManager::new(&databse_url);
        let res = db_manager.is_in_blacklist(&User { 
            chat_id: 100000, 
            nickname: "nickname".into(), 
            name: "name".into(), 
            username: None
        }, 100000);
        assert_eq!(res, false);
    }

    #[test]
    fn set_to_blacklist() {
        assert_eq!(false, true);
    }
}