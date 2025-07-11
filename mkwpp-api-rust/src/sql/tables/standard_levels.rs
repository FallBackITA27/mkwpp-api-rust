use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::decode_rows_to_table,
    },
    app_state::cache::CacheItem,
    sql::tables::BasicTableQueries,
};

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardLevels {
    pub id: i32,
    pub code: String,
    pub value: i32,
    pub is_legacy: bool,
}

impl BasicTableQueries for StandardLevels {
    const TABLE_NAME: &'static str = "standard_levels";
}

impl StandardLevels {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query(
    //         "INSERT INTO standard_levels (id, code, value, is_legacy) VALUES($1, $2, $3, $4);",
    //     )
    //     .bind(self.id)
    //     .bind(&self.code)
    //     .bind(self.value)
    //     .bind(self.is_legacy)
    //     .execute(executor)
    //     .await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data_old")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO standard_levels (id, code, value, is_legacy) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET code = $2, value = $3, is_legacy = $4 WHERE standard_levels.id = $1;").bind(self.id).bind(&self.code).bind(self.value).bind(self.is_legacy).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}

impl CacheItem for StandardLevels {
    type Input = ();

    async fn load(
        executor: &mut sqlx::PgConnection,
        _input: Self::Input,
    ) -> Result<Vec<Self>, FinalErrorResponse>
    where
        Self: Sized,
    {
        decode_rows_to_table::<Self>(
            sqlx::query(
                format!(
                    "SELECT * FROM {this_table} WHERE is_legacy = TRUE;",
                    this_table = Self::TABLE_NAME
                )
                .as_str(),
            )
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )
    }
}
