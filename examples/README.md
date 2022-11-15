# Examples for using `exc`

## Simple Script Trading
We are going to use the following script as an example:
```toml
# Content of `script.toml`
exec = [
  { op = "wait", millis = 1000 },
  { op = "limit", name = "foo", price = "0.10", size = "-10" },
  { op = "wait", millis = 1000 },
  { op = "check", name = "foo" },
  { op = "cancel", name = "foo" },
  { op = "wait", millis = 1000 },
]
```
The following command will execute the above trading script (which has been saved as `script.toml`) on the *DOGE-USDT* instrument of *OKX* .
```bash
KEY=$OKX_KEY cargo run --example exc_trading -- DOGE-USDT \
  -s script.toml \
  --exchange okx
```
The format of the *APIKey* is
```
{"apikey":"xxxxxxx","secretkey":"yyyyyyy","passphrase":"zzzzzz"}
```
, which has been stored in the environment variable `OKX_KEY` in advance.
