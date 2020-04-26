# JFO - Joplin Folder Organizer

This is a quick-and-dirty script to organize
[Joplin](https://git.juliobiason.me/downfav.git/) Notebooks.

The basic premise is this: You have a folder with a bunch of notes; you go
through you notes, adding tags; tagged notes are moved to sub-notebooks, based
on the note title.

The format expected for the title of the notes is
"{sub-folder}/{new-note-title}". This format is what
[downfav](https://git.juliobiason.me/downfav.git/) uses when downloading
favourites from Mastodon directly to Joplin, so you may have an easier time if
you use it for that.

WARNING! Again, this is a quick-and-dirty solution. I did some serious work on
_ignoring_ errors, specially when I shouldn't; code organization is basically
non-existent. Use at your own risk.

## Copyright

Copyright (C) 2020  Julio Biason

## License

GNU AFFERO GENERAL PUBLIC LICENSE, Version 3.
