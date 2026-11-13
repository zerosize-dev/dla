CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic -D_POSIX_C_SOURCE=200809L
CFLAGS  += -Isrc
BIN     := parcelroute
LIB_SRC := src/intern.c src/parcel.c src/report.c src/router.c src/spool.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_router: $(LIB_OBJ) tests/test_router.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_router
	./test_router

%.o: %.c src/types.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_router src/*.o tests/*.o

.PHONY: all test clean
