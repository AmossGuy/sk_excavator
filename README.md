# Shovel Knight Excavator

Shovel Knight modding tool, currently in a very primitive state of development. It doesn't yet have the functionality needed to actually make a mod, but here's a list of what is implemented so far:

* It can list the files contained within Shovel Knight's .pak archives. Right clicking on a file shows the option to extract it from the archive.
* It can display the contents of certain filetypes contained in said archives:
  * .png - You're surely aware of this image format. Shovel Knight only uses these for palettes and a few assets used for OS integration.
  * .stb / .stl / .stm - A group of similar formats for tabular data. The most important use of these is the storage of all the dialogue and other text.
  * I've started work on reading .anb files, the main format for the game's graphics, but my implementation isn't complete enough for it to parse them properly just yet. This is next on my list of things to get working.
