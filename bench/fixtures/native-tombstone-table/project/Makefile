CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic
CFLAGS  += -Isrc
BIN     := leasebroker
LIB_SRC := src/lease_table.c src/broker.c src/report.c
LIB_OBJ := $(LIB_SRC:.c=.o)

all: $(BIN)

$(BIN): $(LIB_OBJ) src/main.o
	$(CC) $(CFLAGS) -o $@ $^

test_lease: $(LIB_OBJ) tests/test_lease.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_lease
	./test_lease

%.o: %.c src/types.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_lease src/*.o tests/*.o

.PHONY: all test clean
