use crate::schema::{users, histories, blacklists};

#[derive(Identifiable, Queryable, Insertable, Debug, PartialEq)]
#[table_name="users"]
#[primary_key(chat_id)]
pub struct User {
    pub chat_id: i64,
    pub nickname: String,
    pub name: String,
    pub username: Option<String>
}


#[derive(Queryable, Debug)]
pub struct History {
    pub id: i32,
    pub from_id: i64,
    pub to_id: i64,
    pub history_type: i16,
    pub msg_id: i32,
    pub file_id: Option<String>
}

#[derive(Insertable, Debug)]
#[table_name="histories"]
pub struct NewHistory {
    pub from_id: i64,
    pub to_id: i64,
    pub history_type: i16,
    pub msg_id: i32,
    pub file_id: Option<String>
}


#[derive(Identifiable, Queryable, Associations, Debug, PartialEq)]
#[table_name="blacklists"]
#[belongs_to(User, foreign_key="reporter_id")]
pub struct Blacklist {
    pub id: i32,
    pub reporter_id: i64,
    pub reported_id: i64,
    pub is_active: bool
}


#[derive(Insertable, Debug)]
#[table_name="blacklists"]
pub struct NewBlacklist {
    pub reporter_id: i64,
    pub reported_id: i64,
    pub is_active: bool
}