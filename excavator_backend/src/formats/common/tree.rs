pub trait TreeFormat: Send + Sync + 'static {
	type ItemId;
	type ItemRef<'a>;
	
	fn root_id(&self) -> Self::ItemId;
	fn get_ref(&self, id: Self::ItemId) -> Option<Self::ItemRef<'_>>;
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
