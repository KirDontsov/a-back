use crate::{
	api::Driver,
	models::{Count, Firm, UpdateFirmAddress},
	utils::{get_counter, update_counter},
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/address")]
async fn firms_address_crawler_handler(
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
	let counter_id: String = String::from("1e69083b-ef25-43d6-8a08-8e1d2673826e");
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
	let city_name = env::var("CRAWLER_CITY_NAME").expect("CRAWLER_CITY_NAME not set");
	let category_name = env::var("CRAWLER_CATEGOTY_NAME").expect("CRAWLER_CATEGOTY_NAME not set");
	let _rubric_id = env::var("CRAWLER_RUBRIC_ID").expect("CRAWLER_RUBRIC_ID not set");

	let _empty_field = "address".to_string();

	let driver = <dyn Driver>::get_driver().await?;

	let firms_count =
		// Count::count_firms_with_empty_field(&data.db, table.clone(), empty_field.clone())
		// 	.await
		// 	.unwrap_or(0);

		Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
			.await
			.unwrap_or(0);

	// получаем из базы начало счетчика
	let start = get_counter(&data.db, &counter_id).await;

	for j in start.clone()..=firms_count {
		let firm =
			// Firm::get_firm_with_empty_field(&data.db, table.clone(), empty_field.clone(), j)
			// .await
			// .unwrap();

			Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
				.await
				.unwrap();

		let mut firms: Vec<UpdateFirmAddress> = Vec::new();

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

		let blocks = match find_address_blocks(
			driver.clone(),
			"//div[contains(@class, \"_49kxlr\")]".to_string(),
		)
		.await
		{
			Ok(elem) => elem,
			Err(e) => {
				println!("error while searching firm_site block: {}", e);
				driver.clone().quit().await?;
				[].to_vec()
			}
		};

		let mut address = "".to_string();

		for block in blocks {
			let block_content = block.inner_html().await?;
			if block_content.contains("Оформить")
				|| block_content.contains("↗")
				|| block_content.contains("Инфо")
				|| block_content.contains("Отзывы")
				|| block_content.contains("Меню")
				|| block_content.contains("Фото")
			{
				continue;
			}

			let street_address = find_block(block.clone(), "_2lcm958".to_string()).await;
			let city_address = find_block(block.clone(), "_1p8iqzw".to_string()).await;

			let address_array = vec![street_address, city_address];

			address = address_array
				.into_iter()
				.collect::<Vec<String>>()
				.join(", ");

			// берем только превый блок и прерываем цикл
			break;
		}

		if address == ", ".to_string() {
			continue;
		}

		firms.push(UpdateFirmAddress {
			firm_id: firm.firm_id.clone(),
			address: address.clone().replace("\n", ", "),
		});

		// запись в бд
		for firm in firms {
			let _ = sqlx::query_as::<_, Firm>(
				r#"UPDATE firms SET address = $1 WHERE firm_id = $2 RETURNING *"#,
			)
			.bind(&firm.address)
			.bind(&firm.firm_id)
			.fetch_one(&data.db)
			.await;

			dbg!(&firm);
		}
		// обновляем в базе счетчик
		let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;

		println!("№ {}", &j + 1);
	}

	driver.clone().quit().await?;

	Ok(())
}

pub async fn find_block(elem: WebElement, xpath: String) -> String {
	let block_arr = match elem.find_all(By::ClassName(&xpath)).await {
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while searching block: {}", e);
			Vec::<WebElement>::new()
		}
	};

	let res = match block_arr.get(0).unwrap_or(&elem).text().await {
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while extracting text: {}", e);
			"".to_string()
		}
	};

	res
}

pub async fn find_address_blocks(
	driver: WebDriver,
	xpath: String,
) -> Result<Vec<WebElement>, WebDriverError> {
	let block = driver
		.query(By::XPath(&xpath))
		.all_from_selector_required()
		.await?;

	Ok(block)
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
