extends MenuBar

@onready var file_menu: PopupMenu = $File

enum FileItems {OPEN, QUIT}

func _ready() -> void:
	file_menu.add_item("Choose folder...", FileItems.OPEN)
	file_menu.add_item("Quit", FileItems.QUIT)
	file_menu.id_pressed.connect(file_id_pressed)
	
func file_id_pressed(id: int) -> void:
	match id:
		FileItems.OPEN: U.action_open()
		FileItems.QUIT: U.action_quit()
