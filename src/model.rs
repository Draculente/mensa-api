use serde::Serialize;

use crate::scrapers::{scrape_allergens, scrape_meals};
use anyhow::anyhow;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Data {
    allergens: Vec<Allergen>,
    meals: Vec<Meal>,
    locations: Vec<APILocation>,
}

impl Data {
    pub(crate) async fn fetch() -> anyhow::Result<Data> {
        let locations: Vec<APILocation> = Location::iter().map(|l| l.into()).collect();
        let allergens = scrape_allergens().await?;
        let meals = scrape_meals(&allergens).await?;

        Ok(Self {
            locations,
            allergens,
            meals,
        })
    }

    pub fn get_meals(&self) -> &Vec<Meal> {
        &self.meals
    }

    pub fn get_allergens(&self) -> &Vec<Allergen> {
        &self.allergens
    }

    pub fn get_locations(&self) -> &Vec<APILocation> {
        &self.locations
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Allergen {
    pub(crate) code: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct APILocation {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) city: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Meal {
    pub(crate) name: String,
    pub(crate) date: String,
    pub(crate) price: Prices,
    pub(crate) vegan: bool,
    pub(crate) vegetarian: bool,
    pub(crate) location: APILocation,
    pub(crate) allergens: Vec<Allergen>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Prices {
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
        let num_values: Vec<f32> = cleaned_values
            .split("/")
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if num_values.len() != 3 {
            return Err(anyhow!("Invalid number of prices."));
        }

        Ok(Prices {
            students: num_values[0],
            employees: num_values[1],
            guests: num_values[2],
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
    pub(crate) fn to_url_code(&self) -> usize {
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
                code: "HL_MH".to_string(),
                name: "Musikhochschule".to_string(),
                city: "Lübeck".to_string(),
            },
            Location::Cafeteria => APILocation {
                code: "HL_CA".to_string(),
                name: "Cafeteria".to_string(),
                city: "Lübeck".to_string(),
            },
            Location::Mensa => APILocation {
                code: "HL_ME".to_string(),
                name: "Mensa".to_string(),
                city: "Lübeck".to_string(),
            },
        }
    }
}
