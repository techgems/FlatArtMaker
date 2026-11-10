//! [`ColourMapper`](flatart_ports::ColourMapper) implementation (Pipeline
//! step 3): merges detected colours into output colours, then replays the
//! edit log on top.
//!
//! - `auto`: the automatic first pass: merges the most similar colours in
//!   OKLab down to the user's count, and records the merge order.
//! - `replay`: applies each active edit-log operation, or reports why not.
//! - `names`: automatic names from the nearest common colour name.
//! - `base`: the default base colour.

mod auto;
mod base;
mod colour_space;
mod mapper;
mod names;
mod rematch;
mod replay;

pub use mapper::MergeMapper;
pub use rematch::REMATCH_LIMIT;

#[cfg(test)]
mod tests;
