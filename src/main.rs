use std::{convert::Infallible, sync::Arc};

use serde::Serialize;
use tokio::sync::RwLock;

use envconfig::Envconfig;
use mensa_api::api_filter::{APIFilter, AllergensQuery, LocationsQuery, MealsQuery};
use mensa_api::cache::Cache;
use mensa_api::config::Config;
use mensa_api::model::{APILocation, Allergen, Data, Meal};
use warp::http::StatusCode;
use warp::{
    reject::{Reject, Rejection},
    reply::{self, Json, Reply},
    Filter,
};

#[derive(Debug)]
struct APIError(anyhow::Error);
impl Reject for APIError {}

impl APIError {
    async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
        let code;
        let message: String;

        if err.is_not_found() {
            code = StatusCode::NOT_FOUND;
            message = "Not Found".into();
        } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
            code = StatusCode::BAD_REQUEST;
            message = "Invalid Body".into();
        } else if let Some(e) = err.find::<warp::reject::InvalidQuery>() {
            code = StatusCode::BAD_REQUEST;
            message = e.to_string();
        } else if let Some(e) = err.find::<anyhow::Error>() {
            eprint!("{e}");
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "Internal Server Error".into()
        } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
            code = StatusCode::METHOD_NOT_ALLOWED;
            message = "Method Not Allowed".into();
        } else {
            eprintln!("unhandled error: {:?}", err);
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "Internal Server Error".into();
        }

        let json = warp::reply::json(&ErrorResponse {
            message: message.into(),
        });

        Ok(warp::reply::with_status(json, code))
    }
}

fn custom_reject(error: impl Into<anyhow::Error>) -> warp::Rejection {
    warp::reject::custom(APIError(error.into()))
}

#[derive(Debug, Serialize)]
struct DefaultResponse<'a, T> {
    last_updated: String,
    data: Vec<&'a T>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

type State = Arc<RwLock<Cache>>;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprint!("{e}")
    }
}

async fn run() -> anyhow::Result<()> {
    let config = Config::init_from_env()?;
    let state = Arc::new(RwLock::new(Cache::new(chrono::Duration::seconds(
        config.ttl as i64,
    ))?));

    let meals_route = warp::path!("v2" / "meals")
        .and(with_state_and_query_filter::<Meal, MealsQuery>(
            state.clone(),
        ))
        .and_then(move |(query, state)| default_handler(query, state, |d| d.get_meals()));

    let allergens_route = warp::path!("v2" / "allergenes")
        .or(warp::path!("v2" / "allergens"))
        .and(with_state_and_query_filter::<Allergen, AllergensQuery>(
            state.clone(),
        ))
        .and_then(move |_, (query, state)| default_handler(query, state, |d| d.get_allergens()));

    let locations_route = warp::path!("v2" / "locations")
        .and(with_state_and_query_filter::<APILocation, LocationsQuery>(
            state.clone(),
        ))
        .and_then(move |(query, state)| default_handler(query, state, |d| d.get_locations()));

    let routes = meals_route
        .or(allergens_route)
        .or(locations_route)
        .with(warp::cors().allow_any_origin())
        .and(warp::get())
        .recover(APIError::handle_rejection);

    println!("Server starting on port {}", config.port);
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;

    Ok(())
}

fn with_state_and_query_filter<A, T: APIFilter<A> + 'static>(
    state: State,
) -> impl Filter<Extract = ((impl APIFilter<A>, State),), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::query::<T>())
        .and(with_state(state))
        .and_then(ensure_up_to_date)
}

fn with_state(
    state: State,
) -> impl Filter<Extract = (State,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn ensure_up_to_date<T>(
    filter: impl APIFilter<T>,
    cache: State,
) -> Result<(impl APIFilter<T>, State), warp::Rejection> {
    if cache.read().await.needs_update() {
        let mut cache = cache.write().await;
        cache.fetch().await.map_err(custom_reject)?;
    }
    Ok((filter, cache))
}

async fn default_handler<T: Serialize, F>(
    query: impl APIFilter<T>,
    state: State,
    data_fn: F,
) -> Result<impl warp::Reply, warp::Rejection>
where
    F: Fn(&Data) -> &Vec<T>,
{
    let cache = state.read().await;
    let data = cache.get_data().await.map_err(custom_reject)?;
    Ok::<Json, warp::Rejection>(reply::json(&DefaultResponse {
        last_updated: cache.get_last_update_as_string(),
        data: query.filter(data_fn(data)),
    }))
}
