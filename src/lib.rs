#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

mod color;
mod error;
mod serialization;
mod structs;

pub use color::Color;
pub use error::*;
pub use petgraph::*;
pub use structs::node::Node;
pub use structs::node::NodeOptions;
pub use structs::node::NodeType;
pub use structs::polyfill::Polyfill;
pub use structs::polyfill::PolyfillOptions;
pub use structs::stickfigure::DrawOrderIndex;
pub use structs::stickfigure::IWillNotAbuseUnlimitedNodes;
pub use structs::stickfigure::Stickfigure;
pub use structs::stickfigure::SUPPORTED_APP_VERSION;
