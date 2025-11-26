use crate::{
	api::Driver,
	models::{Count, Firm, UpdateFirmCoords},
	utils::get_counter,
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/coords")]
async fn firms_coords_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	loop {
		let _: Result<(), Box<dyn std::error::Error>> = match crawler(data.clone()).await {
			Ok(x) => Ok(x),
			Err(e) => {
				println!("{:?}", e);
				Err(Box::new(e))
			}
	};
	}
	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn crawler(data: web::Data<AppState>) -> WebDriverResult<()> {
	let counter_id: String = String::from("ab887681-f062-40f0-88e2-421de924d573");
	let table = String::from("firms");
	let _city_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CITY_ID")
			.expect("CRAWLER_CITY_ID not set")
			.as_str(),
	)
	.unwrap();
	let _category_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CATEGORY_ID")
			.expect("CRAWLER_CATEGORY_ID not set")
			.as_str(),
	)
	.unwrap();
	let city_name = env::var("CRAWLER_CITY_NAME").expect("CRAWLER_CITY_NAME not set");
	let category_name = env::var("CRAWLER_CATEGOTY_NAME").expect("CRAWLER_CATEGOTY_NAME not set");
	let _rubric_id = env::var("CRAWLER_RUBRIC_ID").expect("CRAWLER_RUBRIC_ID not set");

	let empty_field = "coords".to_string();

	let driver = <dyn Driver>::get_driver().await?;

	let firms_count =
		Count::count_firms_with_empty_field(&data.db, table.clone(), empty_field.clone())
			.await
			.unwrap_or(0);

	// Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
	// 	.await
	// 	.unwrap_or(0);

	// получаем из базы начало счетчика
	let start = get_counter(&data.db, &counter_id).await;

	for j in start.clone()..=firms_count {
		let firm = Firm::get_firm_with_empty_field(&data.db, table.clone(), empty_field.clone(), j)
			.await
			.unwrap();

		// Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
		// 	.await
		// 	.unwrap();

		let mut firms: Vec<UpdateFirmCoords> = Vec::new();

		let url = format!(
			"https://2gis.ru/{}/search/{}/firm/{}",
			&city_name,
			&category_name,
			&firm.two_gis_firm_id.clone().unwrap()
		);

		driver.goto(url).await?;
		sleep(Duration::from_secs(5)).await;

		let error_block = match find_error_block(driver.clone()).await {
			Ok(img_elem) => img_elem,
			Err(e) => {
				println!("error while searching error block: {}", e);
				driver.clone().quit().await?;
				"".to_string()
			}
		};

		if error_block.contains("Что-то пошло не так") {
			driver.refresh().await?;
		}

		let coords_block = match find_tag_block(driver.clone(), "//body/div[2]/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div/div[1]/div[1]/div[3]/a".to_string()).await {
			Ok(elem) => elem,
			Err(e) => {
				println!("error while searching href block: {}", e);
				driver.clone().quit().await?;
				String::new()
			}
		};

		let split_target = String::from("/points/%7C");

		// TODO: попробовать заменить на regexp
		let url_part_one = *coords_block
			.split(&split_target)
			.collect::<Vec<&str>>()
			.get_mut(1)
			.unwrap_or(&mut "-?");

		let coords = *url_part_one
			.split("%3B")
			.collect::<Vec<&str>>()
			.get(0)
			.unwrap_or(&mut "");

		firms.push(UpdateFirmCoords {
			firm_id: firm.firm_id.clone(),
			coords: coords.replace("%2C", ", "),
		});

		// запись в бд
		for firm in firms {
			let _ = sqlx::query_as::<_, Firm>(
				r#"UPDATE firms SET coords = $1 WHERE firm_id = $2 RETURNING *"#,
			)
			.bind(&firm.coords)
			.bind(&firm.firm_id)
			.fetch_one(&data.db)
			.await;

			dbg!(&firm);
		}
		// обновляем в базе счетчик
		// let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;

		println!("№ {}", &j + 1);
	}

	driver.clone().quit().await?;

	Ok(())
}



pub async fn find_error_block(driver: WebDriver) -> Result<String, WebDriverError> {
	let err_block = driver
		.query(By::Id("root"))
		.first()
		.await?
		.inner_html()
		.await?;

	Ok(err_block)
}

pub async fn find_tag_block(driver: WebDriver, xpath: String) -> Result<String, WebDriverError> {
	let block = driver
		.query(By::XPath(&xpath))
		.nowait()
		.first()
		.await?
		.attr("href")
		.await?
		.unwrap_or("".to_string());

	Ok(block)
}
