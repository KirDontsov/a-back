use crate::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use photon_rs::colour_spaces::darken_hsl;
use photon_rs::conv::box_blur;
use photon_rs::multiple::watermark;
use photon_rs::native::{open_image, save_image};
use photon_rs::transform::crop;
use photon_rs::PhotonImage;

use glob::glob;

#[allow(unreachable_code)]
#[get("/processing/images")]
async fn images_processing_handler(
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

async fn processing(_data: web::Data<AppState>) -> Result<(), Box<dyn std::error::Error>> {
	// в цикле берем каждое фото
	// обрабатываем
	// и сохраняем обратно с тем же именем

	for entry in glob("output/images/**/*.png")? {
		match entry {
			Ok(path) => {
				println!("{}", path.display());
				let mut img = open_image(path.to_str().unwrap()).expect("File should open");
				let width = *&mut img.get_width();
				let height = *&mut img.get_height();

				let mut cropped_img: PhotonImage = crop(
					&mut img,
					width - 121_u32,
					height - 121_u32,
					width,
					height - 51_u32,
				);

				darken_hsl(&mut cropped_img, 0.1_f32);
				box_blur(&mut cropped_img);

				watermark(
					&mut img,
					&cropped_img,
					(width - 120_u32).into(),
					(height - 60_u32).into(),
				);
				save_image(img, path.to_str().unwrap()).expect("File should be saved");
			}
			Err(e) => {
				println!("Err: {:?}", e);
			}
		}
	}

	Ok(())
}
