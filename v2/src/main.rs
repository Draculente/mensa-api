use std::{sync::Arc, time::Duration};

use serde::Serialize;
use tokio::{join, sync::RwLock, time::sleep};
use v2::{Cache, Meal, MealsQuery};
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
    let cache = Arc::new(RwLock::new(
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
        .and(warp::query::<MealsQuery>())
        .and_then({
            let arc_clone = cache.clone();
            move |query| {
                let arc_clone = arc_clone.clone();
                async move {
                    ensure_up_to_date(arc_clone.clone()).await.map_err(|e| {
                        eprint!("{e}");
                        warp::reject::custom(TempError)
                    })?;
                    let cache = arc_clone.read().await;
                    let data = cache.get_data().await.map_err(|e| {
                        eprint!("{e}");
                        warp::reject::custom(TempError)
                    })?;
                    Ok::<Json, warp::Rejection>(reply::json(&MealResponse {
                        last_updated: cache.get_last_update_as_string(),
                        data: data.get_meals_filtered(&query),
                    }))
                }
            }
        });

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

async fn ensure_up_to_date(cache: State) -> anyhow::Result<()> {
    if cache.read().await.needs_update() {
        let mut cache = cache.write().await;
        cache.fetch().await?;
    }
    Ok(())
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
