use crate::{
    api::errors::FinalErrorResponse,
    sql::tables::{
        BasicTableQueries,
        regions::{
            RegionType, Regions,
            tree::{ChildrenTree, generate_region_tree},
            with_player_count::RegionsWithPlayerCount,
        },
    },
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};
use std::collections::HashMap;

macro_rules! region_fn {
    ($fn_name:ident, $handle:expr) => {
        async fn $fn_name(
            path: web::Path<i32>,
        ) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
            return basic_get_i32(path, $handle).await;
        }
    };
}

pub fn regions() -> impl HttpServiceFactory {
    web::scope("/regions")
        .service(web::scope("/ancestors/{region_id}").default_service(web::get().to(get_ancestors)))
        .service(
            web::scope("/descendants/{region_id}").default_service(web::get().to(get_descendants)),
        )
        .route("/type_hashmap", web::get().to(get_region_type_hashmap))
        .route("/descendence_tree", web::get().to(get_region_child_tree))
        .route("/with_player_count", web::get().to(get_with_player_count))
        .default_service(web::get().to(default))
}

default_paths_fn!(
    "/ancestors/:regionId",
    "/descendants/:regionId",
    "/type_hashmap",
    "/with_player_count",
    "/descendence_tree"
);

region_fn!(get_ancestors, Regions::get_ancestors);
region_fn!(get_descendants, Regions::get_descendants);

async fn get_region_type_hashmap() -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    crate::api::v1::basic_get_with_data_mod::<Regions, HashMap<RegionType, Vec<i32>>>(
        Regions::select_star_query,
        async |data: &[Regions]| {
            let mut hashmap: HashMap<RegionType, Vec<i32>> = HashMap::new();
            hashmap.insert(RegionType::World, vec![]);
            hashmap.insert(RegionType::Continent, vec![]);
            hashmap.insert(RegionType::Country, vec![]);
            hashmap.insert(RegionType::CountryGroup, vec![]);
            hashmap.insert(RegionType::Subnational, vec![]);
            hashmap.insert(RegionType::SubnationalGroup, vec![]);

            for region in data {
                hashmap
                    .get_mut(&region.region_type)
                    .expect("A RegionType is missing from get_region_type_hashmap")
                    .push(region.id);
            }

            hashmap
        },
    )
    .await
}

// TODO: rewrite more optimally
async fn get_with_player_count() -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    crate::api::v1::basic_get_with_data_mod::<RegionsWithPlayerCount, Vec<RegionsWithPlayerCount>>(
        RegionsWithPlayerCount::select_star_query,
        RegionsWithPlayerCount::collapse_counts_of_regions,
    )
    .await
}

pub async fn basic_get_i32(
    path: web::Path<i32>,
    rows_function: impl AsyncFnOnce(
        &mut sqlx::PgConnection,
        i32,
    ) -> Result<Vec<i32>, FinalErrorResponse>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let rows = rows_function(&mut connection, path.into_inner()).await?;
    crate::api::v1::close_connection(connection).await?;

    crate::api::v1::send_serialized_data(rows)
}

async fn get_region_child_tree() -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    crate::api::v1::basic_get_with_data_mod::<Regions, ChildrenTree>(
        Regions::select_star_query,
        generate_region_tree,
    )
    .await
}
