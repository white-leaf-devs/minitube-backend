use rusoto_rekognition::Label;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailEvent {
    pub video_id: String,
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Labels {
    pub labels: Vec<String>,
}

impl From<Vec<Label>> for Labels {
    fn from(prelabels: Vec<Label>) -> Self {
        let mut labels: Vec<_> = prelabels
            .iter()
            .cloned()
            .filter_map(|label| label.name.map(|s| s.to_lowercase()))
            .collect();

        let parent_labels = prelabels
            .iter()
            .cloned()
            .filter_map(|label| {
                label.parents.map(|parents| {
                    parents
                        .into_iter()
                        .filter_map(|parent| parent.name.map(|s| s.to_lowercase()))
                })
            })
            .flatten();

        labels.extend(parent_labels);
        labels.sort();
        labels.dedup();
        Self { labels }
    }
}
