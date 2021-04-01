lazy_static! {
    pub static ref HELPTEXT: &'static str = "
        Send Music: You can send your music by clicking on user's uniqu link\
        \nGet your own link: You can get your link by click on `Get Link Button`\
        \nAvailable commands:\
            \n\t/start: Start the bot and send music or get your link\
            \n\t/help: See this Desctiption again :)
    ";

    pub static ref WRONG: &'static str = "Something went wrong on server side\nPlease try again";

    pub static ref INVALID_MSG: &'static str = "Invalid Message";

    pub static ref INVALID_MSG_FORMAT: &'static str = "Invalid message format";

    pub static ref BOT_URL: &'static str = "https://t.me/FeelMeBot?start=";

    pub static ref CANCEL_SENDING: &'static str = "Cancel sendig music";

    pub static ref FEEDBACK_BLOCKED: &'static str = "You can't send Feedback\nYou are blocked";

    pub static ref FEEDBACK_SENT: &'static str = "Feedback is sent before";

    pub static ref FEEDBACK_SEND: &'static str = "Send Feedback Text";

    pub static ref USER_REPORTED_BLOCKED: &'static str = "User is reported and blocked";

    pub static ref ERROR_ON_GETTING_USER: &'static str = "Erro on getting Users";

    pub static ref DB_CONN_ERROR: &'static str = "Error on connecting to";

    pub static ref USER_CREATING: &'static str = "Error on creating new user";

    pub static ref HISTORY_CREATING: &'static str = "Error while storing history";

    pub static ref SET_BL_ERR: &'static str = "Error on set to blacklist";

    pub static ref SEARCH_BL_ERR: &'static str = "Error on searching for blacklist";

    pub static ref SEND_MSG_TO_SELF: &'static str = "Dude?! You have schizophrenia\n Why you send music to yourself";

    pub static ref SEND_MUSIC_TEXT: &'static str = "Please Send the music that you want to share with";

    pub static ref BLOCKED_TEXT1: &'static str = "You can't send music to";

    pub static ref BLOCKED_TEXT2: &'static str = "You are blocked";

    pub static ref INVALID_LINK: &'static str = "Oops!\nInvalid link";

    pub static ref INVALID_CMD: &'static str = "Invalid Command";

    pub static ref FEEDBACK_IS_SENT: &'static str = "Feedback is sent";

    pub static ref UNKNOWN_TARGET: &'static str = "The target is unknown\nClick on target user's link and then send the music";

    pub static ref LOG: i16 = 0;

    pub static ref FEEDBACK: i16 = 1;

    pub static ref REPORT: i16 = 2;

    pub static ref UNREPORT: i16 = 3;
}