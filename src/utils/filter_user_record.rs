use crate::models::{FilteredUser, User};

pub fn filter_user_record(user: &User) -> FilteredUser {
	FilteredUser {
		id: user.id.to_string(),
		email: user.email.clone().unwrap_or_default(),
		name: user.name.clone().unwrap_or_default(),
		photo: user.photo.clone().unwrap_or_default(),
		role: user.role.clone().unwrap_or_default(),
		verified: user.verified.unwrap_or_default(),
		favourite: user.favourite.clone().unwrap_or(Vec::new()),
		createdAt: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
		updatedAt: user.updated_at.unwrap_or_else(|| chrono::Utc::now()),
	}
}
