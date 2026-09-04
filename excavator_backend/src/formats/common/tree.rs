pub trait TreeFormat: Send + Sync + Sized + 'static {
	type RootId: ItemId<Self>;
	type AnyItemRef<'a>;
	
	fn root_id(&self) -> Self::RootId;
	fn get<T: ItemId<Self>>(&self, id: T) -> Option<T::Ref<'_>> {
		id.get_from(self)
	}
}

pub struct TreeItem<T: TreeItemType> {
	pub data: T,
	pub parent: T::ParentId,
	pub children: T::ChildrenIdList,
}

impl<T: TreeItemType> TreeItem<T> {
	pub fn new(data: T, parent: T::ParentId, children: T::ChildrenIdList) -> Self {
		Self { data, parent, children }
	}
}

pub trait TreeItemType {
	type Format: TreeFormat;
	type ParentId;
	type ChildrenIdList;
}

pub trait ItemId<Format: TreeFormat>: Copy {
	type Ref<'a>;
	fn get_from<'a>(self, source: &'a Format) -> Option<Self::Ref<'a>>;
}
