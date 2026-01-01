extends Container

func file_view_open(path: String) -> void:
	for child in get_children():
		child.queue_free()
	
	var file_resource: Resource = R.open_file(path)
	
	var scene: Node
	if file_resource is SkePak:
		scene = preload("uid://dkjfbf8i8bd6t").instantiate()
		scene.resource = file_resource
	else:
		scene = preload("uid://bc7du68pcbhuw").instantiate()
	
	add_child(scene)
	scene.file_view_open(path)
