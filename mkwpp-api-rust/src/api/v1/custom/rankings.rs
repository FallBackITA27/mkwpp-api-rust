use crate::api::errors::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::api::v1::{close_connection, send_serialized_data};
use crate::sql::tables::scores::country_rankings::CountryRankings;
use crate::sql::tables::scores::rankings::{RankingType, Rankings};
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

macro_rules! ranking {
    ($fn_name:ident, $enum_variant:ident, $default_val:expr) => {
        async fn $fn_name(req: HttpRequest) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
            return get(RankingType::$enum_variant($default_val), req).await;
        }
    };
}

pub fn rankings() -> impl HttpServiceFactory {
    web::scope("/rankings")
        .route("/totaltime", web::get().to(total_time))
        .route("/prwr", web::get().to(prwr))
        .route("/tally", web::get().to(tally))
        .route("/af", web::get().to(af))
        .route("/arr", web::get().to(arr))
        .route("/country", web::get().to(country))
        .default_service(web::get().to(default))
}
default_paths_fn!("/af", "/arr", "/tally", "/prwr", "/totaltime");

ranking!(af, AverageFinish, 0.0);
ranking!(arr, AverageRankRating, 0.0);
ranking!(prwr, PersonalRecordWorldRecord, 0.0);
ranking!(tally, TallyPoints, 0);
ranking!(total_time, TotalTime, 0);

async fn get(
    ranking_type: RankingType,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let data = Rankings::get(
        &mut connection,
        ranking_type,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await?;

    close_connection(connection).await?;
    send_serialized_data(data)
}

async fn country(req: HttpRequest) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let data = CountryRankings::get_country_af(
        &mut connection,
        params.category,
        params.lap_mode,
        params.date,
        params.region_type,
        params.limit,
    )
    .await?;

    close_connection(connection).await?;
    send_serialized_data(data)
}
