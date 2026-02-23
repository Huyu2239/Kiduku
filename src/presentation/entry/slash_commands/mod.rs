pub mod help;
pub mod my_mentions;
pub mod view_read_status;

use crate::presentation::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![help::main(), view_read_status::main(), my_mentions::main()]
}
