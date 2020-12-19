mod data;

use netlify_lambda::{lambda, Context};
use rusoto_core::Region;
use rusoto_rekognition::{DetectLabelsRequest, Image, Rekognition, RekognitionClient, S3Object};

use crate::data::{Labels, Records};

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: Records, _: Context) -> Result<Labels, DynError> {
    let record = event.records[0].clone();

    let rekognition = RekognitionClient::new(Region::UsEast1);
    let input = DetectLabelsRequest {
        image: Image {
            s3_object: Some(S3Object {
                name: Some(record.s3.object.key),
                bucket: Some(record.s3.bucket.name),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    let output = rekognition.detect_labels(input).await?;

    Ok(if let Some(labels) = output.labels {
        Labels::from(labels)
    } else {
        Labels::default()
    })
}
