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
    fn from(rekognition_labels: Vec<Label>) -> Self {
        let mut labels: Vec<_> = rekognition_labels
            .into_iter()
            .filter_map(|label| {
                let label_str = label.name?;

                let mut labels = label_str
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_lowercase())
                    .collect::<Vec<_>>();

                let parents = label
                    .parents
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|p| p.name)
                    .map(|s| {
                        s.split(' ')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_lowercase())
                            .collect::<Vec<_>>()
                    })
                    .flatten();

                labels.extend(parents);
                Some(labels)
            })
            .flatten()
            .collect();

        labels.sort();
        labels.dedup();

        Self { labels }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_rekognition::{Label, Parent};

    #[test]
    fn labels() {
        let labels = vec![
            Label {
                name: Some("Composed Label".to_string()),
                parents: Some(vec![Parent {
                    name: Some("Complex Tag".to_string()),
                }]),
                ..Default::default()
            },
            Label {
                name: Some("Simple".to_string()),
                parents: Some(vec![Parent {
                    name: Some("Plain".to_string()),
                }]),
                ..Default::default()
            },
            Label {
                name: Some("Simple".to_string()),
                ..Default::default()
            },
        ];

        let labels = Labels::from(labels);

        let expected = ["complex", "composed", "label", "plain", "simple", "tag"]
            .iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(expected, labels.labels);
    }
}
