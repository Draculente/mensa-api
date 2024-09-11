use anyhow::anyhow;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use futures::future::join_all;
use htmlentity::entity::decode;
use htmlentity::entity::ICodedDataTrait;
use itertools::Itertools;
use regex::Regex;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;
use strum::IntoEnumIterator;

pub trait APIFilter<T>: for<'a> Deserialize<'a> + Send {
    fn accepts(&self, to_filter: &T) -> bool;
    fn get_location_query_string(&self) -> &str;
    fn get_location_query(&self) -> Vec<Location> {
        Location::iter()
            .filter(|l| {
                let api_location: APILocation = (*l).into();
                self.get_location_query_string()
                    .contains(&api_location.code)
            })
            .collect()
    }
    fn filter<'a>(&self, to_be_filtered: &'a Vec<T>) -> Vec<&'a T> {
        to_be_filtered.iter().filter(|t| self.accepts(t)).collect()
    }
}
// Warp currently does not support vec. So I parse those manually with ',' as separator: https://github.com/seanmonstar/warp/issues/732
#[derive(Debug, Serialize, Deserialize)]
pub struct MealsQuery {
    date: Option<String>,
    location: String,
    exclude_allergenes: Option<String>,
    vegan: Option<bool>,
    vegetarian: Option<bool>,
}

impl APIFilter<Meal> for MealsQuery {
    fn accepts(&self, meal: &Meal) -> bool {
        self.date
            .as_ref()
            .map(|d| d.contains(&meal.date))
            .unwrap_or(true)
            && self.location.contains(&meal.location.code)
            && self
                .exclude_allergenes
                .as_ref()
                .map(|excluded_allergenes| {
                    !meal
                        .allergens
                        .iter()
                        .any(|a| excluded_allergenes.contains(&a.code))
                })
                .unwrap_or(true)
            && self
                .vegan
                .as_ref()
                .map(|vegan| &meal.vegan == vegan)
                .unwrap_or(true)
            && self
                .vegetarian
                .as_ref()
                .map(|vegetarian| &meal.vegetarian == vegetarian)
                .unwrap_or(true)
    }

    fn get_location_query_string(&self) -> &str {
        &self.location
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllergenesQuery {
    code: Option<String>,
    name: Option<String>,
    location: String,
}

impl APIFilter<Allergene> for AllergenesQuery {
    fn accepts(&self, allergene: &Allergene) -> bool {
        self.code
            .as_ref()
            .map(|c| c.contains(&allergene.code))
            .unwrap_or(true)
            && self
                .name
                .as_ref()
                .map(|n| n.contains(&allergene.code))
                .unwrap_or(true)
    }

    fn get_location_query_string(&self) -> &str {
        &self.location
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationsQuery {
    code: Option<String>,
    name: Option<String>,
    city: Option<String>,
}

impl APIFilter<APILocation> for LocationsQuery {
    fn accepts(&self, location: &APILocation) -> bool {
        self.code
            .as_ref()
            .map(|c| c.contains(&location.code))
            .unwrap_or(true)
            && self
                .name
                .as_ref()
                .map(|n| n.contains(&location.name))
                .unwrap_or(true)
            && self
                .city
                .as_ref()
                .map(|c| c.contains(&location.city))
                .unwrap_or(true)
    }

    fn get_location_query_string(&self) -> &str {
        ""
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    allergenes: Vec<Allergene>,
    meals: Vec<Meal>,
    locations: Vec<APILocation>,
}

impl Data {
    async fn fetch() -> anyhow::Result<Data> {
        let locations: Vec<APILocation> = Location::iter().map(|l| l.into()).collect();
        let allergenes = scrape_allergens().await?;
        let meals = scrape_meals(&allergenes).await?;

        Ok(Self {
            locations,
            allergenes,
            meals,
        })
    }

    pub fn get_meals(&self) -> &Vec<Meal> {
        &self.meals
    }

    pub fn get_allergenes(&self) -> &Vec<Allergene> {
        &self.allergenes
    }

    pub fn get_locations(&self) -> &Vec<APILocation> {
        &self.locations
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    data: Option<Data>,
    last_updated: DateTime<Utc>,
    ttl: Duration,
}

impl Cache {
    pub async fn get_data(&self) -> anyhow::Result<&Data> {
        self.data.as_ref().ok_or(anyhow!(
            "Failed to get data, because option is empty. This should not have happened!"
        ))
    }

    pub fn needs_update(&self) -> bool {
        let now = chrono::offset::Utc::now();
        self.last_updated + self.ttl < now
    }

    pub async fn fetch(&mut self) -> anyhow::Result<()> {
        self.data = Some(Data::fetch().await?);
        self.last_updated = chrono::offset::Utc::now();
        Ok(())
    }

    pub async fn new(ttl: Duration) -> anyhow::Result<Self> {
        Ok(Self {
            data: None,
            last_updated: DateTime::from_timestamp_nanos(0),
            ttl,
        })
    }

    pub fn get_last_update_as_string(&self) -> String {
        self.last_updated.to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Allergene {
    code: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct APILocation {
    code: String,
    name: String,
    city: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Meal {
    name: String,
    date: String,
    price: Prices,
    vegan: bool,
    vegetarian: bool,
    location: APILocation,
    allergens: Vec<Allergene>,
}

#[derive(Debug, Clone, Serialize)]
struct Prices {
    students: f32,
    employees: f32,
    guests: f32,
}

impl Default for Prices {
    fn default() -> Self {
        Self {
            students: Default::default(),
            employees: Default::default(),
            guests: Default::default(),
        }
    }
}

impl TryFrom<String> for Prices {
    type Error = anyhow::Error;

    fn try_from(value: String) -> anyhow::Result<Self> {
        let cleaned_values = value.replace("€", "").replace(",", ".");

        let num_values = cleaned_values
            .split("/")
            .filter(|s| !s.trim().is_empty())
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        if num_values.len() < 3 {
            return Err(anyhow!("Too few prices."));
        };

        Ok(Prices {
            students: *num_values
                .get(0)
                .expect("Too few item. This should not happen (0)"),
            employees: *num_values
                .get(0)
                .expect("Too few items. This should not happen (1)"),
            guests: *num_values
                .get(2)
                .expect("Too few items. This should not happen (2)"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Location {
    Musikhochschule,
    Cafeteria,
    Mensa,
}

impl Location {
    /// The speiseplan website uses number codes to differentiate between locations.
    /// This method translates the location into these codes.
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
                city: "Lübeck".to_string(),
            },
            Location::Cafeteria => APILocation {
                code: "CA".to_string(),
                name: "Cafeteria".to_string(),
                city: "Lübeck".to_string(),
            },
            Location::Mensa => APILocation {
                code: "ME".to_string(),
                name: "Mensa".to_string(),
                city: "Lübeck".to_string(),
            },
        }
    }
}

pub async fn scrape_meals(allergenes: &Vec<Allergene>) -> anyhow::Result<Vec<Meal>> {
    // 0,1
    let weeks = 0..2;

    let futures = weeks
        .cartesian_product(Location::iter().unique_by(|l| l.to_url_code()))
        .map(|(week, location)| scrape_meals_of_week(location, week, allergenes));

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

            let price = meal_info
                .select(&price_selector)
                .next()
                .ok_or(anyhow!("Failed to select price element"))
                .map(|e| e.inner_html())
                .and_then(|html| decode(html.as_bytes()).to_string())?
                .try_into()
                .unwrap_or_default();

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
