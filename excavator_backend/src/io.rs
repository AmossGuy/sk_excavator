pub mod dir;

use std::sync::Arc;

#[derive(Clone)]
pub enum LoadState<T> {
	Unloaded,
	Loading,
	Loaded(T),
	Failed(Arc<anyhow::Error>),
}

impl<T> LoadState<T> {
	pub fn from_load_result<E>(result: Result<T, E>) -> Self
		where E: Into<anyhow::Error>,
	{
		match result {
			Ok(loaded) => Self::Loaded(loaded),
			Err(e) => Self::Failed(Arc::new(e.into())),
		}
	}
	
	pub fn set_from_load_result<E>(&mut self, result: Result<T, E>)
		where E: Into<anyhow::Error>,
	{
		*self = Self::from_load_result(result);
	}
}
