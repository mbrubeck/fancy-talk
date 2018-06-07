target/c-fancy-server: src/main.c
	gcc -o target/c-fancy-server -L../proto/target/debug -lfancy_talk src/main.c

run: target/c-fancy-server
	LD_LIBRARY_PATH=../proto/target/debug ./target/c-fancy-server

dbg: target/c-fancy-server
	LD_LIBRARY_PATH=../proto/target/debug gdb ./target/c-fancy-server
