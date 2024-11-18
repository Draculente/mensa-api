use serde::Serialize;

pub trait SourceData: Sync + Send {
    fn get_meals(&self) -> &Vec<Meal>;
    fn get_allergens(&self) -> &Vec<Allergen>;
    fn get_locations(&self) -> &Vec<APILocation>;
}

pub trait Source {
    type Item: SourceData + Sync;

    async fn fetch(&mut self) -> anyhow::Result<&Self::Item>;
    /// This MUST work without prior fetch!
    fn get_locations(&self) -> &Vec<APILocation>;
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

impl Prices {
    pub fn new(students: f32, employees: f32, guests: f32) -> Self {
        Self {
            students,
            employees,
            guests,
        }
    }
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
