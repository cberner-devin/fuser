//! macOS mount tests

use std::time::Duration;

use anyhow::Context;
use anyhow::bail;
use tempfile::TempDir;
use tokio::process::Command;

use crate::ansi::green;
use crate::cargo::cargo_build_example;
use crate::command_utils::command_output;
use crate::unmount::Unmount;
use crate::unmount::kill_and_unmount;

pub(crate) async fn run_macos_mount_tests() -> anyhow::Result<()> {
    let mount_dir = TempDir::new().context("Failed to create mount directory")?;
    let mount_path = mount_dir.path().to_str().context("Invalid mount path")?;

    let hello_exe = cargo_build_example("hello", &[]).await?;

    eprintln!("Starting hello filesystem...");
    let mut fuse_process = Command::new(&hello_exe)
        .args([mount_path])
        .kill_on_drop(true)
        .spawn()
        .context("Failed to start hello example")?;

    wait_for_mount("hello", Duration::from_secs(4)).await?;

    let hello_path = mount_dir.path().join("hello.txt");
    let content = wait_for_hello(&hello_path, Duration::from_secs(4)).await?;

    if content == "Hello World!\n" {
        green!("OK with macFUSE");
    } else {
        bail!(
            "hello.txt content mismatch: expected 'Hello World!', got '{}'",
            content
        );
    }

    kill_and_unmount(
        fuse_process,
        Unmount::Manual,
        "hello",
        mount_path,
        "with macFUSE",
    )
    .await?;

    green!("All macOS mount tests passed!");
    Ok(())
}

async fn wait_for_mount(device: &str, timeout: Duration) -> anyhow::Result<()> {
    let start = tokio::time::Instant::now();
    loop {
        let mount_output = command_output(["mount"]).await?;
        if mount_output.contains(device) {
            return Ok(());
        }
        if start.elapsed() > timeout {
            bail!("Timeout waiting for mount with device: {}", device);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_hello(path: &std::path::Path, timeout: Duration) -> anyhow::Result<String> {
    let start = tokio::time::Instant::now();
    loop {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => return Ok(content),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                if start.elapsed() > timeout {
                    return Err(err).context("Failed to read hello.txt");
                }
            }
            Err(err) => return Err(err).context("Failed to read hello.txt"),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
