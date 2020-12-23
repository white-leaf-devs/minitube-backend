use std::time::Duration;

use json::Value;
use netlify_lambda_http::http::Method;
use netlify_lambda_http::{Body, IntoResponse, Request, RequestExt, Response};
use rusoto_core::Region;
use rusoto_credential::{ChainProvider, ProvideAwsCredentials};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::PutObjectRequest;
use serde::Deserialize;
use serde_json::{self as json, json};

use crate::error::Error;
use crate::utils::{generate_id, is_valid_id};
use crate::{handle_preflight_request, invoke_lambda, validate_request};

pub async fn create_video(req: Request) -> Result<Response<Body>, Error> {
    handle_preflight_request!(req);
    validate_request!(Method::GET, req);

    let id = generate_id();
    println!("Generated video id: {}", id);

    let input = PutObjectRequest {
        bucket: "minitube.videos".to_string(),
        key: format!("{}.mp4", id),
        acl: Some("public-read".to_string()),
        content_type: Some("video/mp4".to_string()),
        ..Default::default()
    };

    let credentials = ChainProvider::new().credentials().await?;
    let expires_in = Duration::from_secs(600);
    let options = PreSignedRequestOption { expires_in };

    println!("PutObject request: {:?}", input);
    println!("Credentials: {:?}", credentials);
    println!("Expires in: {:?}", expires_in);

    let presigned_url = input.get_presigned_url(&Region::UsEast1, &credentials, &options);
    let json_output = json!({
        "video_id": id,
        "presigned_url": presigned_url
    });

    Ok(json_output.into_response())
}

#[derive(Debug, Clone, Deserialize)]
struct GenerateThumbnail {
    video_id: String,
    timestamp: usize,
}

pub async fn generate_thumbnail(req: Request) -> Result<Response<Body>, Error> {
    handle_preflight_request!(req);
    validate_request!(Method::POST, "application/json", req);

    let body: GenerateThumbnail = if let Body::Text(json) = req.body() {
        json::from_str(&json).map_err(|e| Error::bad_request(e.to_string()))?
    } else {
        return Err(Error::bad_request("Invalid JSON body"));
    };

    println!("Parsed JSON body: {:?}", body);
    if !is_valid_id(&body.video_id) {
        return Err(Error::bad_request("Invalid `video_id`"));
    }

    let payload = json!({
        "video_key": format!("{}.mp4", body.video_id),
        "timestamp": body.timestamp,
    });

    let (output, payload) = invoke_lambda!("GenerateThumbnailLambda", payload);
    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("{} ({})", err, payload)))
    } else {
        Ok(().into_response())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DetectAndSaveLabels {
    video_id: String,
}

pub async fn detect_and_save_labels(req: Request) -> Result<Response<Body>, Error> {
    handle_preflight_request!(req);
    validate_request!(Method::POST, "application/json", req);

    let body: DetectAndSaveLabels = if let Body::Text(json) = req.body() {
        json::from_str(&json).map_err(|e| Error::bad_request(e.to_string()))?
    } else {
        return Err(Error::bad_request("Invalid JSON body"));
    };

    println!("Parsed JSON body: {:?}", body);
    if !is_valid_id(&body.video_id) {
        return Err(Error::bad_request("Invalid `video_id`"));
    }

    let payload = json!({
        "video_id": body.video_id.clone(),
        "bucket": "minitube.thumbnails"
    });

    let (output, payload) = invoke_lambda!("LabelThumbnailLambda", payload);
    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("{} ({})", err, payload)))
    } else {
        Ok(json::from_str::<Value>(&payload)?.into_response())
    }
}

pub async fn search(req: Request) -> Result<Response<Body>, Error> {
    handle_preflight_request!(req);
    validate_request!(Method::GET, req);

    let query = req.query_string_parameters();
    println!("Query parameters: {:?}", query);

    let query = query
        .get("query")
        .ok_or_else(|| Error::bad_request("Missing `query` param"))?;

    let labels: Vec<_> = query.split(' ').map(ToOwned::to_owned).collect();
    let payload = json!({ "labels": labels });

    let (output, payload) = invoke_lambda!("SearchLabelsLambda", payload);
    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("{} ({})", err, payload)))
    } else {
        Ok(json::from_str::<Value>(&payload)?.into_response())
    }
}
