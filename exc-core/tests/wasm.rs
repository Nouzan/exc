use exc_core::types::Trade;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::macros::datetime;
use wasm_bindgen_test::*;

/// It is just a simple test to check if we can compile `exc-core` to wasm.
#[wasm_bindgen_test]
fn it_works() {
    let trade = Trade {
        ts: datetime!(2023-10-21 00:00:00 +00:00),
        price: dec!(1),
        size: dec!(1),
        buy: true,
    };
    assert_eq!(trade.price, Decimal::ONE);
}
