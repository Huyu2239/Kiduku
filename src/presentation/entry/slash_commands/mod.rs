pub mod check_reads;
pub mod help;

use crate::presentation::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![help::main(), check_reads::main()]
}
