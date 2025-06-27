use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;

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
    pub(crate) language: Language,
}

#[derive(Debug, Clone, Serialize)]
pub struct APILocation {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) city: String,
    pub(crate) available_languages: Vec<Language>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Meal {
    pub(crate) name: String,
    pub(crate) date: String,
    pub(crate) price: Prices,
    pub(crate) vegan: bool,
    pub(crate) vegetarian: bool,
    #[serde(serialize_with = "serialize_nested_location")]
    pub(crate) location: APILocation,
    #[serde(serialize_with = "serialize_nested_allergen")]
    pub(crate) allergens: Vec<Allergen>,
    pub(crate) language: Language,
}

fn serialize_nested_location<S>(nested: &APILocation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("APILocation", 2)?;
    state.serialize_field("code", &nested.code)?;
    state.serialize_field("name", &nested.name)?;
    state.serialize_field("city", &nested.city)?;
    state.end()
}

fn serialize_nested_allergen<S>(nested: &Vec<Allergen>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_seq(Some(nested.len()))?;
    for e in nested {
        let nested_allergen: NestedAllergen = NestedAllergen::from(e);
        state.serialize_element(&nested_allergen)?;
    }
    state.end()
}

#[derive(Debug, Clone, Serialize)]
struct NestedAllergen<'a> {
    code: &'a str,
    name: &'a str,
}

