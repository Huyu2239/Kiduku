pub mod help;

use crate::presentation::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![help::main()]
}
