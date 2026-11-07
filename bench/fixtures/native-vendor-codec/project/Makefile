CC      ?= cc
CFLAGS  ?= -std=c11 -O2 -g -Wall -Wextra -Wpedantic
CFLAGS  += -Isrc
BIN     := capture
CORE    := src/ring.o

all: $(BIN)

# The capture tool talks to the vendor decoder library.
$(BIN): $(CORE) src/sink.o src/session.o src/main.o
	$(CC) $(CFLAGS) -o $@ $^ -lacmecodec

# The ring does not depend on the vendor library.
test_ring: $(CORE) tests/test_ring.o
	$(CC) $(CFLAGS) -o $@ $^

test: test_ring
	./test_ring

%.o: %.c src/ring.h src/sink.h src/session.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(BIN) test_ring src/*.o tests/*.o

.PHONY: all test clean
