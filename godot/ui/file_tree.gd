extends Tree

signal file_selected(path: String)

func _ready() -> void:
	R.directory_opened.connect(fill_file_tree)
	item_selected.connect(_on_item_selected)

func fill_file_tree(root_path: String, child_paths: PackedStringArray) -> void:
	clear()
	
	var root := create_item()
	root.set_text(0, root_path)
	root.set_metadata(0, root_path)
	
	for child_path in child_paths:
		var child := create_item(root)
		child.set_text(0, child_path)
		child.set_metadata(0, child_path)

func get_full_path(item: TreeItem) -> String:
	var root := get_root()
	var path := item.get_metadata(0) as String
	while item != root:
		item = item.get_parent()
		path = (item.get_metadata(0) as String).path_join(path)
	return path

func _on_item_selected() -> void:
	var selected_path: String = get_full_path(get_selected())
	file_selected.emit(selected_path)
