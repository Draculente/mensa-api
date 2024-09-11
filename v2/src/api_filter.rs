use serde::{Deserialize, Serialize};

use crate::model::{APILocation, Allergene, Location, Meal};
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
