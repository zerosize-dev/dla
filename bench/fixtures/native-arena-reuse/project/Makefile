CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic -D_DEFAULT_SOURCE
CFLAGS  += -Isrc
BIN     := settlerun
LIB_SRC := src/batch.c src/carry.c src/record.c src/report.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_batch: $(LIB_OBJ) tests/test_batch.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_batch
	./test_batch

%.o: %.c src/types.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_batch src/*.o tests/*.o

.PHONY: all test clean
