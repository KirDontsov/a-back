use crate::{
	api::Driver,
	models::{Firm, Image},
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use std::{fs::create_dir, path::Path};
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[allow(unreachable_code)]
#[get("/crawler/exact_images")]
async fn exact_firm_images_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let _: Result<(), Box<dyn std::error::Error>> = match crawler(data.clone()).await {
		Ok(x) => Ok(x),
		Err(e) => {
			println!("{:?}", e);
			Err(Box::new(e))
		}
	};

	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn crawler(data: web::Data<AppState>) -> WebDriverResult<()> {
	// let counter_id: String = String::from("2a94ecc5-fb8d-4b4d-bb03-e3ee2eb708da");
	let _table = String::from("firms");
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

	let url = env::var("CRAWLER_EXACT_FIRM_URL").expect("CRAWLER_EXACT_FIRM_URL not set");

	let firm = Firm::get_firm_by_url(&data.db, &url).await.unwrap();

	let existed_images = sqlx::query_as!(
		Image,
		"SELECT * FROM images WHERE firm_id = $1",
		firm.firm_id.clone(),
	)
	.fetch_one(&data.db)
	.await;

	println!("DELETE {:?}", existed_images);

	if existed_images.is_ok() {
		let _delete = sqlx::query_as!(
			Image,
			"DELETE FROM images WHERE firm_id = $1",
			firm.firm_id.clone(),
		)
		.fetch_one(&data.db)
		.await;
	}

	let driver = <dyn Driver>::get_driver().await?;
	driver
		.goto(format!(
			"https://2gis.ru/{}/search/{}/firm/{}/tab/photos",
			&city_name,
			&category_name,
			&firm.two_gis_firm_id.clone().unwrap()
		))
		.await?;
	sleep(Duration::from_secs(5)).await;

	let error_block = match find_error_block(driver.clone()).await {
		Ok(img_elem) => img_elem,
		Err(e) => {
			println!("error while searching error block: {}", e);
			"".to_string()
		}
	};

	if error_block.contains("Что-то пошло не так") {
		driver.refresh().await?;
	}

	let main_block = find_main_block(driver.clone()).await?;

	let main_block_content = main_block.inner_html().await?;

	if main_block_content.contains("Филиал удалён из справочника")
		|| main_block_content.contains("Филиал временно не работает")
		|| main_block_content.contains("Добавьте")
		|| main_block_content.contains("Людям нравится")
		|| main_block_content.contains("Скоро открытие")
	{
		driver.clone().quit().await?;
	}

	let mut blocks: Vec<WebElement> = Vec::new();

	// кол-во фото
	let img_count = match find_count_block(main_block.clone()).await {
		Ok(img_elem) => img_elem,
		Err(e) => {
			println!("error while searching count block: {}", e);
			0.0
		}
	};
	dbg!(&img_count);

	if img_count == 0.0 {
		driver.clone().quit().await?;
	}

	let edge: i32 = ((if img_count > 100.0 { 50.0 } else { img_count }) / 12.0).ceil() as i32;

	// скролим в цикле
	for _ in 0..edge {
		blocks = driver.query(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div/div[2]/div[2]/div[2]/div")).or(By::XPath("//div[contains(@class, \"_14aaw6sk\")]")).all_from_selector_required().await?;
		let last = blocks.last().unwrap();
		last.scroll_into_view().await?;
		sleep(Duration::from_secs(2)).await;
	}

	let dir_name = format!("{}", &firm.firm_id.clone());
	let _ = create_dir(Path::new(format!("output/images/{}", &dir_name).as_str()))?;

	for (_i, block) in blocks.clone().into_iter().enumerate() {
		let block_content = block.inner_html().await?;

		if block_content.contains("Добавить фото") {
			continue;
		}
		// Записываем в бд этот img_id, firm_id и можно сгенерить Alt для него
		let img_id = Uuid::new_v4();
		let file_name = format!("{}.png", &img_id);

		block.scroll_into_view().await?;

		let img = match find_img(block).await {
			Ok(img_elem) => img_elem,
			Err(e) => {
				println!("error while searching image: {}", e);
				"".to_string()
			}
		};

		if &img == "" {
			break;
		}

		dbg!(&dir_name);
		dbg!(&img);

		// Get the current window handle.
		let handle = driver.window().await?;
		// Open a new tab.
		driver.new_tab().await?;

		// Get window handles and switch to the new tab.
		let handles = driver.windows().await?;
		driver.switch_to_window(handles[1].clone()).await?;

		// We are now controlling the new tab.
		driver.goto(&img.replace("_328x170", "")).await?;

		let img_block = find_img_block(driver.clone()).await.unwrap();

		match img_block
			.screenshot(Path::new(
				format!("{}/{}/{}", "output/images", dir_name, &file_name).as_str(),
			))
			.await
		{
			Ok(_) => println!("image saved successfully"),
			Err(e) => println!("error while downloading image: {}", e),
		};

		// Switch back to original tab.
		driver.switch_to_window(handle.clone()).await?;

		// запись в бд
		let _ = sqlx::query_as!(
			Image,
			"INSERT INTO images (img_id, firm_id, img_alt) VALUES ($1, $2, $3) RETURNING *",
			img_id.clone(),
			firm.firm_id.clone(),
			firm.name.clone().unwrap(),
		)
		.fetch_one(&data.db)
		.await;
	}

	println!("id: {}", &firm.two_gis_firm_id.clone().unwrap());
	println!("{}", "======");
	driver.clone().quit().await?;

	Ok(())
}

pub async fn find_img(block: WebElement) -> Result<String, WebDriverError> {
	let img = block
		.query(By::Tag("img"))
		.or(By::XPath("//div[contains(@class, \"_14aaw6sk\")]/img"))
		.first()
		.await?
		.attr("src")
		.await?
		.unwrap();

	Ok(img)
}

pub async fn find_main_block(driver: WebDriver) -> Result<WebElement, WebDriverError> {
	let main_block = driver
			.query(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div"))
			.first()
			.await?;

	Ok(main_block)
}

pub async fn find_count_block(elem: WebElement) -> Result<f32, WebDriverError> {
	let elem_arr = match elem
			.query(By::XPath("//*[contains(text(),'Фото')]/span"))
			.or(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div/div[2]"))
			.all_from_selector_required()
			.await {
				Ok(block_elem) => block_elem,
				Err(e) => {
					println!("error while searching block: {}", e);
					Vec::<WebElement>::new()
				}
			};

	let res = match elem_arr.get(0).unwrap_or(&elem).inner_html().await {
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while searching block: {}", e);
			"".to_string()
		}
	};

	Ok(res.parse::<f32>().unwrap_or(0.0))
}

pub async fn find_img_block(driver: WebDriver) -> Result<WebElement, WebDriverError> {
	let img_block = driver
		.query(By::Tag("img"))
		.or(By::XPath("//div[contains(@class, \"_14aaw6sk\")]/img"))
		.first()
		.await?;

	Ok(img_block)
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
