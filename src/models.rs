use crate::schema::{users, histories};

#[derive(Queryable, Insertable, Debug)]
#[table_name="users"]
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