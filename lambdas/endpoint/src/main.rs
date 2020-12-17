pub mod error;
pub mod routes;
pub mod utils;

use anyhow::Result as AnyResult;
use fern::Dispatch;
use lambda_http::{lambda, Body, IntoResponse, Request, Response};
use lambda_runtime::error::HandlerError;
use lambda_runtime::Context;
use log::LevelFilter;
use tokio::runtime::Runtime;

use crate::error::Error;

fn handler(req: Request, _: Context) -> Result<Response<Body>, HandlerError> {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(async move {
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
    })
}

fn main() -> AnyResult<()> {
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
    lambda!(handler);
    log::info!("Handler registered");
    Ok(())
}
