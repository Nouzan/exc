/// Place: the order builder.
pub mod place;

/// Order.
pub mod order;

use either::Either;
pub use order::{Order, OrderId, OrderKind};
pub use place::Place;
use positions::{Normal, Reversed};

use super::Request;

impl Request for Place {
    type Response = Either<Order<Normal>, Order<Reversed>>;
}
