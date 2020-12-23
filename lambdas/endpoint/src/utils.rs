use netlify_lambda_http::{Body, IntoResponse, Response};

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

pub trait IntoCorsResponse {
    fn into_cors_response(self) -> Response<Body>;
}

impl<T> IntoCorsResponse for T
where
    T: IntoResponse,
{
    fn into_cors_response(self) -> Response<Body> {
        let mut res = self.into_response();
        let headers = res.headers_mut();

        headers.insert("Access-Control-Allow-Headers", "*".parse().unwrap());
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        headers.insert(
            "Access-Control-Allow-Methods",
            "OPTIONS,GET,POST".parse().unwrap(),
        );

        res
    }
}

#[macro_export]
macro_rules! handle_preflight_request {
    ($req:expr) => {{
        use netlify_lambda_http::http::Method;
        use $crate::utils::IntoCorsResponse;

        if $req.method() == &Method::OPTIONS {
            return Ok(().into_cors_response());
        }
    }};
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

#[macro_export]
macro_rules! invoke_lambda {
    ($name:expr, $payload:expr) => {{
        use rusoto_core::Region;
        $crate::invoke_lambda!($name, $payload, Region::UsEast1)
    }};

    ($name:expr, $payload:expr, $region:expr) => {{
        use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};

        println!("Starting invokation of {}", $name);
        let lambda = LambdaClient::new($region);
        println!("Input payload: {:?}", $payload);

        let input = InvocationRequest {
            function_name: $name.to_string(),
            payload: Some($payload.to_string().into()),
            ..Default::default()
        };

        println!("Invocation request: {:?}", input);
        let output = lambda.invoke(input).await?;
        println!("Lambda output: {:?}", output);

        let payload = output
            .payload
            .as_ref()
            .map(|bytes| String::from_utf8(bytes.to_vec()))
            .transpose()?
            .unwrap_or_default();

        println!("Output payload: {:?}", payload);
        (output, payload)
    }};
}
