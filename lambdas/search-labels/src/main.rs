mod data;

use common_macros::hash_map;
use netlify_lambda::{lambda, Context};
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, BatchGetItemInput, KeysAndAttributes};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use std::collections::HashMap;

use crate::data::{SearchLabels, SearchResult, VideoInfo};

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: SearchLabels, _: Context) -> Result<SearchResult, DynError> {
    println!("Labels to search: {:?}", event);

    let label_keys: Vec<_> = event
        .labels
        .iter()
        .map(|label| {
            hash_map! {
                "Label".to_string() => AttributeValue {
                    s: Some(label.to_lowercase()),
                    ..Default::default()
                }
            }
        })
        .collect();

    println!("Label keys: {:?}", label_keys);
    let db = DynamoDbClient::new(Region::UsEast1);
    let input = BatchGetItemInput {
        request_items: hash_map! {
            "Labels".to_string() => KeysAndAttributes {
                keys: label_keys,
                ..Default::default()
            }
        },
        ..Default::default()
    };

    println!("BatchGetItem request: {:?}", input);
    let output = db.batch_get_item(input).await?;

    let labels_items = output
        .responses
        .map(|res| res.get("Labels").cloned())
        .flatten();

    println!("Items matching criteria: {:?}", labels_items);
    if let Some(labels_items) = labels_items {
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

        Ok(SearchResult { videos })
    } else {
        Ok(SearchResult::default())
    }
}
