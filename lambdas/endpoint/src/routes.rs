use std::collections::HashMap;
use std::time::Duration;

use common_macros::hash_map;
use json::Value;
use netlify_lambda_http::http::Method;
use netlify_lambda_http::{Body, IntoResponse, Request, Response};
use rusoto_core::{ByteStream, Region};
use rusoto_credential::{ChainProvider, ProvideAwsCredentials};
use rusoto_dynamodb::{
    AttributeValue, BatchGetItemInput, DynamoDb, DynamoDbClient, KeysAndAttributes,
};
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use serde::{Deserialize, Serialize};
use serde_json::{self as json, json};

use crate::error::Error;
use crate::utils::{generate_id, is_valid_id, query_params};
use crate::validate_request;

/// Expects base64 encode contents
pub async fn request_upload(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::GET, req);

    log::debug!("Generating video id");
    let id = generate_id();

    let input = PutObjectRequest {
        bucket: "minitube.videos".to_string(),
        key: id.clone(),
        acl: Some("public-read".to_string()),
        ..Default::default()
    };

    let mut provider = ChainProvider::new();
    provider.set_timeout(Duration::from_secs(600));
    let credentials = provider.credentials().await?;
    let options = PreSignedRequestOption {
        expires_in: Duration::from_secs(600),
    };

    let presigned_url = input.get_presigned_url(&Region::UsEast1, &credentials, &options);
    let res = json! {{
        "video_id": id,
        "presigned_url": presigned_url,
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
        acl: Some("public-read".to_string()),
        ..Default::default()
    };

    s3.put_object(input).await?;
    let res = json! {{
        "video_id": id
    }};

    Ok(res.into_response())
}

#[derive(Debug, Clone, Serialize)]
struct VideoInfo {
    labels: Vec<String>,
    video_url: String,
    preview_url: String,
    thumbnail_url: String,
}

impl VideoInfo {
    fn new(labels: Vec<String>, video_id: &str) -> Self {
        let video_url = format!("https://s3.amazonaws.com/minitube.videos/{}", video_id);
        let preview_url = format!(
            "https://s3.amazonaws.com/minitube.previews/{}.gif",
            video_id
        );
        let thumbnail_url = format!(
            "https://s3.amazonaws.com/minitube.thumbnails/{}.png",
            video_id
        );

        Self {
            labels,
            video_url,
            preview_url,
            thumbnail_url,
        }
    }
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
        .map(|label| {
            hash_map! {
                "Label".to_string() => AttributeValue {
                    s: Some(label.to_string()),
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
    let labels_items = output
        .responses
        .map(|res| res.get("Labels").cloned())
        .flatten();

    let json = if let Some(labels_items) = labels_items {
        let simplified = labels_items.into_iter().filter_map(|item| {
            let label = item.get("Label").cloned()?.s?;
            let videos = item.get("Videos").cloned()?.ss?;

            Some((label, videos))
        });

        let mut video_to_labels = HashMap::new();
        for (label, videos) in simplified {
            for video in videos {
                video_to_labels
                    .entry(video)
                    .or_insert_with(Vec::new)
                    .push(label.clone());
            }
        }

        let videos: Vec<_> = video_to_labels
            .into_iter()
            .map(|(video_id, labels)| VideoInfo::new(labels, &video_id))
            .collect();

        json! {{
            "videos": videos
        }}
    } else {
        json! {{
            "videos": Vec::<VideoInfo>::new()
        }}
    };

    Ok(json.into_response())
}
