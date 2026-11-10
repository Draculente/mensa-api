use serde::{Deserialize, Serialize};

use crate::model::{APILocation, Allergen, Language, Meal};

pub trait APIFilter<T>: for<'a> Deserialize<'a> + Send {
    fn accepts(&self, to_filter: &T) -> bool;
    fn filter<'a>(&self, to_be_filtered: &'a Vec<T>) -> Vec<&'a T> {
        to_be_filtered.iter().filter(|t| self.accepts(t)).collect()
    }
}
// Warp currently does not support vec. So I parse those manually with ',' as separator: https://github.com/seanmonstar/warp/issues/732
#[derive(Debug, Serialize, Deserialize)]
pub struct MealsQuery {
    date: Option<String>,
    location: Option<String>,
    exclude_allergens: Option<String>,
    vegan: Option<bool>,
    vegetarian: Option<bool>,
    organic: Option<bool>,
    contains_pork: Option<bool>,
    contains_beef: Option<bool>,
    contains_poultry: Option<bool>,
    contains_alcohol: Option<bool>,
    schleswig_holstein_plate: Option<bool>,
    streetfood: Option<bool>,
    /// Meals without a published CO2 value are excluded if this is set
    max_co2_grams: Option<u32>,
    /// Meals without a published CO2 value are excluded if this is set
    min_co2_score: Option<u8>,
    language: Option<String>,
}

fn matches_flag(filter: Option<bool>, value: bool) -> bool {
    filter.map(|f| f == value).unwrap_or(true)
}

impl APIFilter<Meal> for MealsQuery {
    fn accepts(&self, meal: &Meal) -> bool {
        self.date
            .as_ref()
            .map(|d| d.contains(&meal.date))
            .unwrap_or(true)
            && self
                .location
                .as_ref()
                .map(|d| d.contains(&meal.location.code))
                .unwrap_or(true)
            && self
                .exclude_allergens
                .as_ref()
                .map(|excluded_allergens| {
                    !meal
                        .allergens
                        .iter()
                        .any(|a| excluded_allergens.contains(&a.code))
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
            && {
                let d = &meal.dietary_preferences;
                matches_flag(self.organic, d.organic)
                    && matches_flag(self.contains_pork, d.contains_pork)
                    && matches_flag(self.contains_beef, d.contains_beef)
                    && matches_flag(self.contains_poultry, d.contains_poultry)
                    && matches_flag(self.contains_alcohol, d.contains_alcohol)
                    && matches_flag(self.schleswig_holstein_plate, d.schleswig_holstein_plate)
                    && matches_flag(self.streetfood, d.streetfood)
            }
            && self
                .max_co2_grams
                .map(|max| meal.co2_grams.is_some_and(|g| g <= max))
                .unwrap_or(true)
            && self
                .min_co2_score
                .map(|min| meal.co2_score.is_some_and(|s| s >= min))
                .unwrap_or(true)
            && self
                .language
                .as_ref()
                .cloned()
                .unwrap_or_else(|| Language::german().code)
                .split(",")
                .collect::<Vec<_>>()
                .contains(&meal.language.code.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllergensQuery {
    code: Option<String>,
    name: Option<String>,
    location: Option<String>,
    language: Option<String>,
}

impl APIFilter<Allergen> for AllergensQuery {
    fn accepts(&self, allergen: &Allergen) -> bool {
        self.code
            .as_ref()
            .map(|c| c.contains(&allergen.code))
            .unwrap_or(true)
            && self
                .name
                .as_ref()
                .map(|n| n.contains(&allergen.code))
                .unwrap_or(true)
            && self
                .language
                .as_ref()
                .cloned()
                .unwrap_or_else(|| Language::german().code)
                .split(",")
                .collect::<Vec<_>>()
                .contains(&allergen.language.code.as_str())
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
}
