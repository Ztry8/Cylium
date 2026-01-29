# Copyright (c) 2026 Ztry8 (AslanD)
# Licensed under the Apache License, Version 2.0 (the "License");
# You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

import tkinter as tk
from tkinter import filedialog, messagebox
import subprocess
import sys
import os
import re
from tkinter import font

KEYWORDS = [
    "call",
    "as",
    "while",
    "endwhile",
    "var",
    "const",
    "if",
    "else",
    "endif",
    "echo",
    "exit",
    "delete",
    "proc",
    "end",
    "and",
    "or",
    "not",
]

DEFAULT_FONT_SIZE = 21
MIN_FONT_SIZE = 8
MAX_FONT_SIZE = 32

THEMES = {
    "light": {
        "bg": "#fafafa",             
        "fg": "#2e2e2e",             
        "keyword": "#0077aa",        
        "comment": "#6a737d",        
        "number": "#d14",            
        "string": "#50a14f",         
        "linenumber_bg": "#f0f0f0",  
        "linenumber_fg": "#999999",  
        "insert": "#2e2e2e"         
    },
    "dark": {
        "bg": "#1e1e2e",             
        "fg": "#dcdcdc",             
        "keyword": "#569cd6",        
        "comment": "#6a9955",        
        "number": "#b5cea8",         
        "string": "#ce9178",        
        "linenumber_bg": "#2e2e3e",  
        "linenumber_fg": "#858585",  
        "insert": "#ffffff"          
    }
}

