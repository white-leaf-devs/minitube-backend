pub mod error;
pub mod routes;
pub mod utils;

use fern::Dispatch;
use log::LevelFilter;
use netlify_lambda::Context;
use netlify_lambda_http::{Body, IntoResponse, Request, Response};

use crate::error::Error;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

async fn handler(req: Request, _: Context) -> Result<Response<Body>, DynError> {
    let res = match req.uri().path() {
        "/uploadVideo" => routes::upload_video(req).await,
        "/genThumbnails" => routes::gen_thumbnails(req).await,
        "/uploadThumbnail" => routes::upload_thumbnail(req).await,
        "/search" => routes::search(req).await,
        invalid => Err(Error::invalid_route(invalid)),
    };

    Ok(match res {
        Ok(res) => res,
        Err(err) => err.into_response(),
    })
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    Dispatch::new()
        .level(LevelFilter::Info)
        .level_for("endpoint", LevelFilter::Debug)
        .level_for("lambda_http", LevelFilter::Debug)
        .chain(
            Dispatch::new()
                .format(move |out, msg, rec| {
                    out.finish(format_args! {
                        "{}[{}][{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        rec.level(),
                        rec.target(),
                        msg
                    })
                })
                .chain(std::io::stdout()),
        )
        .apply()?;

    log::info!("Registering handler");
    netlify_lambda::run(netlify_lambda_http::handler(handler)).await?;
    log::info!("Handler registered");

    Ok(())
}
