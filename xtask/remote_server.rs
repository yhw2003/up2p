use config::TestConfig;
use tokio::signal;
use tracing::{info, Level};
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    let config = TestConfig::parse_config()?;
    let _ = tokio::process::Command::new("ssh")
        .args([&config.server_ssh_string, "pkill", "-f", "server"])
        .spawn()
        .unwrap()
        .wait().await.unwrap();
    info!("Building...");
    let exit_code = tokio::process::Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("server")
        .spawn()
        .unwrap()
        .wait().await.unwrap();
    assert_eq!(exit_code.code(), Some(0));
    info!("Deploying...");
    
    let exit_code = tokio::process::Command::new("scp")
        .args([
            "./target/x86_64-unknown-linux-musl/debug/server",
            &format!("{}:/tmp/debug/tmp_server", &config.server_ssh_string),
        ])
        .spawn()
        .unwrap()
        .wait().await.unwrap();
    assert_eq!(exit_code.code(), Some(0));
    info!("running...");
    let mut run_exit = tokio::process::Command::new("ssh")
        .args([
            &config.server_ssh_string,
            "cd /tmp/debug && ./tmp_server",
        ])
        .spawn()
        .unwrap();
    // send ctrl-c to the server
    let cc = signal::ctrl_c();
    loop {
        tokio::select! {
            _ = cc => {
                run_exit.kill().await?;
                let _ = tokio::process::Command::new("ssh")
                    .args([&config.server_ssh_string, "pkill", "-f", "server"])
                    .spawn()
                    .unwrap()
                    .wait().await.unwrap();
                info!("Server killed");
                break;
            },
            _ = run_exit.wait() => {
                break;
            }
        }
    }
    Ok(())
}