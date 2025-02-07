use std::time::Duration;

type HttpResponse = reqwest::blocking::Response;


#[derive(Debug)]
pub enum ScryfallError {
    MaxRetries {
        times_tried: u32,
        time_elapsed: Duration
    },
    HttpError(reqwest::Error),
    HttpErrorWithResponse {
        http_error: reqwest::Error,
        response: serde_json::Value
    },
    Deserialization(serde_json::Error)
}
impl ScryfallError {
    pub fn raise_on_error(response: HttpResponse) -> Result<HttpResponse, ScryfallError> {
        let http_error = match response.error_for_status_ref() {
            Ok(_) => return Ok(response),
            Err(e) => e
        };
        let text = match response.text() {
            Ok(t) => t,
            Err(decode_error) => {
                log::error!("could not decode response during error recovery: {decode_error}");
                // simply replace it with the original error
                return Err(http_error.into());
            }
        };
        let json_value : serde_json::Value = match serde_json::from_str(text.as_str()) {
            Ok(v) => v,
            Err(parse_error) => {
                log::error!("could not parse json during error recovery: {parse_error}");
                // again, simply replace with original error
                return Err(http_error.into());
            }
        };
        Err(ScryfallError::HttpErrorWithResponse { http_error, response: json_value })
    }
}

impl std::fmt::Display for ScryfallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScryfallError::MaxRetries { times_tried, time_elapsed } => write!(f, "hit maximum retries while calling api. tried: {times_tried} attempts to acquire rate limit, {time_elapsed:?} has elapsed since the call began"),
            ScryfallError::HttpError(e) => e.fmt(f),
            ScryfallError::HttpErrorWithResponse { http_error, response } => write!(f, "{http_error}: {response}"),
            ScryfallError::Deserialization(e) => write!(f, "failed to deserialize response from scryfall: {e}"),
        }
    }
}
impl std::error::Error for ScryfallError { }

impl From<reqwest::Error> for ScryfallError {
    fn from(e: reqwest::Error) -> Self {
        ScryfallError::HttpError(e)
    }
}
impl From<serde_json::Error> for ScryfallError {
    fn from(e: serde_json::Error) -> Self {
        ScryfallError::Deserialization(e)
    }
}


