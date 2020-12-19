use std::collections::HashMap;

pub const ALPHABET: [char; 63] = [
    '_', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A',
    'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub const ID_LEN: usize = 22;

pub fn generate_id() -> String {
    nanoid::nanoid!(ID_LEN, &ALPHABET)
}

pub fn is_valid_id(id: &str) -> bool {
    id.len() == ID_LEN && id.chars().all(|c| ALPHABET.contains(&c))
}

pub fn query_params<'a>(query: &'a str) -> HashMap<&'a str, &'a str> {
    query
        .split('&')
        .filter_map(|kv| {
            let split: Vec<_> = kv.split('=').collect();
            if split.len() != 2 {
                None
            } else {
                Some((split[0], split[1]))
            }
        })
        .collect()
}

#[macro_export]
macro_rules! validate_request {
    ($method:path, $content_type:expr, $req:expr) => {{
        use $crate::error::Error;

        if !matches!($req.method(), &$method) {
            return Err(Error::invalid_request(format!(
                "Invalid method: {}",
                $req.method()
            )));
        }

        let headers = $req.headers();
        let valid_content = headers
            .get("Content-Type")
            .map(|val| val.to_str().map_or(false, |v| v.contains($content_type)))
            .unwrap_or(false);

        if !valid_content {
            return Err(Error::invalid_request(format!(
                "Invalid content, expecting {}",
                $content_type
            )));
        }
    }};

    ($method:path, $req:expr) => {{
        use $crate::error::Error;

        if !matches!($req.method(), &$method) {
            return Err(Error::invalid_request(format!(
                "Invalid method: {}",
                $req.method()
            )));
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::query_params;
    use common_macros::hash_map;

    #[test]
    fn single_query_params() {
        let query = "key=val";
        let expected = hash_map! {
            "key" => "val"
        };

        assert_eq!(expected, query_params(query));
    }

    #[test]
    fn multi_query_params() {
        let query = "key1=val1&key2=&key3=val3";
        let expected = hash_map! {
            "key1" => "val1",
            "key2" => "",
            "key3" => "val3",
        };

        assert_eq!(expected, query_params(query));
    }
}
