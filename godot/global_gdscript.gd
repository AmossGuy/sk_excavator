extends Node

func _ready() -> void:
	load_settings()
	
	if settings.has_section_key("game", "path"):
		R.open_directory(settings.get_value("game", "path"))

const settings_path: String = "user://settings.cfg"
var settings: ConfigFile

func load_settings() -> void:
	if settings != null:
		show_error("Tried to load settings when they were already loaded!")
		return
	
	var new_settings := ConfigFile.new()
	var err := new_settings.load(settings_path)
	if err != OK and err != ERR_FILE_NOT_FOUND:
		show_error("Error while loading settings: {0}".format([error_string(err)]))
		return
	
	settings = new_settings

func save_settings() -> void:
	if settings == null:
		show_error("Tried to save settings before they were loaded!")
		return
	
	var err := settings.save(settings_path)
	if err != OK:
		show_error("Error while saving settings: {0}".format([error_string(err)]))
		return

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

func show_error(message: String) -> void:
	var error_dialog := AcceptDialog.new()
	error_dialog.title = "Error"
	error_dialog.dialog_text = message
	show_dialog(error_dialog)
