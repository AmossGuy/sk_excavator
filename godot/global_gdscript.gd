extends Node

func action_open() -> void:
	var open_dialog := FileDialog.new()
	open_dialog.set_file_mode(FileDialog.FILE_MODE_OPEN_DIR)
	open_dialog.dir_selected.connect(R.open_directory)
	
	open_dialog.use_native_dialog = true
	show_dialog(open_dialog)

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
