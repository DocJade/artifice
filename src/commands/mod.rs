mod ping;
use crate::commands::ping::ping;

// return the commands in this folder.
pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        // todo
        ping()
    ]
}
