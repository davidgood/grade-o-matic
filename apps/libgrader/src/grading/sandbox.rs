use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

#[derive(Debug, Clone)]
pub struct SandboxExecutionInput {
    pub submission_file: PathBuf,
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub timed_out: bool,
}

#[async_trait]
pub trait SandboxRunner: Send + Sync {
    async fn run(&self, input: &SandboxExecutionInput) -> Result<SandboxExecutionResult>;
}

#[derive(Debug, Clone)]
pub struct DockerSandboxRunner {
    pub image: String,
    pub timeout_secs: u64,
    pub memory_limit: String,
    pub cpus: String,
    pub pids_limit: u32,
}

#[derive(Debug, Clone, Default)]
pub struct NoopSandboxRunner;

#[async_trait]
impl SandboxRunner for NoopSandboxRunner {
    async fn run(&self, input: &SandboxExecutionInput) -> Result<SandboxExecutionResult> {
        let command_preview = if input.command.is_empty() {
            "<none>".to_string()
        } else {
            input.command.join(" ")
        };

        Ok(SandboxExecutionResult {
            exit_code: Some(0),
            stdout: format!(
                "noop sandbox run for file={} cmd={}",
                input.submission_file.display(),
                command_preview
            ),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
        })
    }
}

#[async_trait]
impl SandboxRunner for DockerSandboxRunner {
    async fn run(&self, input: &SandboxExecutionInput) -> Result<SandboxExecutionResult> {
        if input.command.is_empty() {
            bail!("sandbox command must not be empty");
        }

        let parent_dir = input
            .submission_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("submission file has no parent directory"))?;
        ensure_path_exists(parent_dir)?;

        let bind = format!("{}:/workspace/input:ro", parent_dir.display());

        let mut args: Vec<String> = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--network".to_string(),
            "none".to_string(),
            "--read-only".to_string(),
            "--cap-drop".to_string(),
            "ALL".to_string(),
            "--security-opt".to_string(),
            "no-new-privileges".to_string(),
            "--pids-limit".to_string(),
            self.pids_limit.to_string(),
            "--cpus".to_string(),
            self.cpus.clone(),
            "--memory".to_string(),
            self.memory_limit.clone(),
            "-v".to_string(),
            bind,
            "-w".to_string(),
            "/workspace/input".to_string(),
            self.image.clone(),
        ];

        args.extend(input.command.iter().cloned());

        let start = Instant::now();
        match timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new("docker").args(&args).output(),
        )
        .await
        {
            Ok(Ok(out)) => Ok(SandboxExecutionResult {
                exit_code: out.status.code(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                duration_ms: start.elapsed().as_millis(),
                timed_out: false,
            }),
            Ok(Err(err)) => Err(anyhow::Error::from(err)),
            Err(_) => Ok(SandboxExecutionResult {
                exit_code: None,
                stdout: String::new(),
                stderr: "sandbox timed out".to_string(),
                duration_ms: start.elapsed().as_millis(),
                timed_out: true,
            }),
        }
    }
}

pub fn create_sandbox_runner_from_env() -> Arc<dyn SandboxRunner> {
    let mode = std::env::var("GRADER_SANDBOX_MODE").unwrap_or_else(|_| "noop".to_string());

    if mode.eq_ignore_ascii_case("docker") {
        let image = std::env::var("GRADER_SANDBOX_IMAGE")
            .unwrap_or_else(|_| "ghcr.io/your-org/grade-runner:latest".to_string());
        let timeout_secs = std::env::var("GRADER_SANDBOX_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30);
        let memory_limit =
            std::env::var("GRADER_SANDBOX_MEMORY").unwrap_or_else(|_| "512m".to_string());
        let cpus = std::env::var("GRADER_SANDBOX_CPUS").unwrap_or_else(|_| "1.0".to_string());
        let pids_limit = std::env::var("GRADER_SANDBOX_PIDS_LIMIT")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(128);

        Arc::new(DockerSandboxRunner {
            image,
            timeout_secs,
            memory_limit,
            cpus,
            pids_limit,
        })
    } else {
        Arc::new(NoopSandboxRunner)
    }
}

fn ensure_path_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }
    Ok(())
}
