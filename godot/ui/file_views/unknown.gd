extends Node

@onready var label: Label = $Label

func file_view_open(path: String) -> void:
	label.text = "This file has an unsupported type:\n{0}".format([path.get_file()])
