CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic
CFLAGS  += -Isrc
BIN     := textrun
LIB_SRC := src/raster.c src/cache.c src/layout.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_cache: $(LIB_OBJ) tests/test_cache.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_cache
	./test_cache

%.o: %.c src/glyph.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_cache src/*.o tests/*.o

.PHONY: all test clean
