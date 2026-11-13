CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic
CFLAGS  += -Isrc
BIN     := msgrelay
LIB_SRC := src/gateway.c src/message.c src/pool.c src/report.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_pool: $(LIB_OBJ) tests/test_pool.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_pool
	./test_pool

%.o: %.c src/types.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_pool src/*.o tests/*.o

.PHONY: all test clean
