extends Node

func _ready() -> void:
	load_settings()
	# deferred because otherwise a signal fires before it's connected
	call_deferred(&"_ready_deferred")

func _ready_deferred() -> void:
	if settings.has_section_key("game", "path"):
		R.open_directory(settings.get_value("game", "path"))

const settings_path: String = "user://settings.cfg"
@onready var settings: ConfigFile = load_settings()

func load_settings() -> ConfigFile:
	var new_settings := ConfigFile.new()
	var err := new_settings.load(settings_path)
	if err == OK or err == ERR_FILE_NOT_FOUND:
		return new_settings
	else:
		return null

func save_settings() -> Error:
	return settings.save(settings_path)

func action_open() -> void:
	var open_dialog := FileDialog.new()
	open_dialog.set_file_mode(FileDialog.FILE_MODE_OPEN_DIR)
	open_dialog.dir_selected.connect(_action_open_dir_selected)
	
	open_dialog.use_native_dialog = true
	show_dialog(open_dialog)

func _action_open_dir_selected(dir: String) -> void:
	settings.set_value("game", "path", dir)
	save_settings()
	R.open_directory(dir)

func action_quit() -> void:
	get_tree().quit()

func show_dialog(dialog: Window) -> void:
	add_child(dialog)
	dialog.move_to_center()
	dialog.show()
