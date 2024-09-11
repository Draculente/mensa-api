use std::{sync::Arc, time::Duration};

use serde::Serialize;
use tokio::{join, sync::RwLock, time::sleep};
use v2::{APIFilter, Cache, Meal, MealsQuery};
use warp::{
    reject::Reject,
    reply::{self, Json},
    Filter,
};

#[derive(Debug)]
struct TempError;
impl Reject for TempError {}

#[derive(Debug, Serialize)]
struct MealResponse<'a> {
    last_updated: String,
    data: Vec<&'a Meal>,
}

type State = Arc<RwLock<Cache>>;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprint!("{e}")
    }
}

async fn run() -> anyhow::Result<()> {
    let state = Arc::new(RwLock::new(
        Cache::new(chrono::Duration::minutes(10)).await?,
    ));

    // let cache_interval = tokio::spawn({
    //     let cache = cache.clone();
    //     async move {
    //         let cache = cache.clone();
    //         loop {
    //             {
    //                 let mut cache = cache.write().await;
    //                 if let Err(e) = cache.fetch().await {
    //                     eprint!("{e}");
    //                 };
    //             }
    //             // TODO: Extract duration into config
    //             sleep(Duration::from_secs(60)).await;
    //         }
    //     }
    // });

    let meals_route = warp::get()
        .and(warp::path!("v2" / "meals"))
        .and(with_state_and_query_filter::<Meal, MealsQuery>(state))
        // .and(warp::query::<MealsQuery>())
        // .and(with_state(state))
        .and_then(move |(query, state)| meals_handler(query, state));

    // let (_, b) = join!(
    //     warp::serve(meals_route).run(([127, 0, 0, 1], 3030)),
    //     cache_interval
    // );

    warp::serve(meals_route).run(([127, 0, 0, 1], 3030)).await;

    // if let Err(e) = b {
    //     eprint!("{e}");
    // };

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
        cache.fetch().await.map_err(|e| {
            eprint!("{e}");
            warp::reject::custom(TempError)
        })?;
    }
    Ok((filter, cache))
}

async fn meals_handler(
    query: impl APIFilter<Meal>,
    state: State,
) -> Result<impl warp::Reply, warp::Rejection> {
    // ensure_up_to_date(&query, state.clone())
    //     .await
    //     .map_err(|e| {
    //         eprint!("{e}");
    //         warp::reject::custom(TempError)
    //     })?;
    let cache = state.read().await;
    let data = cache.get_data().await.map_err(|e| {
        eprint!("{e}");
        warp::reject::custom(TempError)
    })?;
    Ok::<Json, warp::Rejection>(reply::json(&MealResponse {
        last_updated: cache.get_last_update_as_string(),
        data: data.get_meals_filtered(&query),
    }))
}

/*
https://stackoverflow.com/questions/66111599/how-can-i-achieve-shared-application-state-with-warp-async-routes
https://github.com/seanmonstar/warp/issues/732
#[derive(serde::Deserialize)]
pub struct Params {
    pub date: chrono::NaiveDate,
    pub ids: String,
}

pub async fn handler(params: Params) -> Result<impl warp::Reply, warp::Rejection> {
    let ids = parse_ids(&params.ids);
    // code
}

fn parse_ids(s: &str) -> Vec<i32> {
    s.split(",").map(|id| id.parse::<i32>().unwrap()).collect()
}

let get_posts = warp::get()
    .and(warp::path("posts"))
    .and(warp::query::<Params>())
    .and_then(handler); */
