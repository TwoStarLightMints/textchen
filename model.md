# DOCUMENT

The document structure is used like a buffer. It stores the file name, lines of a file, and visible rows.

# CURSOR

The cursor structure contains the information for the terminal cursor such as cursor position on the screen,
the cursor position within the document, and when the developer wishes, the previous position.

# GAPBUF

The gapbuf structure is used to hold the contents of the selected line to be edited in a document.

# EDITOR

The editor structure is used to hold the dimensions (width and height) of the editor including the editor screen's
home row, the bottom, left edge, right edge, mode row, and command row of the editor window.

# WH

The wh structure is used to catch the values of the editor's width and height of the terminal from the C library.
