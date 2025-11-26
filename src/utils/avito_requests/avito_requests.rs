use crate::models::{AvitoRequest, FilteredAvitoRequest};

// Function to filter and transform AvitoRequest record to FilteredAvitoRequest
pub fn filter_add_avito_request_record(request: &AvitoRequest) -> FilteredAvitoRequest {
	FilteredAvitoRequest {
		request_id: request.request_id.to_string(),
		user_id: request.user_id.to_string(),
		request: request.request.clone(),
		city: request.city.clone(),
		coords: request.coords.clone(),
		radius: request.radius.clone(),
		district: request.district.clone(),
		created_ts: request.created_ts,
	}
}
