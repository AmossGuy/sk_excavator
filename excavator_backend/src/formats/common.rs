pub mod editable;

pub type ArcBytes = yoke::Yoke<&'static [u8], std::sync::Arc<Vec<u8>>>;

pub trait TreeFormat: Send + Sync + 'static {
	type ItemId;
	fn root_id(&self) -> Self::ItemId;
}
