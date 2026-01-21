use crate::presentation::{Data, Error};

pub async fn handle_framework_error(error: poise::FrameworkError<'_, Data, Error>) {
    let _ = poise::builtins::on_error(error).await;
}

pub fn handle_exec_error(error: Error) {
    tracing::error!("failed to execute discord plan: {:?}", error);
}
