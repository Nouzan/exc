use exc_core::types::Request;

use super::request::HttpRequest;
use super::response::HttpResponse;

impl Request for HttpRequest {
    type Response = HttpResponse;
}

mod candle;
mod trading;
