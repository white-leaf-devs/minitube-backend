mod data;

use common_macros::hash_map;
use netlify_lambda::{lambda, Context};
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, TransactWriteItem, TransactWriteItemsInput, Update,
};
use rusoto_rekognition::{DetectLabelsRequest, Image, Rekognition, RekognitionClient, S3Object};

use crate::data::{Labels, ThumbnailEvent};

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: ThumbnailEvent, _: Context) -> Result<Labels, DynError> {
    let rekognition = RekognitionClient::new(Region::UsEast1);
    let input = DetectLabelsRequest {
        image: Image {
            s3_object: Some(S3Object {
                name: Some(event.thumbnail_key.clone()),
                bucket: Some(event.bucket.clone()),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    let output = rekognition.detect_labels(input).await?;
    let labels = if let Some(labels) = output.labels {
        Labels::from(labels)
    } else {
        Labels::default()
    };

    let db = DynamoDbClient::new(Region::UsEast1);
    for chunk in labels.labels.chunks(25) {
        let transact_items = chunk
            .iter()
            .map(|label| {
                let update = Update {
                    table_name: "Labels".to_string(),
                    key: hash_map! {
                        "Label".to_string() => AttributeValue{
                            s: Some(label.to_owned()),
                            ..Default::default()
                        }
                    },
                    update_expression: "ADD Videos :video".to_string(),
                    expression_attribute_values: Some(hash_map! {
                        ":video".to_string() => AttributeValue {
                            ss: Some(vec![ event.video_id.clone() ]),
                            ..Default::default()
                        }
                    }),
                    ..Default::default()
                };

                TransactWriteItem {
                    update: Some(update),
                    ..Default::default()
                }
            })
            .collect();

        let input = TransactWriteItemsInput {
            transact_items,
            ..Default::default()
        };

        db.transact_write_items(input).await?;
    }

    Ok(labels)
}
