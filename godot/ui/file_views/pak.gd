extends VBoxContainer

@onready var item_list: ItemList = $ItemList
var resource: SkePak

func _ready() -> void:
	item_list.item_activated.connect(_item_list_item_activated)

func file_view_open(_path: String) -> void:
	item_list.clear()
	for file_name in resource.get_file_names():
		item_list.add_item(file_name)

func _item_list_item_activated(index: int) -> void:
	var file := item_list.get_item_text(index)
	if file.get_extension() == "png":
		var data: PackedByteArray = resource.read_archived_file(file)
		var image := Image.new()
		image.load_png_from_buffer(data)
		var texture := ImageTexture.create_from_image(image)
		
		var window := Window.new()
		window.close_requested.connect(window.queue_free)
		var sprite := Sprite2D.new()
		sprite.texture = texture
		sprite.centered = false
		sprite.scale = Vector2(3, 3)
		window.add_child(sprite)
		add_child(window)
	else:
		push_error("not a png ngl")
