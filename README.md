# renpy-text-hook
a tool to extract the currently displayed dialogue or narration text to the clipboard.

only works with ren'py games running on linux.

not tested and will probably not work with ren'py >= 8.0 (i.e., python 3+) and games running in 32-bit.

## usage
```
cargo build --release
cd target/release
sudo ./renpy-text-hook -p <RENPY_GAME_PID>
```

## how?
the tool consists of two parts, an injector and a shared library. when running the program, the injector injects the shared library into the game process which in turn hooks a function most text lines are passed through (`PyUnicodeUCS4_Format`). the text is then sent through a unix socket to the injector which places it in the system clipboard.
