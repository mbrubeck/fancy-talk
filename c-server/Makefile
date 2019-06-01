target/c-fancy-server: src/main.c src/fancy_talk.h target/bindings.h
	gcc -o target/c-fancy-server -L../proto/target/debug -lfancy_talk -ltalloc -Itarget src/main.c

target/bindings.h:
	cbindgen ../proto -o target/bindings.h

run: target/c-fancy-server
	LD_LIBRARY_PATH=../proto/target/debug ./target/c-fancy-server

dbg: target/c-fancy-server
	LD_LIBRARY_PATH=../proto/target/debug gdb ./target/c-fancy-server
