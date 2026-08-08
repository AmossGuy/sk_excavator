use bstr::BString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

use crate::io::LoadState;
use crate::io::dir::{DirContents, DirItem};
use crate::request_thread::{ThreadRequest, ThreadRequester, Waker};

// So I read this article:
// https://preshing.com/20111118/locks-arent-slow-lock-contention-is
// My takeaway from it is that mutexes can be efficient as long as the amount of time spent in critical sections is small.
// Accordingly, while the design of what follows uses many Arcs and Mutexes, I have made sure the critical sections are incredibly simple.
// If someone profiles it in the future and it turns out it performs badly, then that'll be my fault - but I'll never regret it.
// This design is a lot easier to work with than what I tried before it.

pub struct FileTreeBackend<W: Waker> {
	loader: ThreadRequester<LoadRequest, W>,
	tree_root: Arc<Mutex<TreeRoot>>,
}

impl<W: Waker> FileTreeBackend<W> {
	pub fn new(root_path: PathBuf, waker: W) -> Self {
		let tree_root = Arc::new(Mutex::new(TreeRoot { node: LoadState::Loading }));
		let request = LoadRequest::Root { path: root_path.clone(), target: Arc::downgrade(&tree_root) };
		let loader = ThreadRequester::new_with_request_and_waker(request, waker);
		
		Self { loader, tree_root }
	}
	
	pub fn replace_waker(&mut self, waker: W) {
		self.loader.replace_waker(waker);
	}
	
	pub fn tree_root(&mut self) -> LoadState<Arc<Mutex<TreeNode>>> {
		LoadState::clone(&self.tree_root.lock().unwrap().node)
	}
	
	pub fn start_load_if_unloaded(&mut self, node: &Arc<Mutex<TreeNode>>) {
		let mut node_lock = node.lock().unwrap();
		if matches!(node_lock.children, LoadState::Unloaded) {
			match &node_lock.source {
				TreeNodeSource::Fs(item) => {
					if item.is_dir() {
						let path = item.file_path().to_path_buf();
						self.loader.make_request(LoadRequest::Dir { path, target: Arc::downgrade(node) });
						node_lock.children = LoadState::Loading;
					} else if item.is_pak() {
						let path = item.file_path().to_path_buf();
						self.loader.make_request(LoadRequest::Pak { path, target: Arc::downgrade(node) });
						node_lock.children = LoadState::Loading;
					} else {
						node_lock.children = LoadState::Failed(Arc::new(
							anyhow::anyhow!("not directory or pak; shouldn't be expandable")
						));
					}
				},
				_ => {},
			}
		}
		drop(node_lock);
	}
}

struct TreeRoot {
	node: LoadState<Arc<Mutex<TreeNode>>>,
}

pub struct TreeNode {
	parent: Weak<Mutex<TreeNode>>,
	children: LoadState<Arc<TreeChildren>>,
	
	display_name: String,
	source: TreeNodeSource,
}

impl TreeNode {
	fn from_dir_item(item: DirItem, parent: Weak<Mutex<TreeNode>>) -> Self {
		let path = item.file_path();
		TreeNode {
			parent,
			children: LoadState::Unloaded,
			
			display_name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
			source: TreeNodeSource::Fs(item),
		}
	}
	
	fn pak_wahtever(pak_path: PathBuf, entry_name: BString, parent: Weak<Mutex<TreeNode>>) -> Self {
		use bstr::ByteSlice;
		
		TreeNode {
			parent,
			children: LoadState::Unloaded,
			
			display_name: entry_name.to_str_lossy().to_string(),
			source: TreeNodeSource::Pak { pak_path, entry_name },
		}
	}
	
	pub fn parent(&self) -> Option<Arc<Mutex<TreeNode>>> {
		self.parent.upgrade()
	}
	
	pub fn display_name(&self) -> String {
		self.display_name.to_string()
	}
	
	pub fn is_dir(&self) -> bool {
		match &self.source {
			TreeNodeSource::Fs(item) => item.is_dir(),
			_ => false,
		}
	}
	
	pub fn is_pak(&self) -> bool {
		match &self.source {
			TreeNodeSource::Fs(item) => item.is_pak(),
			_ => false,
		}
	}
	
	pub fn is_expandable(&self) -> bool {
		self.is_dir() || self.is_pak()
	}
	
	pub fn children(&self) -> LoadState<Arc<TreeChildren>> {
		LoadState::clone(&self.children)
	}
	
	pub fn unique_id(&self) -> TreeItemId {
		match &self.source {
			TreeNodeSource::Fs(item) => TreeItemId::Fs(item.file_path().to_path_buf()),
			TreeNodeSource::Pak { pak_path, entry_name } => TreeItemId::Pak(pak_path.clone(), entry_name.clone()),
		}
	}
}

pub struct TreeChildren {
	nodes: Vec<Arc<Mutex<TreeNode>>>,
}

impl TreeChildren {
	pub fn iter(&self) -> impl Iterator<Item = &Arc<Mutex<TreeNode>>> {
		self.nodes.iter()
	}
}

pub enum TreeNodeSource {
	Fs(DirItem),
	Pak { pak_path: PathBuf, entry_name: BString },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TreeItemId {
	Fs(PathBuf),
	Pak(PathBuf, BString),
}

enum LoadRequest {
	Root { path: PathBuf, target: Weak<Mutex<TreeRoot>> },
	Dir { path: PathBuf, target: Weak<Mutex<TreeNode>> },
	Pak { path: PathBuf, target: Weak<Mutex<TreeNode>> },
}

impl ThreadRequest for LoadRequest {
	type Output = ();
	
	fn execute(self) {
		match self {
			Self::Root { path, target } => {
				let result = DirItem::read_single(&path);
				let result = result.map(|item| {
					let parent = Weak::new(); // no parent
					let node = TreeNode::from_dir_item(item, parent);
					Arc::new(Mutex::new(node))
				});
				
				if let Some(strong) = target.upgrade() {
					let mut lock = strong.lock().unwrap();
					lock.node.set_from_load_result(result);
					drop(lock);
				}
			},
			Self::Dir { path, target } => {
				let contents_result = DirContents::read(&path);
				let result = contents_result.map(|mut contents| {
					contents.sort_by_name();
					
					let nodes = contents.iter()
						.map(|x| TreeNode::from_dir_item(DirItem::clone(x), Weak::clone(&target)))
						.map(|x| Arc::new(Mutex::new(x)))
						.collect::<Vec<_>>();
					
					let children = TreeChildren { nodes };
					Arc::new(children)
				});
				
				if let Some(strong) = target.upgrade() {
					let mut lock = strong.lock().unwrap();
					lock.children.set_from_load_result(result);
					drop(lock);
				}
			},
			Self::Pak { path, target } => {
				let result = (|| {
					use std::{fs::File, io::BufReader};
					use crate::formats::pak::PakParser;
					
					let file = File::open(&path)?;
					let bufreader = BufReader::new(file);
					
					let mut parser = PakParser::new(bufreader)?;
					let nodes = parser.files()?
						.map(|r| r.map(|(_, name)| TreeNode::pak_wahtever(path.to_path_buf(), name, Weak::clone(&target))))
						.map(|r| r.map(|x| Arc::new(Mutex::new(x))))
						.collect::<Result<Vec<_>, _>>()?;
					
					let children = TreeChildren { nodes };
					Ok(Arc::new(children))
				})();
				
				if let Some(strong) = target.upgrade() {
					let mut lock = strong.lock().unwrap();
					lock.children.set_from_load_result::<anyhow::Error>(result);
					drop(lock);
				}
			},
		}
	}
}
