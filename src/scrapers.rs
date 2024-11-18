use anyhow::anyhow;
use htmlentity::entity::decode;
use htmlentity::entity::ICodedDataTrait;
use itertools::Itertools;
use regex::Regex;
use scraper::Html;
use scraper::Selector;
use strum::EnumIter;

use crate::model::APILocation;
use crate::model::Prices;
use crate::model::Source;
use crate::model::SourceData;
use crate::model::{Allergen, Meal};
use futures::future::join_all;
use strum::IntoEnumIterator;

pub struct LuebeckMensaSource {
    locations: Vec<APILocation>,
    data: LuebeckData,
}

impl Source for LuebeckMensaSource {
    type Item = LuebeckData;

    async fn fetch(&mut self) -> anyhow::Result<&LuebeckData> {
        let locations: Vec<APILocation> = LuebeckLocation::iter().map(|l| l.into()).collect();
        let allergens = scrape_allergens().await?;
        let meals = scrape_meals(&allergens).await?;

        let data = LuebeckData {
            locations,
            allergens,
            meals,
        };
        self.data = data;
        Ok(&self.data)
    }

    fn get_locations(&self) -> &Vec<APILocation> {
        &self.locations
    }
}

#[derive(Debug, Clone)]
pub struct LuebeckData {
    allergens: Vec<Allergen>,
    meals: Vec<Meal>,
    locations: Vec<APILocation>,
}

impl SourceData for LuebeckData {
    fn get_meals(&self) -> &Vec<Meal> {
        &self.meals
    }

    fn get_allergens(&self) -> &Vec<Allergen> {
        &self.allergens
    }

    fn get_locations(&self) -> &Vec<APILocation> {
        &self.locations
    }
}

impl TryFrom<String> for Prices {
    type Error = anyhow::Error;

    fn try_from(value: String) -> anyhow::Result<Self> {
        let cleaned_values = value.replace("€", "").replace(",", ".");
        let num_values: Vec<f32> = cleaned_values
            .split("/")
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if num_values.len() != 3 {
            return Err(anyhow!("Invalid number of prices."));
        }

        Ok(Prices::new(num_values[0], num_values[1], num_values[2]))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum LuebeckLocation {
    Musikhochschule,
    Cafeteria,
    Mensa,
}

impl LuebeckLocation {
    /// The speiseplan website uses number codes to differentiate between locations.
    /// This method translates the location into these codes.
    pub(crate) fn to_url_code(&self) -> usize {
        match self {
            LuebeckLocation::Musikhochschule => 9,
            LuebeckLocation::Cafeteria => 8,
            LuebeckLocation::Mensa => 8,
        }
    }
}

impl Into<APILocation> for LuebeckLocation {
    fn into(self) -> APILocation {
        match self {
            LuebeckLocation::Musikhochschule => APILocation {
                code: "HL_MH".to_string(),
                name: "Musikhochschule".to_string(),
                city: "Lübeck".to_string(),
            },
            LuebeckLocation::Cafeteria => APILocation {
                code: "HL_CA".to_string(),
                name: "Cafeteria".to_string(),
                city: "Lübeck".to_string(),
            },
            LuebeckLocation::Mensa => APILocation {
                code: "HL_ME".to_string(),
                name: "Mensa".to_string(),
                city: "Lübeck".to_string(),
            },
        }
    }
}

pub async fn scrape_meals(allergens: &Vec<Allergen>) -> anyhow::Result<Vec<Meal>> {
    // 0,1
    let weeks = 0..2;

    let futures = weeks
        .cartesian_product(LuebeckLocation::iter().unique_by(|l| l.to_url_code()))
        .map(|(week, location)| scrape_meals_of_week(location, week, allergens));

    let vecs_of_meals = join_all(futures)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(vecs_of_meals.into_iter().flatten().collect())
}
async fn scrape_meals_of_week(
    location: LuebeckLocation,
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

            let meal_location = if location == LuebeckLocation::Musikhochschule {
                LuebeckLocation::Musikhochschule
            } else {
                meal_info
                    .select(&menu_location_selector)
                    .next()
                    .map(|e| {
                        if e.inner_html().contains("Mensa") {
                            LuebeckLocation::Mensa
                        } else {
                            LuebeckLocation::Cafeteria
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
