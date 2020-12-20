use std::collections::HashMap;
use std::time::Duration;

use common_macros::hash_map;
use json::Value;
use netlify_lambda_http::http::Method;
use netlify_lambda_http::{Body, IntoResponse, Request, RequestExt, Response};
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
use crate::utils::{generate_id, is_valid_id};
use crate::validate_request;

pub async fn request_upload(req: Request) -> Result<Response<Body>, Error> {
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

pub async fn gen_thumbnails(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::GET, req);

    let query = req.query_string_parameters();
    println!("Query parameters: {:?}", query);

    let video_id = query
        .get("video_id")
        .ok_or_else(|| Error::bad_request("Missing `video_id` param"))?;

    println!("Received video_id: {}", video_id);
    if !is_valid_id(video_id) {
        return Err(Error::bad_request("Invalid `video_id` param"));
    }

    let lambda = LambdaClient::new(Region::UsEast1);
    let payload = json!({ "video_key": format!("{}.mp4", video_id) });
    println!("Payload for lambda: {:#?}", payload);

    let input = InvocationRequest {
        function_name: "GenerateThumbnailsLambda".to_string(),
        payload: Some(payload.to_string().into()),
        ..Default::default()
    };

    println!("Invocation request: {:#?}", input);
    let output = lambda.invoke(input).await?;
    println!("Lambda output: {:#?}", output);

    let payload = output
        .payload
        .map(|bytes| String::from_utf8(bytes.to_vec()))
        .transpose()?
        .unwrap_or_default();
    println!("Lambda payload: {:?}", payload);

    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("{} ({})", err, payload)))
    } else {
        Ok(json::from_str::<Value>(&payload)?.into_response())
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

    let body: UploadThumbnail = if let Body::Text(json) = req.body() {
        json::from_str(&json).map_err(|e| Error::bad_request(e.to_string()))?
    } else {
        return Err(Error::bad_request("Invalid JSON body"));
    };

    println!("Parsed body: {:#?}", body);
    let decoded_data =
        base64::decode(body.thumbnail_data).map_err(|e| Error::bad_request(e.to_string()))?;

    let s3 = S3Client::new(Region::UsEast1);
    let input = PutObjectRequest {
        bucket: "minitube.thumbnails".to_string(),
        key: format!("{}.png", body.video_id),
        body: Some(ByteStream::from(decoded_data)),
        acl: Some("public-read".to_string()),
        content_type: Some("image/png".to_string()),
        ..Default::default()
    };

    println!("PutObject request: {:?}", input);
    s3.put_object(input).await?;

    let lambda = LambdaClient::new(Region::UsEast1);
    let payload = json!({
        "video_id": body.video_id.clone(),
        "bucket": "minitube.thumbnails"
    });
    println!("Payload for lambda: {:#?}", payload);

    let input = InvocationRequest {
        function_name: "LabelThumbnailLambda".to_string(),
        payload: Some(payload.to_string().into()),
        ..Default::default()
    };

    println!("Invocation request: {:#?}", input);
    let output = lambda.invoke(input).await?;
    println!("Lambda output: {:#?}", output);

    let payload = output
        .payload
        .map(|bytes| String::from_utf8(bytes.to_vec()))
        .transpose()?
        .unwrap_or_default();
    println!("Lambda payload: {:?}", payload);

    if let Some(err) = output.function_error {
        Err(Error::internal_error(format!("{} ({})", err, payload)))
    } else {
        Ok(json::from_str::<Value>(&payload)?.into_response())
    }
}

#[derive(Debug, Clone, Serialize)]
struct VideoInfo {
    labels: Vec<String>,
    video_url: String,
    preview_url: String,
    thumbnail_url: String,
}

impl VideoInfo {
    const S3_URL: &'static str = "https://s3.amazonaws.com";

    fn new(labels: Vec<String>, video_id: &str) -> Self {
        let video_url = format!("{}/minitube.videos/{}.mp4", Self::S3_URL, video_id);
        let preview_url = format!("{}/minitube.previews/{}.gif", Self::S3_URL, video_id);
        let thumbnail_url = format!("{}/minitube.thumbnails/{}.png", Self::S3_URL, video_id);

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

    let query = req.query_string_parameters();
    println!("Query parameters: {:?}", query);

    let query = query
        .get("q")
        .ok_or_else(|| Error::bad_request("Missing `q` param"))?;

    let labels: Vec<_> = query
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

    println!("Labels to search: {:?}", labels);
    let db = DynamoDbClient::new(Region::UsEast1);
    let input = BatchGetItemInput {
        request_items: hash_map! {
            "Labels".to_string() => KeysAndAttributes {
                keys: labels,
                ..Default::default()
            }
        },
        ..Default::default()
    };

    println!("BatchGetItem request: {:#?}", input);
    let output = db.batch_get_item(input).await?;

    let labels_items = output
        .responses
        .map(|res| res.get("Labels").cloned())
        .flatten();

    println!("Items matching criteria: {:#?}", labels_items);
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

        println!("Video to labels: {:#?}", video_to_labels);
        let videos: Vec<_> = video_to_labels
            .into_iter()
            .map(|(video_id, labels)| VideoInfo::new(labels, &video_id))
            .collect();

        json!({ "videos": videos })
    } else {
        json!({ "videos": Vec::<VideoInfo>::new() })
    };

    Ok(json.into_response())
}
