use anyhow::anyhow;
use htmlentity::entity::decode;
use htmlentity::entity::ICodedDataTrait;
use itertools::Itertools;
use regex::Regex;
use scraper::Html;
use scraper::Selector;

use crate::model::{Allergen, Location, Meal};
use futures::future::join_all;
use strum::IntoEnumIterator;

pub async fn scrape_meals(allergens: &Vec<Allergen>) -> anyhow::Result<Vec<Meal>> {
    // 0,1
    let weeks = 0..2;

    let futures = weeks
        .cartesian_product(Location::iter().unique_by(|l| l.to_url_code()))
        .map(|(week, location)| scrape_meals_of_week(location, week, allergens));

    let vecs_of_meals = join_all(futures)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(vecs_of_meals.into_iter().flatten().collect())
}
async fn scrape_meals_of_week(
    location: Location,
    week: usize,
    allergens: &Vec<Allergen>,
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

            let raw_allergens = meal_info
                .attr("data-allergene")
                .ok_or(anyhow!("Failed to get allergen attr"))?;

            // TODO: Do not clone, but use a reference into the allergen vec.
            let meal_allergens: Vec<Allergen> = allergens
                .iter()
                .filter(|allergen| raw_allergens.contains(&allergen.code))
                .map(|a| a.clone())
                .collect();

            let date = date_str.ok_or(anyhow!("Failed to extract date info"))?;

            let price = meal_info
                .select(&price_selector)
                .next()
                .ok_or(anyhow!("Failed to select price element"))
                .map(|e| e.text().map(|n| n).join("/"))
                .and_then(|html| decode(html.as_bytes()).to_string())?
                .try_into()
                .unwrap_or_default();

            Ok(Meal {
                name,
                price,
                vegan,
                vegetarian,
                location: meal_location.into(),
                allergens: meal_allergens,
                date: date.to_string(),
            })
        })
        .collect()
}

pub async fn scrape_allergens() -> anyhow::Result<Vec<Allergen>> {
    let url = "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa=8&nw=0#mensaplan";

    let html = reqwest::get(url).await?.text().await?;

    let document = scraper::Html::parse_document(&html);

    let parent_element_selector = Selector::parse(".mbf_content").expect("Selector failed");
    let parent_element = document
        .select(&parent_element_selector)
        .next()
        .ok_or(anyhow!("Failed to get the allergen parent element"))?;

    let allergens: Vec<Allergen> = parent_element
        .child_elements()
        .map(|e| -> Option<Allergen> {
            let code = e.attr("data-wert")?.to_string();
            let name = e.child_elements().skip(1).next()?.inner_html();
            Some(Allergen { code, name })
        })
        .filter_map(|a| a)
        .collect();

    Ok(allergens)
}
