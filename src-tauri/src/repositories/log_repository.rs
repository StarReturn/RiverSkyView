use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::error::AppResult;
use crate::models::project_log::{ActivitySummary, HeatmapCell, ProjectLog};

pub struct LogRepository;

impl LogRepository {
    pub async fn upsert(pool: &SqlitePool, log: &ProjectLog) -> AppResult<()> {
        sqlx::query(
            r#"INSERT INTO project_logs
               (id, project_id, relative_path, content_hash, agent, status, title,
                started_at, finished_at, time_inferred, parse_status, parse_error, indexed_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
               ON CONFLICT(project_id, relative_path) DO UPDATE SET
                content_hash = excluded.content_hash,
                agent = excluded.agent,
                status = excluded.status,
                title = excluded.title,
                started_at = excluded.started_at,
                finished_at = excluded.finished_at,
                time_inferred = excluded.time_inferred,
                parse_status = excluded.parse_status,
                parse_error = excluded.parse_error,
                indexed_at = excluded.indexed_at"#,
        )
        .bind(&log.id)
        .bind(&log.project_id)
        .bind(&log.relative_path)
        .bind(&log.content_hash)
        .bind(&log.agent)
        .bind(&log.status)
        .bind(&log.title)
        .bind(&log.started_at)
        .bind(&log.finished_at)
        .bind(log.time_inferred)
        .bind(&log.parse_status)
        .bind(&log.parse_error)
        .bind(&log.indexed_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_project(
        pool: &SqlitePool,
        project_id: &str,
    ) -> AppResult<Vec<ProjectLog>> {
        let rows = sqlx::query_as::<_, ProjectLog>(
            "SELECT * FROM project_logs WHERE project_id = ?1 ORDER BY finished_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<ProjectLog>> {
        let row = sqlx::query_as::<_, ProjectLog>("SELECT * FROM project_logs WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    pub async fn find_by_relative_path(
        pool: &SqlitePool,
        project_id: &str,
        relative_path: &str,
    ) -> AppResult<Option<ProjectLog>> {
        let row = sqlx::query_as::<_, ProjectLog>(
            "SELECT * FROM project_logs WHERE project_id = ?1 AND relative_path = ?2",
        )
        .bind(project_id)
        .bind(relative_path)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_missing(
        pool: &SqlitePool,
        project_id: &str,
        existing_relative_paths: &[String],
    ) -> AppResult<u64> {
        if existing_relative_paths.is_empty() {
            let result = sqlx::query("DELETE FROM project_logs WHERE project_id = ?1")
                .bind(project_id)
                .execute(pool)
                .await?;
            return Ok(result.rows_affected());
        }

        // 分批删除不存在的日志
        let mut deleted = 0u64;
        let all_logs =
            sqlx::query_as::<_, ProjectLog>("SELECT * FROM project_logs WHERE project_id = ?1")
                .bind(project_id)
                .fetch_all(pool)
                .await?;

        for log in all_logs {
            if !existing_relative_paths.contains(&log.relative_path) {
                let result = sqlx::query("DELETE FROM project_logs WHERE id = ?1")
                    .bind(&log.id)
                    .execute(pool)
                    .await?;
                deleted += result.rows_affected();
            }
        }

        Ok(deleted)
    }

    pub async fn filter(
        pool: &SqlitePool,
        project_id: &str,
        agent: Option<&str>,
        status: Option<&str>,
        date: Option<&str>,
    ) -> AppResult<Vec<ProjectLog>> {
        let mut query = String::from("SELECT * FROM project_logs WHERE project_id = ?1");
        if agent.is_some() {
            query.push_str(" AND agent = ?2");
        }
        if status.is_some() {
            query.push_str(" AND status = ?3");
        }
        if date.is_some() {
            query.push_str(" AND date(finished_at) = ?4");
        }
        query.push_str(" ORDER BY finished_at DESC");

        let mut q = sqlx::query_as::<_, ProjectLog>(&query).bind(project_id);

        if let Some(a) = agent {
            q = q.bind(a);
        }
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(d) = date {
            q = q.bind(d);
        }

        let rows = q.fetch_all(pool).await?;
        Ok(rows)
    }

    pub async fn heatmap(
        pool: &SqlitePool,
        project_id: &str,
        days: i64,
    ) -> AppResult<Vec<HeatmapCell>> {
        let rows = sqlx::query(
            "SELECT date(finished_at) as date, COUNT(*) as count FROM project_logs WHERE project_id = ?1 AND parse_status = 'valid' AND finished_at >= date('now', ?2) GROUP BY date(finished_at) ORDER BY date ASC",
        )
        .bind(project_id)
        .bind(format!("-{} days", days))
        .fetch_all(pool)
        .await?;

        let cells: Vec<HeatmapCell> = rows
            .iter()
            .map(|row| {
                let date: String = row.get("date");
                let count: i64 = row.get("count");
                HeatmapCell {
                    date,
                    count: count as u64,
                }
            })
            .collect();

        Ok(cells)
    }

    pub async fn activity_summary(
        pool: &SqlitePool,
        project_id: &str,
        days: i64,
    ) -> AppResult<ActivitySummary> {
        let heatmap = Self::heatmap(pool, project_id, days).await?;

        let stats: Vec<(String, i64)> = sqlx::query(
            "SELECT status, COUNT(*) as count FROM project_logs WHERE project_id = ?1 AND parse_status = 'valid' AND finished_at >= date('now', ?2) GROUP BY status",
        )
        .bind(project_id)
        .bind(format!("-{} days", days))
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let status: String = row.get("status");
            let count: i64 = row.get("count");
            (status, count)
        })
        .collect();

        let mut completed = 0u64;
        let mut failed = 0u64;
        let mut blocked = 0u64;
        let mut total = 0u64;

        for (status, count) in stats {
            total += count as u64;
            match status.as_str() {
                "completed" => completed += count as u64,
                "failed" => failed += count as u64,
                "blocked" => blocked += count as u64,
                _ => {}
            }
        }

        let now = Utc::now();
        let period_start = (now - chrono::Duration::days(days))
            .format("%Y-%m-%d")
            .to_string();
        let period_end = now.format("%Y-%m-%d").to_string();

        Ok(ActivitySummary {
            total_tasks: total,
            completed,
            failed,
            blocked,
            heatmap,
            period_start,
            period_end,
        })
    }

    pub async fn latest_finished_at(
        pool: &SqlitePool,
        project_id: &str,
    ) -> AppResult<Option<String>> {
        let row = sqlx::query(
            "SELECT finished_at FROM project_logs WHERE project_id = ?1 AND parse_status = 'valid' ORDER BY finished_at DESC LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let finished_at: String = row.get("finished_at");
                Ok(Some(finished_at))
            }
            None => Ok(None),
        }
    }
}
