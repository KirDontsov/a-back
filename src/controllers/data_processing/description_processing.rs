use crate::models::{Count, Firm, OAIDescription, UpdateFirmDesc};
use crate::utils::{get_counter, update_counter};
use crate::AppState;
use actix_web::web::Buf;
use actix_web::{get, web, HttpResponse, Responder};
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug)]
struct OAIMessage {
	role: String,
	content: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct OAIResponse {
	id: Option<String>,
	object: Option<String>,
	created: Option<u64>,
	model: Option<String>,
	choices: Vec<OAIChoices>,
}

// for choices
#[derive(Deserialize, Serialize, Debug)]
struct OAIChoices {
	index: u8,
	logprobs: Option<u8>,
	finish_reason: String,
	message: OAIMessage,
}

#[derive(Serialize, Debug)]
struct OAIRequest {
	model: String,
	messages: Vec<OAIMessage>,
}

#[allow(unreachable_code)]
#[get("/processing/description")]
async fn description_processing_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	loop {
		let _: Result<(), Box<dyn std::error::Error>> = match processing(data.clone()).await {
			Ok(x) => Ok(x),
			Err(e) => {
				println!("{:?}", e);
				let _ = sleep(Duration::from_secs(20)).await;
				Err(e)
			}
	};
	}
	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn processing(data: web::Data<AppState>) -> Result<(), Box<dyn std::error::Error>> {
	let counter_id: String = String::from("5e4f8432-c1db-4980-9b63-127fd320cdde");
	let uri = std::env::var("OPENAI_API_BASE").expect("url not set");
	let oai_token = env::var("OPENAI_API_KEY").expect("token not set");
	let model = "gpt-4o-mini".to_string();
	let auth_header_val = format!("Bearer {}", oai_token);
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
	let _city_name = env::var("CRAWLER_CITY_NAME").expect("CRAWLER_CITY_NAME not set");
	let _category_name = env::var("CRAWLER_CATEGOTY_NAME").expect("CRAWLER_CATEGOTY_NAME not set");
	let _rubric_id = env::var("CRAWLER_RUBRIC_ID").expect("CRAWLER_RUBRIC_ID not set");

	let firms_count =
		Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
			.await
			.unwrap_or(0);

	// получаем из базы начало счетчика
	let start = get_counter(&data.db, &counter_id).await;

	let https = HttpsConnector::new();
	let client = Client::builder().build(https);

	for j in start.clone()..firms_count {
		println!("Firm: {:?}", j + 1);
		let firm =
			Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
				.await?;

		let mut firms: Vec<UpdateFirmDesc> = Vec::new();

		// ====

		let firm_id = &firm.firm_id.clone();
		let firm_name = &firm.name.clone().unwrap_or("".to_string());
		let firm_desc = &firm.description.clone().unwrap_or("".to_string());
		let _firm_phone = &firm.default_phone.clone().unwrap_or("".to_string());
		dbg!(&firm_id);
		dbg!(&firm_name);

		if firm_name == "" || firm_desc == "" {
			continue;
		}

		let oai_description = sqlx::query_as!(
			OAIDescription,
			r#"SELECT * FROM oai_descriptions WHERE firm_id = $1;"#,
			&firm.firm_id
		)
		.fetch_one(&data.db)
		.await;

		if oai_description.is_ok() {
			println!("Already exists");
			continue;
		}
		if firm_desc.clone() == "" {
			println!("Empty description");
			continue;
		}

		let preamble = format!("Вот описание ночного клуба которое ты должен проанализировать: {}

		Напиши большую статью о ресторане, на основе анализа этого описания {},
		важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в ресторанах, но без упоминания слова - \"Статья\"

		Подробно опиши в этой статье:
		1. На чем специализируется данный ночной клуб, например, если об этом указано в описании:

		Данный ночной клуб специализируется на проведении крутых вечеринок и афтерпати

		2. Придумай в чем заключается миссия данного ночного клуба, чем он помогает людям. Какие праздники и концерты проходят в нем.

		3. Укажи что в ночном клубе работают опытные и квалифицированные сотрудники, которые всегда помогут и сделают это быстро и качественно.

		И перечисли все виды блюд, и напитков которые могут быть приготовлены в данном ночном клубе, если обэто упоминается в описании
		", &firm_desc, &firm_name);

		// request
		let oai_request = OAIRequest {
			model: model.clone(),
			messages: vec![OAIMessage {
				role: "user".to_string(),
				content: format!(
					"{}: {}",
					preamble,
					&firm
						.description
						.clone()
						.unwrap_or("Отсутствует описание".to_string())
				),
			}],
		};

		dbg!(uri.clone());

		let body = Body::from(serde_json::to_vec(&oai_request)?);
		let req = Request::post(uri.clone())
			.header(header::CONTENT_TYPE, "application/json")
			.header("Authorization", &auth_header_val)
			.body(body)
			.unwrap();

		// response
		match tokio::time::timeout(Duration::from_secs(60 * 5), client.request(req)).await {
			Ok(result) => match result {
				Ok(response) => {
					println!("Status: {}", response.status());

					let json: OAIResponse =
						serde_json::from_reader(hyper::body::aggregate(response).await?.reader())?;
					let desc = json.choices.get(0).unwrap().message.content.clone();

					if desc == "" {
						println!("Empty openai response {:?}", desc);
						continue;
					}

					firms.push(UpdateFirmDesc {
						firm_id: firm.firm_id.clone(),
						description: desc
							.replace("XYZ", &firm_name)
							.replace("#", "")
							.replace("*", ""),
					});
				}
				Err(e) => {
					println!("Network error: {:?}", e);
				}
			},
			Err(_) => {
				println!("Timeout: no response in 6000 milliseconds.");
			}
		};

		// запись в бд
		for firm in firms {
			let _ = sqlx::query_as!(
				OAIDescription,
				r#"INSERT INTO oai_descriptions (firm_id, oai_description_value) VALUES ($1, $2) RETURNING *"#,
				firm.firm_id,
				firm.description,
			)
			.fetch_one(&data.db)
			.await;

			dbg!(&firm);
		}

		let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
	}

	Ok(())
}

// let preamble = format!("Вот описание автосервиса которое ты должен проанализировать: {}

// 				Напиши большую статью об автосервисе, на основе анализа этого описания {},
// 				важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в автосервисах, но без упоминания слова - \"Статья\"

// 				Подробно опиши в этой статье:
// 				1. Какие виды работ может осуществлять данная организация, например, если об этом указано в описании:
// 				Данная организация может оказывать следующие виды работ: Кузовной ремонт, Замена масла, Замена шин, Покраска

// 				2. Придумай в чем заключается миссия данной организации по ремонту автомобилей, чем она помогает людям.

// 				3. Укажи что в компании работают опытные и квалифицированные сотрудники, которые всегда помогут и сделают это быстро и качественно.

// 				4. В конце текста укажи: Для получения более детальной информации позвоните по номеру: {} (если он указан)

// 				5. И перечисли все виды работ, которые могут быть свзаны с ремонтом автомобиля
// 				", &firm_desc, &firm_name, &firm_phone);

// let preamble = format!("Вот описание ресторана которое ты должен проанализировать: {}

// 		Напиши большую статью о ресторане, на основе анализа этого описания {},
// 		важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в ресторанах, но без упоминания слова - \"Статья\"

// 		Подробно опиши в этой статье:
// 		1. На чем специализируется данный ресторан, например, если об этом указано в описании:

// 		Данный ресторан специализируется на европейской кухне

// 		2. Придумай в чем заключается миссия данного ресторана, чем он помогает людям.

// 		3. Укажи что в ресторане работают опытные и квалифицированные сотрудники, которые всегда помогут и сделают это быстро и качественно.

// 		4. В конце текста укажи: Для получения более детальной информации позвоните по номеру: {}

// 		И перечисли все виды блюд, которые могут быть приготовлены в данном ресторане
// 		", &firm_desc, &firm_name, &firm_phone);
