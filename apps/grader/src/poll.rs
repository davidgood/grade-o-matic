use libgrader::common::config::Config;
use libgrader::grading::sandbox::{
    SandboxExecutionInput, SandboxRunner, create_sandbox_runner_from_env,
};
use sqlx::{FromRow, PgPool};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{Duration, MissedTickBehavior};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
struct GradingJob {
    id: Uuid,
    assignment_id: Uuid,
    file_id: Uuid,
    submitted_by: Option<Uuid>,
    attempt_count: i32,
}

#[derive(Debug, Clone, FromRow)]
struct UploadedFileRow {
    file_relative_path: String,
}

fn is_missing_table(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().as_deref() == Some("42P01"),
        _ => false,
    }
}

async fn load_submission_file(
    pool: &PgPool,
    file_id: Uuid,
    assets_private_path: &str,
) -> Result<PathBuf, anyhow::Error> {
    let row = sqlx::query_as::<_, UploadedFileRow>(
        r#"
        SELECT file_relative_path
        FROM uploaded_files
        WHERE id = $1
        "#,
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| anyhow::anyhow!("uploaded file metadata not found"))?;
    Ok(PathBuf::from(assets_private_path).join(row.file_relative_path))
}

async fn claim_next_job(pool: &PgPool) -> Result<Option<GradingJob>, sqlx::Error> {
    sqlx::query_as::<_, GradingJob>(
        r#"
        WITH candidate AS (
            SELECT id
            FROM grading_jobs
            WHERE status = 'queued'
            ORDER BY created_at ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE grading_jobs g
        SET
            status = 'running',
            locked_at = NOW(),
            started_at = COALESCE(g.started_at, NOW()),
            attempt_count = g.attempt_count + 1,
            updated_at = NOW()
        FROM candidate
        WHERE g.id = candidate.id
        RETURNING g.id, g.assignment_id, g.file_id, g.submitted_by, g.attempt_count
        "#,
    )
    .fetch_optional(pool)
    .await
}

async fn mark_completed(pool: &PgPool, job_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE grading_jobs
        SET status = 'completed',
            completed_at = NOW(),
            error_message = NULL,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_failed(pool: &PgPool, job_id: Uuid, message: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE grading_jobs
        SET status = 'failed',
            error_message = $2,
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(())
}

async fn process_job(
    job: &GradingJob,
    pool: &PgPool,
    config: &Config,
    runner: &Arc<dyn SandboxRunner>,
) -> anyhow::Result<()> {
    let submission_file =
        load_submission_file(pool, job.file_id, &config.assets_private_path).await?;
    let command = std::env::var("GRADER_SANDBOX_COMMAND")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "ls -la && echo grading-placeholder".to_string());

    let exec = runner
        .run(&SandboxExecutionInput {
            submission_file: submission_file.clone(),
            command: vec!["/bin/sh".to_string(), "-lc".to_string(), command],
        })
        .await?;

    info!(
        job_id = %job.id,
        assignment_id = %job.assignment_id,
        file_id = %job.file_id,
        submission_file = %submission_file.display(),
        submitted_by = ?job.submitted_by,
        attempt = job.attempt_count,
        exit_code = ?exec.exit_code,
        timed_out = exec.timed_out,
        duration_ms = exec.duration_ms,
        "processed grading job in sandbox"
    );

    if !exec.stdout.is_empty() {
        info!(job_id = %job.id, stdout = %exec.stdout, "sandbox stdout");
    }
    if !exec.stderr.is_empty() {
        warn!(job_id = %job.id, stderr = %exec.stderr, "sandbox stderr");
    }

    if exec.timed_out || exec.exit_code.unwrap_or(1) != 0 {
        anyhow::bail!(
            "sandbox execution failed (timed_out={}, exit_code={:?})",
            exec.timed_out,
            exec.exit_code
        );
    }

    Ok(())
}

async fn poll_once(pool: &PgPool, config: &Config, runner: &Arc<dyn SandboxRunner>) {
    match claim_next_job(pool).await {
        Ok(Some(job)) => {
            let job_id = job.id;
            match process_job(&job, pool, config, runner).await {
                Ok(()) => {
                    if let Err(err) = mark_completed(pool, job_id).await {
                        error!(job_id = %job_id, error = %err, "failed to mark job completed");
                    }
                }
                Err(err) => {
                    error!(job_id = %job_id, error = %err, "job processing failed");
                    if let Err(mark_err) = mark_failed(pool, job_id, &err.to_string()).await {
                        error!(
                            job_id = %job_id,
                            error = %mark_err,
                            "failed to mark job failed"
                        );
                    }
                }
            }
        }
        Ok(None) => {
            debug!("no queued grading jobs");
        }
        Err(err) if is_missing_table(&err) => {
            warn!("grading_jobs table missing; poller is idle until migration is applied");
        }
        Err(err) => {
            error!(error = %err, "failed to claim grading job");
        }
    }
}

pub async fn run(pool: PgPool, config: Config) -> anyhow::Result<()> {
    let runner = create_sandbox_runner_from_env();
    let interval_ms = std::env::var("GRADER_POLL_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000);
    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    info!(interval_ms, "grader polling loop started");

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("grader polling loop received shutdown signal");
                break;
            }
            _ = ticker.tick() => {
                poll_once(&pool, &config, &runner).await;
            }
        }
    }

    Ok(())
}
