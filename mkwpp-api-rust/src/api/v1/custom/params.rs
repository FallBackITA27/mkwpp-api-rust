use crate::sql::tables::{Category, regions::RegionType};

#[derive(serde::Deserialize, Debug)]
pub struct Params {
    cat: Option<u8>,
    lap: Option<u8>,
    dat: Option<String>,
    reg: Option<i32>,
    lim: Option<i32>,
    rty: Option<u8>,
}

pub struct ParamsDestructured {
    pub category: Category,
    pub lap_mode: Option<bool>,
    pub date: chrono::NaiveDate,
    pub region_id: i32,
    pub limit: i32,
    pub region_type: RegionType,
}

impl ParamsDestructured {
    pub fn from_query(params: actix_web::web::Query<Params>) -> Self {
        let params = params.into_inner();
        ParamsDestructured {
            category: params
                .cat
                .and_then(|x| Category::try_from(x).ok())
                .unwrap_or(Category::NonSc),
            date: params
                .dat
                .and_then(|x| chrono::NaiveDate::parse_from_str(&x, "%F").ok())
                .unwrap_or(chrono::Local::now().date_naive()),
            region_id: params.reg.unwrap_or(1),
            lap_mode: params.lap.map(|x| x == 1),
            limit: params.lim.unwrap_or(i32::MAX),
            region_type: params
                .rty
                .and_then(|x| RegionType::try_from(x).ok())
                .unwrap_or(RegionType::World),
        }
    }
}
