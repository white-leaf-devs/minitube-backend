use std::collections::HashMap;
use std::io::{Cursor, Read};

use multipart::server::Multipart;
use netlify_lambda_http::{Body, Request};

use crate::error::Error;

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

pub fn get_boundary(content_type_value: &str) -> Option<&str> {
    content_type_value.split(' ').nth(1)?.split('=').nth(1)
}

pub fn parse_multipart(req: Request) -> Result<Multipart<impl Read>, Error> {
    log::debug!("{:#?}", req);

    let ct_value = req
        .headers()
        .get("Content-Type")
        .ok_or_else(|| Error::invalid_request("Not found content-type header"))?
        .to_str()
        .map_err(|_| Error::invalid_request("Invalid content-type header value"))?;

    let boundary = get_boundary(ct_value)
        .ok_or_else(|| Error::invalid_request("Invalid multipart header value"))?;

    match req.body() {
        Body::Text(buf) => {
            let buf = buf.as_bytes().to_vec();
            let cursor = Cursor::new(buf);
            Ok(Multipart::with_body(cursor, boundary))
        }

        _ => Err(Error::invalid_request("Invalid body")),
    }
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
    use common_macros::hash_map;

    use super::{get_boundary, query_params};

    #[test]
    fn boundary_extraction() {
        let header_value = "multipart/form-data; boundary=2a8ae6ad-f4ad-4d9a-a92c-6d217011fe0f";
        assert_eq!(
            Some("2a8ae6ad-f4ad-4d9a-a92c-6d217011fe0f"),
            get_boundary(header_value)
        );
    }

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
