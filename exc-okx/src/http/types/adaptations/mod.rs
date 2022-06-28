use exc_core::Request;

use super::request::HttpRequest;
use super::response::HttpResponse;

impl Request for HttpRequest {
    type Response = HttpResponse;
}

mod candle;
mod trading;
