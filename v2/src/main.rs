use v2::{scrape_allergens, scrape_meals};

#[tokio::main]
async fn main() {
    let allergenes = scrape_allergens().await.unwrap();
    dbg!(scrape_meals(v2::Location::Mensa, &allergenes).await);
}
