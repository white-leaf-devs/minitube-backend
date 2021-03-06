pub mod error;
pub mod routes;
pub mod utils;

use netlify_lambda_http::lambda::{lambda, Context};
use netlify_lambda_http::{Body, Request, Response};

use crate::error::Error;
use crate::utils::IntoCorsResponse;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda(http)]
#[tokio::main]
async fn main(req: Request, _: Context) -> Result<Response<Body>, DynError> {
    println!("Processing request: {:?}", req);

    let res = match req.uri().path() {
        "/search" => routes::search(req).await,
        "/createVideo" => routes::create_video(req).await,
        "/generateThumbnail" => routes::generate_thumbnail(req).await,
        invalid => Err(Error::invalid_route(invalid)),
    };

    let res = match res {
        Ok(res) => res,
        Err(err) => err.into_cors_response(),
    };

    println!("Sending response: {:?}", res);
    Ok(res)
}
