use exc_core::types::candle::{Candle, QueryLastCandles};
use exc_core::types::CandleStream;
use exc_core::Adaptor;
use exc_core::ExchangeError;
use std::ops::RangeBounds;

use crate::http::types::request::history_candles::HistoryCandles;
use crate::http::types::request::Get;
use crate::http::types::response::ResponseData;
use crate::utils::timestamp::millis_to_ts;
use crate::utils::{
    period::period_to_bar,
    timestamp::{end_bound_to_millis, start_bound_to_millis},
};
use async_stream::stream;

use super::HttpRequest;

impl Adaptor<QueryLastCandles> for HttpRequest {
    fn from_request(req: QueryLastCandles) -> Result<Self, exc_core::ExchangeError>
    where
        Self: Sized,
    {
        let limit = req.last();
        let query = req.query();
        // from before to after.
        let start = start_bound_to_millis(query.start_bound());
        let end = end_bound_to_millis(query.end_bound());
        let req = Self::Get(Get::HistoryCandles(HistoryCandles {
            inst_id: query.inst().to_string(),
            after: end,
            before: start,
            bar: period_to_bar(&query.period()),
            limit: Some(limit),
        }));
        Ok(req)
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<QueryLastCandles as exc_core::Request>::Response, exc_core::ExchangeError> {
        let stream = stream! {
                for data in resp.data {
        trace!("received a data: {data:?}");
            if let ResponseData::Candle(c) = data {
                if let Some(ts) = millis_to_ts(c.0) {
                yield Ok(Candle {
                    ts,
                    open: c.1,
                    high: c.2,
                    low: c.3,
                    close: c.4,
                    volume: c.5,
                });
                } else {
                yield Err(ExchangeError::Other(anyhow::anyhow!("cannot parse ts")));
                }
            }
            }
                };
        Ok(CandleStream::new_backward(stream))
    }
}
