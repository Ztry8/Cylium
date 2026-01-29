# Cylium Editor
## Cylium Editor is a lightweight text editor designed specifically for Cylium, 

It featuring syntax highlighting, automatic indentation, and basic file management.    
It is built with Python and Tkinter, and works on Windows, Linux, and macOS.

## Features

- **Open and save `.cyl` files** only, ensuring file format consistency.
- **Syntax highlighting** for keywords, numbers, strings, and comments.
- **Automatic indentation** for structured code blocks (`proc`, `if`, `while`).
- **Undo/redo support** with standard keyboard shortcuts.
- **Line numbers** displayed alongside the code.
- **Resizable font** with zoom in/out and reset shortcuts (`Ctrl/Cmd + +/- / 0`).
- **Light and dark themes** for comfortable coding.
- **Run `.cyl` scripts** directly from the editor.
- **Unsaved changes warning** on exit to prevent data loss.

## Installation

1. Install Python 3.12 or newer.
2. Ensure you have Tkinter installed (usually included with Python).
3. Run the editor:
   ```
   git clone https://github.com/yourusername/cylium-editor.git
   python3 main.py
   ```

## Usage

- Open `.cyl` files using the **Open File** button.
- Save changes using the **Save File** button.
- Run the current script using the **Run File** button.
- Switch between **Light** and **Dark** themes with the toolbar buttons.
- Use keyboard shortcuts for font resizing and indentation.

## Keyboard Shortcuts

| Action                     | Shortcut                |
|-----------------------------|------------------------|
| Increase font               | Ctrl/Cmd + `+`         |
| Decrease font               | Ctrl/Cmd + `-`         |
| Reset font                  | Ctrl/Cmd + `0`         |
| Insert indentation (Tab)    | Tab                    |
| New line with indentation   | Enter                  |
| Undo                        | Ctrl/Cmd + Z           |
| Redo                        | Ctrl/Cmd + Y           |
