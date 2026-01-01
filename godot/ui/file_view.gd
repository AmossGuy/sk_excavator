extends Container

@export var view_scenes: Dictionary[String, PackedScene]

func file_view_open(path: String) -> void:
	for child in get_children():
		child.queue_free()
	
	var extension := path.get_extension()
	var scene: Node = view_scenes.get(extension, preload("uid://bc7du68pcbhuw")).instantiate()
	add_child(scene)
	scene.file_view_open(path)
