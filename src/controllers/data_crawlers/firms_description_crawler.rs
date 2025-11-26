use crate::{
	api::Driver,
	models::{Count, Firm, UpdateFirmDesc},
	utils::{get_counter, update_counter},
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/description")]
async fn firms_description_crawler_handler(
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
	let counter_id: String = String::from("7711da84-7d98-4072-aa35-b642c7ac0762");
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

	let driver = <dyn Driver>::get_driver().await?;

	let firms_count =
		Count::count_firms_with_empty_field(&data.db, table.clone(), "description".to_string())
			.await
			.unwrap_or(0);
	// Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
	// 	.await
	// 	.unwrap_or(0);

	// получаем из базы начало счетчика
	let start = get_counter(&data.db, &counter_id).await;

	dbg!(&start);

	for j in start.clone()..=firms_count {
		println!("№ {}", &j + 1);

		let firm =
			Firm::get_firm_with_empty_field(&data.db, table.clone(), "description".to_string(), j)
				.await
				.unwrap();
		// Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
		// 	.await
		// 	.unwrap();

		let mut firms: Vec<UpdateFirmDesc> = Vec::new();

		// let existing_description = firm.description.clone().expect("");

		// if existing_description != "".to_string() {
		// 	continue;
		// }

		let url = format!(
			"https://2gis.ru/{}/search/{}/firm/{}/tab/info",
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
				"".to_string()
			}
		};

		if error_block.contains("Что-то пошло не так") {
			driver.refresh().await?;
		}

		// не запрашиваем информацию о закрытом
		let err_block = driver
			.query(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div/div[2]/div/div"))
			.first()
			.await?
			.inner_html()
			.await?;

		if err_block.contains("Филиал удалён из справочника")
			|| err_block.contains("Филиал временно не работает")
			|| err_block.contains("Скоро открытие")
		{
			continue;
		}

		let desc_block_xpath;

		// находим блоки среди которых есть блок с блоками с инфой
		let info_blocks = driver.query(By::XPath("//body/div/div/div/div/div/div[last()]/div[last()]/div/div/div/div/div[last()]/div[last()]/div/div/div/div/div/div/div[last()]/div[2]/div[1]/div/div/div")).all_from_selector_required().await?;
		// находим номер блока с блоками с инфой
		let mut info_block_number = 1;
		for (i, block) in info_blocks.clone().into_iter().enumerate() {
			if block.rect().await?.height >= info_blocks[0].rect().await?.height
				&& !(block.inner_html().await?.contains("Авторемонт")
					|| block
						.inner_html()
						.await?
						.contains("Продажа легковых автомобилей")
					|| block.inner_html().await?.contains("Кузовной ремонт")
					|| block
						.inner_html()
						.await?
						.contains("Автозапчасти и аксессуары")
					|| block
						.inner_html()
						.await?
						.contains("Марки легковых запчастей")
					|| block
						.inner_html()
						.await?
						.contains("Ремонт ходовой части автомобиля")
					|| block.inner_html().await?.contains("Способы оплаты")
					|| block.inner_html().await?.contains("В справочнике")
					|| block.inner_html().await?.contains("Рядом")
					|| block.inner_html().await?.contains("Транспорт"))
			{
				info_block_number = i + 1;
			}
		}
		desc_block_xpath = format!("//body/div/div/div/div/div/div[last()]/div[last()]/div/div/div/div/div[last()]/div[last()]/div/div/div/div/div/div/div[last()]/div[2]/div[1]/div/div/div[{}]", info_block_number);
		dbg!(&info_block_number);
		dbg!(&desc_block_xpath);

		dbg!(&info_blocks.len());

		let firm_desc = match find_block(driver.clone(), desc_block_xpath).await {
			Ok(elem) => elem,
			Err(e) => {
				println!("error while searching firm_site block: {}", e);
				"".to_string()
			}
		};

		firms.push(UpdateFirmDesc {
			firm_id: firm.firm_id.clone(),
			description: firm_desc.clone().replace("\n", ", "),
		});

		// запись в бд
		for firm in firms {
			let _ = sqlx::query_as::<_, Firm>(
				r#"UPDATE firms SET description = $1 WHERE firm_id = $2 RETURNING *"#,
			)
			.bind(&firm.description)
			.bind(&firm.firm_id)
			.fetch_one(&data.db)
			.await;

			dbg!(&firm);
		}
		// обновляем в базе счетчик
		let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
	}

	driver.clone().quit().await?;

	Ok(())
}

pub async fn find_block(driver: WebDriver, xpath: String) -> Result<String, WebDriverError> {
	let block = driver
		.query(By::XPath(&xpath))
		.first()
		.await?
		.text()
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
