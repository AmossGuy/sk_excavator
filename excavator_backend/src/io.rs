pub mod dir;

pub enum LoadState<T> {
	Loading,
	Loaded(T),
	Failed(anyhow::Error),
}

pub enum LoadStateRef<'a, T> {
	Loading,
	Loaded(&'a T),
	Failed(&'a anyhow::Error),
}

impl<T> LoadState<T> {
	pub fn load_result<E>(result: Result<T, E>) -> Self
		where E: Into<anyhow::Error>,
	{
		match result {
			Ok(loaded) => Self::Loaded(loaded),
			Err(e) => Self::Failed(e.into()),
		}
	}
	
	pub fn set_load_result<E>(&mut self, result: Result<T, E>)
		where E: Into<anyhow::Error>,
	{
		*self = Self::load_result(result);
	}
	
	pub fn state_ref(&self) -> LoadStateRef<'_, T> {
		match self {
			Self::Loading => LoadStateRef::Loading,
			Self::Loaded(loaded) => LoadStateRef::Loaded(loaded),
			Self::Failed(failed) => LoadStateRef::Failed(failed),
		}
	}
}
