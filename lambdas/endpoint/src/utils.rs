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

#[macro_export]
macro_rules! validate_request {
    ($method:path, $content_type:expr, $req:expr) => {{
        use $crate::error::Error;

        if !matches!($req.method(), &$method) {
            return Err(Error::bad_request(format!(
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
            return Err(Error::bad_request(format!(
                "Invalid content, expecting {}",
                $content_type
            )));
        }
    }};

    ($method:path, $req:expr) => {{
        use $crate::error::Error;

        if !matches!($req.method(), &$method) {
            return Err(Error::bad_request(format!(
                "Invalid method: {}",
                $req.method()
            )));
        }
    }};
}
