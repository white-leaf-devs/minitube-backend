use rusoto_rekognition::Label;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct S3Object {
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Bucket {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Record {
    pub bucket: S3Bucket,
    pub object: S3Object,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub s3: S3Record,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Records {
    #[serde(rename = "Records")]
    pub records: Vec<Record>,
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
            .filter_map(|label| label.name)
            .collect();

        let parent_labels = prelabels
            .iter()
            .cloned()
            .filter_map(|label| {
                label
                    .parents
                    .map(|parents| parents.into_iter().filter_map(|parent| parent.name))
            })
            .flatten();

        labels.extend(parent_labels);
        Self { labels }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use anyhow::Result;
    use serde_json as json;

    #[test]
    fn parse_s3_event() -> Result<()> {
        let json = File::open("test/s3.json")?;
        let records: Records = json::from_reader(json)?;
        println!("{:#?}", records);

        Ok(())
    }
}
