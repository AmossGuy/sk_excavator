pub trait TreeFormat: Send + Sync + 'static {
	type ItemId;
	fn root_id(&self) -> Self::ItemId;
}

pub struct TreeItem<T: TreeItemType> {
	pub item: T,
	pub parent: T::ParentId,
	pub children: T::ChildrenIdList,
}

impl<T: TreeItemType> TreeItem<T> {
	pub fn new(item: T, parent: T::ParentId, children: T::ChildrenIdList) -> Self {
		Self { item, parent, children }
	}
}

pub trait TreeItemType {
	type ParentId;
	type ChildrenIdList;
}