class TextEditor(tk.Tk):
    def __init__(self):
        super().__init__()

        self.geometry("1280x720")

        self.current_file = None

        self.current_theme = "light"
        self.font_size = DEFAULT_FONT_SIZE

        self.editor_font = font.Font(family="Consolas" if sys.platform.startswith("win") else "Menlo", size=self.font_size)

        self._build_ui()
        self._apply_theme()
        self._configure_tags()
        self.update_linenumbers()

        self.text.edit_modified(False)
        self.update_title()

        self.protocol("WM_DELETE_WINDOW", self.on_close)

    def _build_ui(self):
        toolbar = tk.Frame(self)
        toolbar.pack(fill="x")

        tk.Button(toolbar, text="Open File", command=self.open_file).pack(side="left")
        tk.Button(toolbar, text="Save File", command=self.save_file).pack(side="left")
        tk.Button(toolbar, text="Run File", command=self.run_file).pack(side="left")

        tk.Button(toolbar, text="Light theme", command=lambda: self.set_theme("light")).pack(side="right")
        tk.Button(toolbar, text="Black theme", command=lambda: self.set_theme("dark")).pack(side="right")
        tk.Button(toolbar, text="Increase Font", command=self.increase_font).pack(side="right")
        tk.Button(toolbar, text="Decrease Font", command=self.decrease_font).pack(side="right")

        main = tk.Frame(self)
        main.pack(fill="both", expand=True)

        self.linenumbers = tk.Canvas(main, width=50, highlightthickness=0)
        self.linenumbers.pack(side="left", fill="y")

        text_frame = tk.Frame(main)
        text_frame.pack(side="left", fill="both", expand=True)

        self.text = tk.Text(
            text_frame,
            undo=True,
            wrap="none",
            font=self.editor_font
        )

        self.text.pack(side="left", fill="both", expand=True)

        scrollbar = tk.Scrollbar(text_frame, command=self._on_scroll)
        scrollbar.pack(side="right", fill="y")

        self.text.config(yscrollcommand=scrollbar.set)

        self.text.bind("<KeyRelease>", self.on_text_change)
        self.text.bind("<Button-4>", self.on_text_change) 
        self.text.bind("<Button-5>", self.on_text_change)

        self.text.bind("<Control-plus>", self.increase_font)
        self.text.bind("<Control-minus>", self.decrease_font)
        self.text.bind("<Control-0>", self.reset_font)
        self.text.bind("<Control-equal>", self.increase_font)

        self.text.bind("<Command-equal>", self.increase_font)
        self.text.bind("<Command-plus>", self.increase_font)
        self.text.bind("<Command-minus>", self.decrease_font)
        self.text.bind("<Command-0>", self.reset_font)

        self.text.bind("<Return>", self.handle_enter)
        self.text.bind("<Tab>", self.insert_spaces)

    def handle_enter(self, event=None):
        line_index = self.text.index("insert").split(".")[0]
        line_start = f"{line_index}.0"
        line_text = self.text.get(line_start, f"{line_index}.end")

        indent = re.match(r"[ \t]*", line_text).group(0)

        if line_text.strip().endswith(("proc", "if", "while")):
            indent += " " * 4

        self.text.insert("insert", "\n" + indent)
        return "break"

    def insert_spaces(self, event=None):
        self.text.insert("insert", " " * 4)
        return "break"

    def set_theme(self, theme):
        self.current_theme = theme
        self._apply_theme()
        self.highlight()
        self.update_linenumbers()

    def _apply_theme(self):
        t = THEMES[self.current_theme]
        self.text.config(
            bg=t["bg"],
            fg=t["fg"],
            insertbackground=t["insert"]
        )
        self.linenumbers.config(
            bg=t["linenumber_bg"]
        )

    def _configure_tags(self):
        self.text.tag_configure("keyword")
        self.text.tag_configure("comment")
        self.text.tag_configure("number")
        self.text.tag_configure("string")

    def open_file(self):
        path = filedialog.askopenfilename(
            filetypes=[("Cylium files", "*.cyl")]
        )

        if not path:
            return

        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            self.text.delete("1.0", "end")
            self.text.insert("1.0", f.read())

        self.current_file = path
        self.update_title()
        self.highlight()
        self.update_linenumbers()
        self.text.edit_modified(False)

    def save_file(self):
        if not self.current_file:
            path = filedialog.asksaveasfilename(
                defaultextension=".cyl",
                filetypes=[("Cylium files", "*.cyl")]
            )

            if not path:
                return
            self.current_file = path

        with open(self.current_file, "w", encoding="utf-8") as f:
            f.write(self.text.get("1.0", "end-1c"))

        self.update_title()
        self.text.edit_modified(False)

    def update_title(self):
        name = os.path.basename(self.current_file) if self.current_file else "Untitled.cyl"
        mark = " *" if self.text.edit_modified() else ""
        self.title(f"Cylium Editor — {name}{mark}")

    def on_close(self):
        if self.text.edit_modified():
            answer = messagebox.askyesnocancel(
                "Unsaved changes",
                "You have unsaved changes. Save before exit?"
            )

            if answer is None:
                return 

            if answer:
                self.save_file()

        self.destroy()

    def on_text_change(self, event=None):
        self.highlight()
        self.update_linenumbers()
        self.text.edit_modified(True)

    def highlight(self):
        self.text.tag_remove("keyword", "1.0", "end")
        self.text.tag_remove("comment", "1.0", "end")
        self.text.tag_remove("number", "1.0", "end")
        self.text.tag_remove("string", "1.0", "end")

        t = THEMES[self.current_theme]

        self.text.tag_configure("keyword", foreground=t["keyword"])
        self.text.tag_configure("comment", foreground=t["comment"])
        self.text.tag_configure("number", foreground=t["number"])
        self.text.tag_configure("string", foreground=t["string"])

        lines = self.text.get("1.0", "end").splitlines()

        for i, line in enumerate(lines, start=1):
            line_start = f"{i}.0"
            line_end = f"{i}.end"

            if re.match(r"^\s*#", line):
                self.text.tag_add("comment", line_start, line_end)
                continue

            for m in re.finditer(r'"[^"\n]*"', line):
                self.text.tag_add(
                    "string",
                    f"{i}.{m.start()}",
                    f"{i}.{m.end()}"
                )

            for m in re.finditer(r"\b\d+(\.\d+)?\b", line):
                self.text.tag_add(
                    "number",
                    f"{i}.{m.start()}",
                    f"{i}.{m.end()}"
                )

            for kw in KEYWORDS:
                for m in re.finditer(rf"\b{re.escape(kw)}\b", line):
                    self.text.tag_add(
                        "keyword",
                        f"{i}.{m.start()}",
                        f"{i}.{m.end()}"
                    )

    def update_linenumbers(self):
        self.linenumbers.delete("all")

        t = THEMES[self.current_theme]

        i = self.text.index("@0,0")
        while True:
            dline = self.text.dlineinfo(i)
            if dline is None:
                break
            y = dline[1]
            line_number = str(i).split(".")[0]
            self.linenumbers.create_text(
                45,
                y,
                anchor="ne",
                text=line_number,
                fill=t["linenumber_fg"],
                font=self.editor_font
            )
            i = self.text.index(f"{i}+1line")

    def _on_scroll(self, *args):
        self.text.yview(*args)
        self.update_linenumbers()

    def increase_font(self, event=None):
        if self.font_size < MAX_FONT_SIZE:
            self.font_size += 1
            self.editor_font.configure(size=self.font_size)
            self.update_linenumbers()
        return "break"

    def decrease_font(self, event=None):
        if self.font_size > MIN_FONT_SIZE:
            self.font_size -= 1
            self.editor_font.configure(size=self.font_size)
            self.update_linenumbers()
        return "break"

    def reset_font(self, event=None):
        self.font_size = DEFAULT_FONT_SIZE
        self.editor_font.configure(size=self.font_size)
        self.update_linenumbers()
        return "break"

    def run_file(self):
        if not self.current_file:
            messagebox.showwarning("Error", "Cannot save file!")
            return

        self.save_file()

        if sys.platform.startswith("win"):
            cmd = ["cmd", "/c"] + [".\\cylium.exe", self.current_file] + ["&&", "pause"]
            subprocess.Popen(cmd, creationflags=subprocess.CREATE_NEW_CONSOLE)
        else:
            cmd = ["./cylium", self.current_file] + ["; read -p 'Press Enter to close'"]
            subprocess.Popen(
                ["bash", "-c", " ".join(cmd)],
                start_new_session=True
            )

if __name__ == "__main__":
    app = TextEditor()
    app.mainloop()
