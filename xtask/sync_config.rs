use tracing::{info, Level};
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    let config = config::TestConfig::parse_config()?;

    info!("Syncing config...");
    let sync_rm_old = &format!("ssh {} rm -f /tmp/debug/up2pd.toml", config.server_ssh_string);
    let sync_rm_old = sync_rm_old.split(" ").collect::<Vec<&str>>();
    let exit_code = tokio::process::Command::new(sync_rm_old[0])
        .args(&sync_rm_old[1..])
        .spawn()
        .unwrap()
        .wait().await.unwrap();
    assert_eq!(exit_code.code(), Some(0));
    let sync_cp_new = &format!("scp ./up2pd.toml.build {}:/tmp/debug/up2pd.toml", config.server_ssh_string);
    let sync_cp_new = sync_cp_new.split(" ").collect::<Vec<&str>>();
    let exit_code = tokio::process::Command::new(sync_cp_new[0])
        .args(&sync_cp_new[1..])
        .spawn()
        .unwrap()
        .wait().await.unwrap();
    assert_eq!(exit_code.code(), Some(0));
    Ok(())
}