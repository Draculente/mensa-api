use core::fmt;

use anyhow::anyhow;
use futures::future::join_all;
use htmlentity::entity::decode;
use htmlentity::entity::ICodedDataTrait;
use regex::Regex;
use scraper::Html;
use scraper::Selector;

#[derive(Debug, Clone)]
pub struct Allergene {
    code: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct APILocation {
    code: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct Meal {
    name: String,
    date: String,
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

impl Into<APILocation> for Location {
    fn into(self) -> APILocation {
        match self {
            Location::Musikhochschule => APILocation {
                code: "MH".to_string(),
                name: "Musikhochschule".to_string(),
            },
            Location::Cafeteria => APILocation {
                code: "CA".to_string(),
                name: "Cafeteria".to_string(),
            },
            Location::Mensa => APILocation {
                code: "ME".to_string(),
                name: "Mensa".to_string(),
            },
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

pub async fn scrape_meals(
    location: Location,
    allergenes: &Vec<Allergene>,
) -> anyhow::Result<Vec<Meal>> {
    // 0,1
    let weeks = 0..2;

    let futures = weeks.map(|week| scrape_meals_of_week(location, week, allergenes));

    let vecs_of_meals = join_all(futures)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(vecs_of_meals.into_iter().flatten().collect())
}

async fn scrape_meals_of_week(
    location: Location,
    week: usize,
    allergenes: &Vec<Allergene>,
) -> anyhow::Result<Vec<Meal>> {
    let url = format!(
        "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa={}&nw={}#mensaplan",
        location.to_url_code(),
        week
    );

    let html = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&html);

    let meal_info_selector =
        Selector::parse(".mensa_menu_detail").expect("Meal info selector failed");
    let name_selector = Selector::parse(".menu_name").expect("Meal name selector failed");
    let price_selector = Selector::parse(".menu_preis").expect("Price selector failed");
    let menu_location_selector =
        Selector::parse(".menu_art").expect("Menu location selector failed");
    let day_element_selector =
        Selector::parse(".tag_headline[data-day]").expect("Day element selector failed");

    document
        .select(&day_element_selector)
        .map(|day_container| {
            let date_str = day_container.attr("data-day");
            (date_str, day_container)
        })
        .flat_map(|(date_str, day_container)| {
            let date_str = date_str.clone();
            day_container
                .select(&meal_info_selector)
                .map(move |meal_info| (date_str, meal_info))
        })
        .map(|(date_str, meal_info)| -> anyhow::Result<Meal> {
            let name = meal_info
                .select(&name_selector)
                .next()
                .ok_or(anyhow!("Failed to select meal name element"))
                .and_then(|name_el| {
                    let re = Regex::new(
                        r#"</?\w+((\s+\w+(\s*=\s*(?:".*?"|'.*?'|[^'">\s]+))?)+\s*|\s*)/?>"#,
                    )
                    .expect("Name Regex failed");
                    let inner_html = name_el.inner_html();
                    let name_vec = re
                        .split(&inner_html)
                        .filter(|item| *item != "" && !item.starts_with("(") && !item.contains("="))
                        .map(|item| item.trim())
                        .collect::<Vec<&str>>();
                    let name_str = name_vec.join(", ");
                    decode(name_str.as_bytes()).to_string()
                })?;

            // TODO: ist inner_html ausreichend?
            let price = meal_info
                .select(&price_selector)
                .next()
                .ok_or(anyhow!("Failed to select price element"))
                .map(|e| e.inner_html())
                .and_then(|html| decode(html.as_bytes()).to_string())?;

            let vegan = meal_info
                .attr("data-arten")
                .is_some_and(|a| a.contains("vn"));

            let vegetarian = meal_info
                .attr("data-arten")
                .is_some_and(|a| a.contains("ve"))
                || vegan;

            let meal_location = if location == Location::Musikhochschule {
                Location::Musikhochschule
            } else {
                meal_info
                    .select(&menu_location_selector)
                    .next()
                    .map(|e| {
                        if e.inner_html().contains("Mensa") {
                            Location::Mensa
                        } else {
                            Location::Cafeteria
                        }
                    })
                    .ok_or(anyhow!("Failed to select menu location"))?
            };

            let raw_allergenes = meal_info
                .attr("data-allergene")
                .ok_or(anyhow!("Failed to get allergene attr"))?;

            // TODO: Do not clone, but use a reference into the allergene vec.
            let meal_allergenes: Vec<Allergene> = allergenes
                .iter()
                .filter(|allergene| raw_allergenes.contains(&allergene.code))
                .map(|a| a.clone())
                .collect();

            let date = date_str.ok_or(anyhow!("Failed to extract date info"))?;

            Ok(Meal {
                name,
                price,
                vegan,
                vegetarian,
                location: meal_location.into(),
                allergens: meal_allergenes,
                date: date.to_string(),
            })
        })
        .collect()
}

pub async fn scrape_allergens() -> anyhow::Result<Vec<Allergene>> {
    let url = "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa=8&nw=0#mensaplan";

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
