use rocket::response::Redirect;
use rocket::{State, response::status::Custom};
use rocket::serde::json::Json;
use rocket::http::Status;

#[macro_use]
extern crate rocket;

mod structures;
use structures::{HealthResponse, UrlRequest, Response, ErrorResponse};

mod db;

mod shortener;
use shortener::generate_shortened_hash;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(db::mount_aerospike_client())
        .mount("/", routes![healthcheck, handle_new_url, handle_get])
}

#[get("/health")]
pub async fn healthcheck(state: &State<db::AppState>) -> Result<Json<HealthResponse>, Status> {
    let response = HealthResponse {
        status: state.aerospike.is_connected().to_string(),
        message: "dummy message".to_string(),
    };
    Ok(Json(response))
}

#[get("/<hash>")]
pub async fn handle_get(state: &State<db::AppState>, hash: String) -> Result<Redirect, Status> {
    match db::get_record(state, hash) {
        Ok(record) => {
            let long_url = record.bins.get("long_url");
            Ok(Redirect::to(long_url.expect("Error while redirecting").to_string()))
        },
        Err(_) => Err(Status::NotFound),
    }
}

#[post("/url", data="<body>")]
pub async fn handle_new_url(state: &State<db::AppState>, body: Json<UrlRequest<'_>>) -> Result<Json<Response>, Custom<Json<ErrorResponse>>> {
    let url_hash = generate_shortened_hash(&body.url.to_string());
    let response = Response {
        shorten_url: url_hash.clone(),
        url        : body.url.to_string(),
    };
    match db::put_record(state, body.url.to_string(), url_hash.clone()) {
        Ok(()) => Ok(Json(response)),
        Err(e) => {
            let response = ErrorResponse {
                message: e.to_string(),
            };
            Err(Custom(Status::InternalServerError, Json(response)))
        }
    }
}
