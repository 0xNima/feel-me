table! {
    users (chat_id) {
        chat_id -> Int8,
        nickname -> Varchar,
        name -> Varchar,
        username -> Nullable<Varchar>,
    }
}


table! {
    histories (id) {
        id -> Int4,
        from_id -> Int8,
        to_id -> Int8,
        history_type -> Int2,
        msg_id -> Int4,
        file_id -> Nullable<Varchar>,
    }
}


table! {
    blacklists (id) {
        id -> Int4,
        reporter_id -> Int8,
        reported_id -> Int8,
        is_active -> Bool,
    }
}