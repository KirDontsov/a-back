use sitemap::structs::UrlEntry;
use sitemap::writer::SiteMapWriter;
use std::env;
use std::io::stdout;

use crate::{
	models::{Category, City, Count, Firm},
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};

#[allow(unreachable_code)]
#[get("/processing/sitemap")]
async fn sitemap_processing_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	println!("start");
	let _: Result<(), Box<dyn std::error::Error>> = match processing(data.clone()).await {
		Ok(x) => Ok(x),
		Err(e) => {
			println!("{:?}", e);
			Err(e)
		}
	};

	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn processing(data: web::Data<AppState>) -> Result<(), Box<dyn std::error::Error>> {
	let table = String::from("firms");
	let city_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CITY_ID")
			.expect("CRAWLER_CITY_ID not set")
			.as_str(),
	)
	.unwrap();
	let category_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CATEGORY_ID")
			.expect("CRAWLER_CATEGORY_ID not set")
			.as_str(),
	)
	.unwrap();
	let domain = "https://xn--90ab9accji9e.xn--p1ai";

	let city = sqlx::query_as!(
		City,
		"SELECT * FROM cities WHERE city_id = $1;",
		city_id.clone()
	)
	.fetch_one(&data.db)
	.await
	.unwrap();

	let category = sqlx::query_as!(
		Category,
		"SELECT * FROM categories WHERE category_id = $1;",
		category_id.clone()
	)
	.fetch_one(&data.db)
	.await
	.unwrap();

	let firms_count = Count::count_firms_by_city_category(
		&data.db,
		table.clone(),
		city_id.clone(),
		category_id.clone(),
	)
	.await
	.unwrap_or(0);

	let mut output = stdout();
	let sitemap_writer = SiteMapWriter::new(&mut output);
	let mut urlwriter = sitemap_writer
		.start_urlset()
		.expect("Unable to write urlset");

	for j in 0..=firms_count {
		let firm = Firm::get_firm_by_city_category(
			&data.db,
			table.clone(),
			city_id.clone(),
			category_id.clone(),
			j,
		)
		.await
		.expect("there is no firm");

		if firm.url.clone().is_none() {
			continue;
		}

		let url = format!(
			"{}/{}/{}/{}",
			&domain,
			&city.abbreviation.clone().unwrap(),
			&category.abbreviation.clone().unwrap(),
			&firm.url.clone().unwrap()
		);

		urlwriter
			.url(UrlEntry::builder().loc(&url))
			.expect("Unable to write url");
	}

	urlwriter.end().expect("Unable to write close tags");

	Ok(())
}
