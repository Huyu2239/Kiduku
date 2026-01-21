use sample_discord_bot::presentation::entry::on_error;

#[test]
fn test_handle_exec_error_does_not_panic() {
    let err = anyhow::anyhow!("test error");
    on_error::handle_exec_error(err);
}
