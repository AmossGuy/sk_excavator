pub mod editable;
pub mod tree;

pub type ArcBytes = yoke::Yoke<&'static [u8], std::sync::Arc<Vec<u8>>>;
