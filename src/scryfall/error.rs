use thiserror::Error;

use std::time::Duration;

use super::convert;

type HttpResponse = reqwest::blocking::Response;


#[derive(Debug,Error)]
pub enum ScryfallError {
    #[error("hit maximum allowable retires while calling API, attempted to acquire rate limit {times_tried}")]
    MaxRetries { times_tried: u32, time_elapsed: Duration },
    #[error("http error while communicating with remote")]
    HttpError(#[from] reqwest::Error),
    #[error("http error while communicating with remote")]
    HttpErrorWithResponse {
        http_error: reqwest::Error,
        response: serde_json::Value
    },
    #[error("deseralization error")]
    Deserialization(#[from] serde_json::Error),
    #[error("error converting from scryfall api model")]
    ConversionFromScryfallApiModel(#[from] convert::ConversionError)
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

