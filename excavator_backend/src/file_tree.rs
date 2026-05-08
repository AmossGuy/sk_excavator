use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;
use std::rc::Rc;

use crate::io::LoadState;
use crate::io::dir::{DirContents, DirItem};
use crate::request_thread::{ThreadRequest, ThreadRequester, Waker};

pub struct FileTreeBackend<W: Waker> {
	loader: ThreadRequester<LoadRequest, W>,
	
	// Rc used because the lifetimes wouldn't work for request_dir otherwise
	root: Rc<LoadState<DirItem>>,
	dirs: HashMap<PathBuf, Rc<LoadState<DirContents>>>,
}

impl<W: Waker> FileTreeBackend<W> {
	pub fn new(root_path: PathBuf, waker: W) -> Self {
		let request = LoadRequest::Root(root_path);

		Self {
			loader: ThreadRequester::new_with_request_and_waker(request, waker),
			root: Rc::new(LoadState::Loading),
			dirs: HashMap::new(),
		}
	}
	
	pub fn replace_waker(&mut self, waker: W) {
		self.loader.replace_waker(waker);
	}
	
	pub fn update_loading(&mut self) {
		for load_result in self.loader.take_results() {
			match load_result {
				LoadResult::Root { result } => {
					self.root = Rc::new(LoadState::load_result(result));
				},
				LoadResult::Dir { result, path } => {
					self.dirs.insert(path, Rc::new(LoadState::load_result(result)));
				},
			}
		}
	}
	
	pub fn request_root(&mut self) -> &Rc<LoadState<DirItem>> {
		&self.root
	}
	
	pub fn request_dir(&mut self, path: PathBuf) -> &Rc<LoadState<DirContents>> {
		match self.dirs.entry(path) {
			Entry::Occupied(occupied) => occupied.into_mut(),
			Entry::Vacant(vacant) => {
				self.loader.make_request(LoadRequest::Dir(vacant.key().clone()));
				vacant.insert(Rc::new(LoadState::Loading))
			},
		}
	}
}

#[derive(Clone, Hash)]
enum LoadRequest {
	Root(PathBuf),
	Dir(PathBuf),
	// Pak(PathBuf),
}

enum LoadResult {
	Root { result: std::io::Result<DirItem> },
	Dir { result: std::io::Result<DirContents>, path: PathBuf },
	// Pak { result: anyhow::Result<> },
}

impl ThreadRequest for LoadRequest {
	type Output = LoadResult;
	
	fn execute(self) -> LoadResult {
		match self {
			Self::Root(path) => LoadResult::Root {
				result: DirItem::read_single(&path),
			},
			Self::Dir(path) => LoadResult::Dir {
				result: DirContents::read(&path).map(|c| c.sorted_by_name()),
				path,
			},
			// Self::Pak(_) => todo!(),
		}
	}
}
