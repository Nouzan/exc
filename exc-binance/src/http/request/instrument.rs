use crate::http::error::RestError;

use super::{Rest, RestEndpoint};

/// Exchange info.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExchangeInfo;

impl Rest for ExchangeInfo {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn path(&self, endpoint: &RestEndpoint) -> Result<&str, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/exchangeInfo"),
            RestEndpoint::Spot => Ok("/api/v1/exchangeInfo"),
        }
    }

    fn body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }
}

#[cfg(test)]
mod test {
    use tower::ServiceExt;

    use crate::{
        http::request::{Payload, RestRequest},
        Binance, Request,
    };

    use super::*;

    #[tokio::test]
    async fn test_exchange_info() -> anyhow::Result<()> {
        let api = Binance::usd_margin_futures().connect();
        let resp = api
            .oneshot(Request::Http(RestRequest::from(Payload::new(
                ExchangeInfo::default(),
            ))))
            .await?
            .into_response::<crate::http::response::instrument::ExchangeInfo>()?;
        println!("{:?}", resp);
        Ok(())
    }
}
