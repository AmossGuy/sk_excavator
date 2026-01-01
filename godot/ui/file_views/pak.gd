extends VBoxContainer

var resource: SkePak

func file_view_open(_path: String) -> void:
	$TextEdit.text = "\n".join(resource.file_names)
