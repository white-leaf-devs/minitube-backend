use std::io::Read;

use json::Value;
use lambda_http::http::Method;
use lambda_http::{Body, IntoResponse, Request, Response};
use multipart::server::Multipart;
use rusoto_cloudsearchdomain::{CloudSearchDomain, CloudSearchDomainClient, SearchRequest};
use rusoto_core::{ByteStream, Region};
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use serde::Deserialize;
use serde_json::{self as json, json};

use crate::error::Error;
use crate::utils::{build_http_buffer, generate_id, is_valid_id, query_params};
use crate::validate_request;

pub async fn upload_video(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::POST, "multipart/form-data", req);

    log::debug!("Building http buffer");
    let http_buffer = build_http_buffer(req)?;
    log::debug!("Parsing multipart data");
    let mut multipart = Multipart::from_request(http_buffer.for_server())
        .map_err(|_| Error::invalid_request("Invalid multipart request"))?;

    let mut buf = Vec::new();
    while let Some(mut field) = multipart.read_entry()? {
        field.data.read_to_end(&mut buf)?;
    }

    log::debug!("Uploading data to S3 (bucket: videos)");
    let s3 = S3Client::new(Region::UsEast1);
    let id = generate_id();

    let input = PutObjectRequest {
        bucket: "videos".to_string(),
        key: id.clone(),
        body: Some(ByteStream::from(buf)),
        ..Default::default()
    };

    s3.put_object(input).await?;
    let res = json! {{
        "video_id": id
    }};

    Ok(res.into_response())
}

pub async fn gen_thumbnails(req: Request) -> Result<Response<Body>, Error> {
    validate_request!(Method::GET, req);

    let query = query_params(req.uri().query().unwrap_or(""));
    if !query.get("video_id").map_or(false, |v| is_valid_id(v)) {
        return Err(Error::invalid_request(
            "Invalid or not present `video_id` query param",
        ));
    }

    let lambda = LambdaClient::new(Region::UsEast1);

    let payload = json! {{
        "video_id": query["video_id"]
    }};

    let input = InvocationRequest {
        function_name: "GenerateThumbnails".to_string(),
        payload: Some(payload.to_string().into()),
        ..Default::default()
    };

    let res = lambda.invoke(input).await?;
    if let Some(err) = res.function_error {
        Err(Error::internal_error(format!("Function error: {}", err)))
    } else {
        let payload = res
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
        bucket: "thumbnails".to_string(),
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

    let cs = CloudSearchDomainClient::new(Region::UsEast1);
    let input = SearchRequest {
        query,
        ..Default::default()
    };

    let res = cs.search(input).await?;

    todo!()
}
