use std::ops::RangeBounds;

use exc::types::candle::{Candle, QueryLastCandles};
use exc::types::Adaptor;
use exc::ExchangeError;

use crate::http::types::request::history_candles::HistoryCandles;
use crate::http::types::request::Get;
use crate::http::types::response::ResponseData;
use crate::util::timestamp::millis_to_ts;
use crate::util::{
    period::period_to_bar,
    timestamp::{end_bound_to_millis, start_bound_to_millis},
};
use async_stream::stream;
use futures::StreamExt;

use super::HttpRequest;

impl Adaptor<QueryLastCandles> for HttpRequest {
    fn from_request(req: QueryLastCandles) -> Result<Self, exc::ExchangeError>
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
        mut resp: Self::Response,
    ) -> Result<<QueryLastCandles as exc::types::Request>::Response, exc::ExchangeError> {
        match resp.code.as_str() {
            "0" => {
                let stream = stream! {
                        while let Some(data) = resp.data.pop() {
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
                Ok(stream.boxed())
            }
            _ => Err(ExchangeError::Other(anyhow::anyhow!(
                "code={}, msg={}",
                resp.code,
                resp.msg
            ))),
        }
    }
}
