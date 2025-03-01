use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

mod custom;
mod raw;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1")
        .service(raw::raw())
        .service(custom::custom())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/raw","/custom"]}"#);
}

pub async fn close_connection(
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> Result<(), HttpResponse> {
    return connection.close().await.map_err(|e| {
        HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(crate::api::generate_error_json_string(
                "Error closing Database connection",
                e.to_string().as_str(),
            ))
    });
}

pub fn match_rows(
    rows_request: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
) -> Result<Vec<sqlx::postgres::PgRow>, HttpResponse> {
    return rows_request.map_err(|e| {
        HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(crate::api::generate_error_json_string(
                "Couldn't get rows from database",
                e.to_string().as_str(),
            ))
    });
}

pub fn decode_rows_to_table<Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>>(
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<Vec<Table>, HttpResponse> {
    return rows
        .into_iter()
        .map(|r| return Table::from_row(&r))
        .collect::<Result<Vec<Table>, sqlx::Error>>()
        .map_err(|e| {
            HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Error decoding rows from database",
                    e.to_string().as_str(),
                ))
        });
}

pub fn send_serialized_data<T: serde::Serialize>(data: T) -> HttpResponse {
    match serde_json::to_string(&data) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Error serializing database data",
                    e.to_string().as_str(),
                ))
        }
    }
}

pub async fn handle_basic_get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize,
>(
    rows_request: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> HttpResponse {
    if let Err(e) = close_connection(connection).await {
        return e;
    }

    let rows = match match_rows(rows_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let data = match decode_rows_to_table::<Table>(rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    return send_serialized_data(data);
}
