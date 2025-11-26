use crate::models::{City, FilteredCity};

pub fn filter_city_record(city: &City) -> FilteredCity {
	FilteredCity {
		city_id: city.city_id.to_string(),
		name: city.name.to_owned(),
		abbreviation: city.abbreviation.to_owned(),
		coords: city.coords.to_owned(),
		order_number: city.order_number.to_owned(),
		is_active: city.is_active.to_owned(),
	}
}
