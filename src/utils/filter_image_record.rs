use crate::models::{FilteredImage, Image};

pub fn filter_image_record(image: &Image) -> FilteredImage {
	FilteredImage {
		img_id: image.img_id.to_string(),
		firm_id: image.firm_id.to_string(),
		img_alt: image.img_alt.to_owned(),
	}
}
