use std::borrow::Cow;

/// Borrowed fixed-point dollar string (e.g. "0.5600").
pub type FixedPointDollarsRef<'a> = Cow<'a, str>;

/// Borrowed fixed-point contract count string (e.g. "10.00").
pub type FixedPointCountRef<'a> = Cow<'a, str>;

mod ticker;
pub use ticker::*;

mod trade;
pub use trade::*;

mod orderbook;
pub use orderbook::*;

mod fill;
pub use fill::*;

mod lifecycle;
pub use lifecycle::*;

mod positions;
pub use positions::*;

mod order_groups;
pub use order_groups::*;

mod user_orders;
pub use user_orders::*;

mod communications;
pub use communications::*;

mod cfbenchmarks;
pub use cfbenchmarks::*;
