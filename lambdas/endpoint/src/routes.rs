use std::collections::HashMap;
use std::io::Read;

use common_macros::hash_map;
use json::Value;
use netlify_lambda_http::http::Method;
use netlify_lambda_http::{Body, IntoResponse, Request, Response};
use rusoto_core::{ByteStream, Region};
use rusoto_dynamodb::{
    AttributeValue, BatchGetItemInput, DynamoDb, DynamoDbClient, KeysAndAttributes,
};
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use serde::Deserialize;
use serde_json::{self as json, json};

use crate::error::Error;
use crate::utils::{generate_id, is_valid_id, parse_multipart, query_params};
use crate::validate_request;

/// Expects base64 encode contents
pub async fn upload_video(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::POST, "multipart/form-data", req);

    log::info!("Uploading video to S3");
    log::debug!("Parsing multipart data");
    let multipart = parse_multipart(req)?;
    let entry = multipart.into_entry().into_result()?;

    let mut buf = Vec::new();

    if let Some(mut entry) = entry {
        entry.data.read_to_end(&mut buf)?;
    } else {
        return Err(Error::invalid_request("No contents"));
    }

    log::debug!("Decoding base64 contents");
    let buf = base64::decode(buf)?;

    log::debug!("Uploading to bucket");
    let s3 = S3Client::new(Region::UsEast1);
    let id = generate_id();

    let input = PutObjectRequest {
        bucket: "minitube.videos".to_string(),
        key: id.clone(),
        body: Some(ByteStream::from(buf)),
        grant_read: Some(true),
        ..Default::default()
    };

    s3.put_object(input).await?;
    log::debug!("Finished upload!");

    let res = json! {{
        "video_id": id
    }};

    Ok(res.into_response())
}

pub async fn gen_thumbnails(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::GET, req);

    log::info!("Requesting thumbnail generation");
    let query = query_params(req.uri().query().unwrap_or(""));
    if !query.get("video_id").map_or(false, |v| is_valid_id(v)) {
        return Err(Error::invalid_request(
            "Invalid or not present `video_id` query param",
        ));
    }

    log::debug!("Invocating GenerateThumbnails lamda");
    let lambda = LambdaClient::new(Region::UsEast1);
    let payload = json! {{
        "video_id": query["video_id"]
    }};

    log::debug!("Payload: {:#}", payload);
    let input = InvocationRequest {
        function_name: "GenerateThumbnailsLambda".to_string(),
        payload: Some(payload.to_string().into()),
        ..Default::default()
    };

    let output = lambda.invoke(input).await?;
    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("Function error: {}", err)))
    } else {
        let payload = output
            .payload
            .ok_or_else(|| Error::internal_error("Received unexpected empty output from lambda"))?;

        let contents = String::from_utf8(payload.to_vec())?;
        let json: Value = json::from_str(&contents)?;
        Ok(json.into_response())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct UploadThumbnail {
    video_id: String,
    /// Base64 (standard) encoded data
    thumbnail_data: String,
}

pub async fn upload_thumbnail(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::POST, "application/json", req);

    let input: UploadThumbnail = if let Body::Text(json) = req.body() {
        json::from_str(&json)?
    } else {
        return Err(Error::invalid_request("Invalid JSON body"));
    };

    let data = base64::decode(input.thumbnail_data)?;

    let s3 = S3Client::new(Region::UsEast1);
    let id = input.video_id;

    let input = PutObjectRequest {
        bucket: "minitube.thumbnails".to_string(),
        key: id.clone(),
        body: Some(ByteStream::from(data)),
        ..Default::default()
    };

    s3.put_object(input).await?;
    let res = json! {{
        "video_id": id
    }};

    Ok(res.into_response())
}

pub async fn search(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::GET, req);

    let query = query_params(req.uri().query().unwrap_or(""));
    if !query.contains_key("q") {
        return Err(Error::invalid_request("Not present `q` query param"));
    }

    let query: String = query["q"].split(' ').collect();
    let keys: Vec<_> = query
        .split(' ')
        .map(|l| {
            hash_map! {
                "Label".to_string() => AttributeValue {
                    s: Some(l.to_string()),
                    ..Default::default()
                }
            }
        })
        .collect();

    let db = DynamoDbClient::new(Region::UsEast1);
    let input = BatchGetItemInput {
        request_items: hash_map! {
            "Labels".to_string() => KeysAndAttributes {
                keys,
                ..Default::default()
            }
        },
        ..Default::default()
    };

    let output = db.batch_get_item(input).await?;
    let res = if let Some(responses) = output.responses {
        if let Some(data) = responses.get("Labels") {
            let simplified: Vec<_> = data
                .iter()
                .map(|m| {
                    m.clone()
                        .into_iter()
                        .filter_map(|(k, v)| {
                            let list = v.l?;
                            let list: Vec<_> = list.into_iter().filter_map(|v| v.s).collect();

                            Some((k, list))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .collect();

            json::to_value(simplified)?
        } else {
            json! {{}}
        }
    } else {
        json! {{}}
    };

    Ok(res.into_response())
}
