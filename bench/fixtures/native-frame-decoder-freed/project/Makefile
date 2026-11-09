CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic
CFLAGS  += -Isrc
BIN     := framedec
LIB_SRC := src/checksum.c src/rle.c src/decoder.c src/synth.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_decoder: $(LIB_OBJ) tests/test_decoder.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_decoder
	./test_decoder

%.o: %.c src/frame.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_decoder src/*.o tests/*.o

.PHONY: all test clean
