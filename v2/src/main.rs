use v2::scrape_allergens;

#[tokio::main]
async fn main() {
    dbg!(scrape_allergens(v2::Location::Mensa).await);
}