impl<'a> From<&'a Allergen> for NestedAllergen<'a> {
    fn from(value: &'a Allergen) -> Self {
        Self {
            code: &value.code,
            name: &value.name,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Language {
    /// The native name of the language
    pub(crate) name: String,
    /// The ISO 639 language code
    pub(crate) code: String,
}

impl Language {
    pub(crate) fn german() -> Self {
        Self {
            name: "Deutsch".to_owned(),
            code: "de".to_owned(),
        }
    }
    pub(crate) fn english() -> Self {
        Self {
            name: "English".to_owned(),
            code: "en".to_owned(),
        }
    }
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
    LuebeckMusikhochschule,
    LuebeckCafeteria,
    LuebeckMensa,
    LuebeckBitsBytes,
    KielMensa1,
    KielCafeteria1,
    KielMensa2,
    KielCafeteria2,
    KielMensaKesselhaus,
    KielSchwentine,
    KielDiner,
    KielDockside,
    HeideMensa,
    FlensburgMensa,
    FlensburgCafeteriaA,
    FlensburgCafeteriaB,
    OsterroenfeldMensa,
    WedelCafeteria,
}

impl Location {
    /// The speiseplan website uses number codes to differentiate between locations.
    /// This method translates the location into these codes.
    pub(crate) fn get_mensa_code(&self) -> usize {
        match self {
            Location::LuebeckMusikhochschule => 9,
            Location::LuebeckCafeteria => 8,
            Location::LuebeckMensa => 8,
            Location::KielMensa1 => 1,
            Location::KielCafeteria1 => 1,
            Location::KielMensa2 => 2,
            Location::KielCafeteria2 => 2,
            Location::KielMensaKesselhaus => 4,
            Location::KielSchwentine => 5,
            Location::KielDiner => 6,
            Location::KielDockside => 16,
            Location::HeideMensa => 16,
            Location::FlensburgMensa => 7,
            Location::FlensburgCafeteriaA => 7,
            Location::FlensburgCafeteriaB => 14,
            Location::OsterroenfeldMensa => 14,
            Location::WedelCafeteria => 15,
            Location::LuebeckBitsBytes => 17,
        }
    }
    /// The speiseplan website uses number codes to differentiate between locations.
    /// This method translates the location into these codes.
    pub(crate) fn get_ort_code(&self) -> usize {
        match self {
            Location::LuebeckMusikhochschule => 3,
            Location::LuebeckCafeteria => 3,
            Location::LuebeckMensa => 3,
            Location::KielMensa1 => 1,
            Location::KielCafeteria1 => 1,
            Location::KielMensa2 => 1,
            Location::KielCafeteria2 => 1,
            Location::KielMensaKesselhaus => 1,
            Location::KielSchwentine => 1,
            Location::KielDiner => 1,
            Location::KielDockside => 1,
            Location::HeideMensa => 4,
            Location::FlensburgMensa => 2,
            Location::FlensburgCafeteriaA => 2,
            Location::FlensburgCafeteriaB => 2,
            Location::OsterroenfeldMensa => 6,
            Location::WedelCafeteria => 5,
            Location::LuebeckBitsBytes => 3,
        }
    }

    pub(crate) fn get_cafeteria_option(&self) -> Self {
        match self {
            Location::LuebeckMusikhochschule => Location::LuebeckMusikhochschule,
            Location::LuebeckCafeteria => Self::LuebeckCafeteria,
            Location::LuebeckMensa => Self::LuebeckCafeteria,
            Location::KielMensa1 => Self::KielCafeteria1,
            Location::KielCafeteria1 => Self::KielCafeteria1,
            Location::KielMensa2 => Self::KielCafeteria2,
            Location::KielCafeteria2 => Self::KielCafeteria2,
            Location::KielMensaKesselhaus => Self::KielMensaKesselhaus,
            Location::KielSchwentine => Self::KielSchwentine,
            Location::KielDiner => Self::KielDiner,
            Location::KielDockside => Self::KielDockside,
            Location::HeideMensa => Self::HeideMensa,
            Location::FlensburgMensa => Self::FlensburgCafeteriaA,
            Location::FlensburgCafeteriaA => Self::FlensburgCafeteriaA,
            Location::FlensburgCafeteriaB => Self::FlensburgCafeteriaB,
            Location::OsterroenfeldMensa => Self::OsterroenfeldMensa,
            Location::WedelCafeteria => Self::WedelCafeteria,
            Location::LuebeckBitsBytes => Self::LuebeckBitsBytes,
        }
    }

    pub(crate) fn get_mensa_option(&self) -> Self {
        match self {
            Location::LuebeckMusikhochschule => Location::LuebeckMusikhochschule,
            Location::LuebeckCafeteria => Self::LuebeckMensa,
            Location::LuebeckMensa => Self::LuebeckMensa,
            Location::KielMensa1 => Self::KielMensa1,
            Location::KielCafeteria1 => Self::KielMensa1,
            Location::KielMensa2 => Self::KielMensa2,
            Location::KielCafeteria2 => Self::KielMensa2,
            Location::KielMensaKesselhaus => Self::KielMensaKesselhaus,
            Location::KielSchwentine => Self::KielSchwentine,
            Location::KielDiner => Self::KielDiner,
            Location::KielDockside => Self::KielDockside,
            Location::HeideMensa => Self::HeideMensa,
            Location::FlensburgMensa => Self::FlensburgMensa,
            Location::FlensburgCafeteriaA => Self::FlensburgMensa,
            Location::FlensburgCafeteriaB => Self::FlensburgCafeteriaB,
            Location::OsterroenfeldMensa => Self::OsterroenfeldMensa,
            Location::WedelCafeteria => Self::WedelCafeteria,
            Location::LuebeckBitsBytes => Self::LuebeckBitsBytes,
        }
    }

    pub(crate) fn is_double(&self) -> bool {
        Self::iter().any(|e| {
            e.get_ort_code() == self.get_ort_code()
                && e.get_mensa_code() == self.get_mensa_code()
                && e != *self
        })
    }
}

impl Into<APILocation> for Location {
    fn into(self) -> APILocation {
        match self {
            Location::LuebeckMusikhochschule => APILocation {
                code: "HL_MH".to_string(),
                name: "Musikhochschule".to_string(),
                city: "Lübeck".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::LuebeckCafeteria => APILocation {
                code: "HL_CA".to_string(),
                name: "Cafeteria".to_string(),
                city: "Lübeck".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::LuebeckMensa => APILocation {
                code: "HL_ME".to_string(),
                name: "Mensa".to_string(),
                city: "Lübeck".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielMensa1 => APILocation {
                code: "KI_ME1".to_string(),
                name: "Mensa I".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielCafeteria1 => APILocation {
                code: "KI_CA1".to_string(),
                name: "Cafeteria I".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielMensa2 => APILocation {
                code: "KI_ME2".to_string(),
                name: "Mensa II".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielCafeteria2 => APILocation {
                code: "KI_CA2".to_string(),
                name: "Cafeteria II".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielMensaKesselhaus => APILocation {
                code: "KI_KESSELHAUS".to_string(),
                name: "Kesselhaus".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielSchwentine => APILocation {
                code: "KI_SCHWENTINE".to_string(),
                name: "Schwentine Mensa".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielDiner => APILocation {
                code: "KI_DINER".to_string(),
                name: "American Diner".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::KielDockside => APILocation {
                code: "KI_DOCKSIDE".to_string(),
                name: "Mensa „Dockside“".to_string(),
                city: "Kiel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::HeideMensa => APILocation {
                code: "HEI_ME1".to_string(),
                name: "Mensa".to_string(),
                city: "Heide".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::FlensburgMensa => APILocation {
                code: "FL_ME1".to_string(),
                name: "Mensa".to_string(),
                city: "Flensburg".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::FlensburgCafeteriaA => APILocation {
                code: "FL_CA1".to_string(),
                name: "Cafeteria A".to_string(),
                city: "Flensburg".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::FlensburgCafeteriaB => APILocation {
                code: "FL_CA2".to_string(),
                name: "Cafeteria B".to_string(),
                city: "Flensburg".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::OsterroenfeldMensa => APILocation {
                code: "RD_ME1".to_string(),
                name: "Mensa".to_string(),
                city: "Osterrönfeld".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::WedelCafeteria => APILocation {
                code: "PI_CA1".to_string(),
                name: "Cafeteria".to_string(),
                city: "Wedel".to_string(),
                available_languages: vec![Language::german(), Language::english()],
            },
            Location::LuebeckBitsBytes => APILocation {
                code: "HL_BB".to_string(),
                name: "Bits + Bytes".to_string(),
                city: "Lübeck".to_string(),
                available_languages: vec![Language::german()],
            },
        }
    }
}
