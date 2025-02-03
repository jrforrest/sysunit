use anyhow::{Result, anyhow};
use async_std::path::PathBuf;
use super::shell_executor::subprocess::{Command, Subprocess};
use crate::models::{FileDependency, Target};

/// Transports a file from the local filesystem to the target by invoking an
/// appropriate command
pub async fn transport_file(file: &FileDependency, target: &Target) -> Result<()> {
    let src_path = PathBuf::from(&file.src);
    if src_path.exists().await {
        let dest_path = PathBuf::from(&file.dest);
        let cmd = match target.proto.as_str() {
            "local" => {
                let cmd = Command {
                    cmd: "cp".to_string(),
                    args: vec![src_path.to_string_lossy().to_string(), dest_path.to_string_lossy().to_string()],
                    env: Default::default(),
                };
                cmd
            },
            "podman" => {
                let cmd = Command {
                    cmd: "podman".to_string(),
                    args: vec![
                        "cp".to_string(),
                        src_path.to_string_lossy().to_string(),
                        format!("{}:{}", target.host, dest_path.to_string_lossy())
                    ],
                    env: Default::default(),
                };
                cmd
            },
            "ssh" => {
                let cmd = Command {
                    cmd: "scp".to_string(),
                    args: vec![
                        src_path.to_string_lossy().to_string(),
                        format!("{}:{}", target.host, dest_path.to_string_lossy())
                    ],
                    env: Default::default(),
                };
                cmd
            },
            _ => {
                return Err(anyhow!("Unsupported transport protocol: {}", target.proto));
            }

        };
        let mut subprocess = Subprocess::init(cmd)?;
        subprocess.close_stdin()?;

        use async_std::io::ReadExt;
        let mut stdout = String::new();
        let mut stderr = String::new();
        let stdout = subprocess.take_stdout().read_to_string(&mut stdout).await?;
        let stderr = subprocess.get_stderr().read_to_string(&mut stderr).await?;

        match subprocess.finalize().await? {
            0 => Ok(()),
            code => Err(anyhow!("Transport failed: exit code {}\nOutput: {}\n{}", code, stderr, stdout)),
        }
    } else {
        Err(anyhow!("Transport failed: File not found: {:?}", src_path))
    }
}
