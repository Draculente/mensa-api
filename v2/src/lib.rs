use core::fmt;

use anyhow::anyhow;
use scraper::Selector;

#[derive(Debug, Clone)]
pub struct Allergene {
    code: String,
    name: String,
}

#[derive(Debug, Clone)]
struct APILocation {
    code: String,
    name: String,
}

#[derive(Debug, Clone)]
struct Meal {
    name: String,
    price: String,
    vegan: bool,
    vegetarian: bool,
    location: APILocation,
    allergens: Vec<Allergene>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Musikhochschule,
    Cafeteria,
    Mensa,
}

impl Location {
    /// The speiseplan website uses number codes to differentiate between locations.
    /// This methods the Location translates into these codes.
    fn to_url_code(&self) -> usize {
        match self {
            Location::Musikhochschule => 9,
            Location::Cafeteria => 8,
            Location::Mensa => 8,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            Location::Musikhochschule => "Cafeteria Musikhochschule",
            Location::Cafeteria => "Cafeteria Hauptcampus",
            Location::Mensa => "Mensa",
        };
        write!(f, "{s}")
    }
}

pub async fn scrape_meals(location: Location) -> anyhow::Result<Vec<Meal>> {
    let weeks = 0..2;

    todo!()
}

async fn scrape_meals_of_week(location: Location, week: usize) -> anyhow::Result<Vec<Meal>> {
    let url = format!(
        "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa={}&nw={}#mensaplan",
        location.to_url_code(),
        week
    );

    todo!()
}

pub async fn scrape_allergens(location: Location) -> anyhow::Result<Vec<Allergene>> {
    let url = format!(
        "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa={}&nw=0#mensaplan",
        location.to_url_code()
    );

    let html = reqwest::get(url).await?.text().await?;

    let document = scraper::Html::parse_document(&html);

    let parent_element_selector = Selector::parse(".mbf_content").expect("Selector failed");
    let parent_element = document
        .select(&parent_element_selector)
        .next()
        .ok_or(anyhow!("Failed to get the allergene parent element"))?;

    let allergenes: Vec<Allergene> = parent_element
        .child_elements()
        .map(|e| -> Option<Allergene> {
            let code = e.attr("data-wert")?.to_string();
            let name = e.child_elements().skip(1).next()?.inner_html();
            Some(Allergene { code, name })
        })
        .filter_map(|a| a)
        .collect();

    Ok(allergenes)
}
