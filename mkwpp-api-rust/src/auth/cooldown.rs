use sqlx::{FromRow, postgres::PgQueryResult};
use std::net::IpAddr;

use crate::api::{
    errors::{EveryReturnedError, FinalErrorResponse},
    v1::decode_rows_to_table,
};

#[derive(FromRow)]
pub struct LogInAttempts {
    pub ip: IpAddr,
    pub user_id: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl LogInAttempts {
    pub async fn insert(
        executor: &mut sqlx::PgConnection,
        ip: IpAddr,
        user_id: i32,
    ) -> Result<PgQueryResult, FinalErrorResponse> {
        sqlx::query(
            r#"
                INSERT INTO ip_request_throttles (ip, user_id, timestamp)
                VALUES($1, $2, NOW())
            "#,
        )
        .bind(ip)
        .bind(user_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))
    }

    pub fn is_on_cooldown(mut data: Vec<Self>, ip: IpAddr, user_id: i32) -> bool {
        if data.len() < 5 {
            return false;
        }

        data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let latest = data.first().unwrap().timestamp;

        let mut equal_ip = 0;
        let mut equal_user_id = 0;
        for request in data {
            if request.ip == ip {
                equal_ip += 1;
            }
            if request.user_id == user_id {
                equal_user_id += 1;
            }
        }

        let equal_ip = if equal_ip < 5 { 0 } else { equal_ip };
        let equal_user_id = if equal_user_id < 5 { 0 } else { equal_user_id };
        latest.timestamp() + (equal_ip * 30 + equal_user_id * 20) > chrono::Utc::now().timestamp()
    }

    pub async fn get_from_sql(
        executor: &mut sqlx::PgConnection,
        ip: IpAddr,
        user_id: i32,
    ) -> Result<Vec<Self>, FinalErrorResponse> {
        let mut user_data = decode_rows_to_table::<Self>(
            sqlx::query(
                "SELECT * FROM ip_request_throttles WHERE user_id = $1 AND timestamp <= $2",
            )
            .bind(user_id)
            .bind(
                chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - 86400, 0)
                    .unwrap(),
            )
            .fetch_all(&mut *executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )?;

        user_data.extend(decode_rows_to_table::<Self>(
            sqlx::query(
                r"
                SELECT *
                FROM ip_request_throttles
                WHERE
                    ip = $1 AND
                    timestamp >= NOW() - interval '1' day
                ORDER BY timestamp DESC
                ",
            )
            .bind(ip)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )?);

        Ok(user_data)
    }
}
