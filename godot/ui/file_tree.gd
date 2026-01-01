extends Tree

signal file_selected(path: String)

func _ready() -> void:
	R.directory_opened.connect(fill_file_tree)
	item_selected.connect(_on_item_selected)

func fill_file_tree(root_path: String, child_paths: PackedStringArray) -> void:
	clear()
	
	var root := create_item()
	root.set_text(0, root_path.get_file())
	root.set_metadata(0, root_path)
	
	for child_path in child_paths:
		var child := create_item(root)
		child.set_text(0, child_path.get_file())
		child.set_metadata(0, child_path)

func _on_item_selected() -> void:
	var selected_path: String = get_selected().get_metadata(0)
	file_selected.emit(selected_path)
