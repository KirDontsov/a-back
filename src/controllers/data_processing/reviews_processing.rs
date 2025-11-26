use crate::models::{Count, Firm, OAIReview, Review, SaveOAIReview};
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

#[derive(Deserialize, Serialize, Debug)]
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
#[get("/processing/reviews")]
async fn reviews_processing_handler(
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
	let counter_id: String = String::from("a518df5b-1258-482b-aa57-e07c57961a69");
	let https = HttpsConnector::new();
	let client = Client::builder().build(https);
	let uri = std::env::var("OPENAI_API_BASE").unwrap();
	let oai_token = env::var("OPENAI_API_KEY").unwrap();
	let model = "gpt-4o-mini".to_string();
	let _auth_header_val = format!("Bearer {}", oai_token);
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

	// let city = "moscow";
	// let category = "клубы";

	let firms_count =
		Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
			.await
			.unwrap_or(0);
	dbg!(&firms_count);

	// получаем из базы начало счетчика
	let start = get_counter(&data.db, &counter_id).await;

	for j in start.clone()..firms_count {
		println!("Firm: {:?}", j + 1);

		let firm =
			Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
				.await?;

		let firm_id = &firm.firm_id.clone();
		let firm_name = &firm.name.clone().unwrap();
		dbg!(&firm_id);
		dbg!(&firm_name);

		let oai_review = sqlx::query_as!(
			OAIReview,
			r#"SELECT * FROM oai_reviews WHERE firm_id = $1;"#,
			&firm.firm_id
		)
		.fetch_one(&data.db)
		.await;

		if oai_review.is_ok() {
			println!("Already exists");
			continue;
		}

		let mut reviews: Vec<SaveOAIReview> = Vec::new();

		let reviews_by_firm = Review::get_all_reviews(&data.db, &firm.firm_id)
			.await
			.unwrap();

		if reviews_by_firm.len() < 2 {
			println!("SKIP - Too few reviews");
			continue;
		}

		let reviews_string = &reviews_by_firm
			.into_iter()
			.map(|review| review.text.unwrap_or("".to_string()))
			.filter(|n| n != "")
			.collect::<Vec<String>>()
			.join("; ");

		// system preamble
		let first_message = OAIMessage {
			role: "system".to_string(),
			content: "Ты писатель-копирайтер, пишешь SEO оптимизированные тексты".to_string(),
		};

		// user preamble
		let preamble = format!("
		Вот отзывы которые ты должен проанализировать: {}

		Напиши большую статью, на основе этих отзывов об автосервисе {},
		важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в автосервисах, но без упоминания слова - Статья

		1. Подробно опиши в этой статье: какие виды работ обсуждают люди,
		2. Что из этих работ было сделано хорошо, а что плохо,
		3. Обманывают ли в этом автосервисе или нет.
		Например, если об этом говорят в отзывах:
		В отзывах обсуждаются следующие услуги:
		- Кузовной ремонт - плохое качество
		- Мастера - отзывчивые

		4. Выведи нумерованный список: плюсов и минусов если человек обратится в этот автосервис для ремонта своего автомобиля.
		Например, если об этом говорят в отзывах:
		Плюсы
			1. Хорошо чинят машины
			2. Хорошо красят
			Минусы
			1. Далеко от центра города

		5. Важно - подсчитай и выведи не нумерованным списком сумму положительных и сумму отрицательных отзывов,
		Например:
		Положительных отзывов проанализировано - 15
		Отрицательных отзывов проанализировано - 5

		6. Сделай выводы, на основе плюсов и минусов организации, количества положительных и отрицательных отзывов.
		Например:
		У организации больше положительных отзывов, укажи что рейтинг организации хороший, и объясни почему.
		Или например:
		У организации поровну положительных и отрицательных отзывов, укажи что рейтинг организации удовлетворительный, и объясни почему.
		Или например:
		У организации больше отрицательных отзывов, укажи что рейтинг организации не удовлетворительный, и объясни почему.
		", &reviews_string.chars().take(3800).collect::<String>(), &firm_name);

		let second_message = OAIMessage {
			role: "user".to_string(),
			content: format!("{}", preamble),
		};

		// request
		let oai_request = OAIRequest {
			model: model.clone(),
			messages: vec![first_message, second_message],
		};

		let body = Body::from(serde_json::to_vec(&oai_request)?);
		let req = Request::post(&uri)
			.header(header::CONTENT_TYPE, "application/json")
			// .header("Authorization", &auth_header_val)
			.body(body)
			.unwrap();

		// response
		match tokio::time::timeout(Duration::from_secs(60 * 5), client.request(req)).await {
			Ok(result) => match result {
				Ok(response) => {
					println!("Status: {}", response.status());

					let json: OAIResponse =
						serde_json::from_reader(hyper::body::aggregate(response).await?.reader())?;

					let review = json.choices.get(0).unwrap().message.content.clone();

					if review == "" {
						println!("Empty openai response {:?}", review);
						continue;
					}

					reviews.push(SaveOAIReview {
						firm_id: firm.firm_id.clone(),
						text: review
							.clone()
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
		for review in reviews {
			let x = sqlx::query_as!(
				OAIReview,
				r#"INSERT INTO oai_reviews (firm_id, text) VALUES ($1, $2) RETURNING *"#,
				review.firm_id,
				review.text,
			)
			.fetch_one(&data.db)
			.await;

			dbg!(&x);
		}

		let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
	}

	Ok(())
}

// Вот отзывы которые ты должен проанализировать: {}

// 		Напиши большую статью, на основе этих отзывов об автосервисе {},
// 		важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в автосервисах, но без упоминания слова - Статья

// 		Подробно опиши в этой статье: какие виды работ обсуждают люди,
// 		что из этих работ было сделано хорошо, а что плохо,
// 		обманывают ли в этом автосервисе или нет.
// 		Например, если об этом говорят в отзывах:
// 		В отзывах обсуждаются следующие услуги:
// 		1. Кузовной ремонт - плохое качество
// 		2. Мастера - отзывчивые

// 		Выведи нумерованный список: плюсов и минусов если человек обратится в этот автосервис для ремонта своего автомобиля.
// 		Например, если об этом говорят в отзывах:
// 		Плюсы
// 		1. Хорошо чинят машины
// 		2. Хорошо красят
// 		Минусы
// 		1. Далеко от центра города

// 		Важно - подсчитай и выведи не нумерованным списком сумму положительных и сумму отрицательных отзывов,
// 		Например:
// 		Положительных отзывов - 15
// 		Отрицательных отзывов - 5

// 		Сделай выводы, на основе плюсов и минусов организации, количества положительных и отрицательных отзывов.
// 		Например:
// 		У организации больше положительных отзывов, укажи что рейтинг организации хороший, и объясни почему.
// 		Или например:
// 		У организации поровну положительных и отрицательных отзывов, укажи что рейтинг организации удовлетворительный, и объясни почему.
// 		Или например:
// 		У организации больше отрицательных отзывов, укажи что рейтинг организации не удовлетворительный, и объясни почему.

// let preamble = format!("
// 		Вот отзывы которые ты должен проанализировать: {}

// 		Напиши большую статью, на основе этих отзывов о ресторане {},
// 		важно, чтобы текст был понятен 18-летним девушкам и парням, которые не разбираются в ресторанах, но без упоминания слова - Статья

// 		Подробно опиши в этой статье:
// 		1. Что обсуждают люди в отзывах,
// 		2. Что в ресторане хорошо, а что плохо,
// 		3. Если в ресторане есть детская комната, что пишут о ней,
// 		4. Какие блюда рекомендуют, а какие лучше не заказывать.

// 		Выведи нумерованный список: плюсов и минусов ресторана, например:
// 		Плюсы
// 		1. Если об этом говорят в отзывах: Дружелюбный персонал
// 		2. Если об этом говорят в отзывах: Уютная атмосфера
// 		Минусы
// 		1. Если об этом говорят в отзывах: Громкая музыка

// 		Важно - подсчитай и выведи не нумерованным списком сумму положительных и сумму отрицательных отзывов которые проанализировал,
// 		Например:
// 		Проанализировано положительных отзывов - 15
// 		Проанализировано отрицательных отзывов - 5

// 		Сделай выводы, на основе плюсов и минусов организации, количества положительных и отрицательных отзывов.
// 		Например:
// 		У ресторана больше положительных отзывов, укажи что рейтинг ресторана хороший, и объясни почему.
// 		Или например:
// 		У ресторана поровну положительных и отрицательных отзывов, укажи что рейтинг ресторана удовлетворительный, и объясни почему.
// 		Или например:
// 		У ресторана больше отрицательных отзывов, укажи что рейтинг ресторана не удовлетворительный, и объясни почему.
// 		", &reviews_string.chars().take(3800).collect::<String>(), &firm_name);
