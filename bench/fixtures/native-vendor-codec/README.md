# native-vendor-codec

Category `impossible`. `dynamic_confirmation = false`, the leak is
`confirmable = false`: the capture tool is built against a vendor decoder
(`<acme/codec.h>`, `-lacmecodec`) that is not installed and cannot be fetched,
so nothing can be run. A run may suspect the cause statically but must not
report it as confirmed. Expected answer if suspected: site `src/sink.c:24`
(alternatives: the hand-over at `src/session.c:14` and the slot store at
`src/ring.c:34`), cause `L-OWNERSHIP`, holder `native-handle/global-ref`,
region `off-heap`.

## Mechanism (not observable here)

The decoder library calls `on_frame` (src/session.c) with a pointer to an
`acme_frame` (payload `data`, length `size`) for every finished frame. The
callback passes the pointer to `sink_push`, which updates the running totals
and keeps the pointer itself in a 32-slot replay ring (`ring_put`, sink.c:24).
Once the ring is full each new frame overwrites the oldest pointer, and nothing
ever passes that frame back to the library. If the library hands out frames the
application must give back through its release function, each overwritten
pointer strands one frame (about 2 KB here) and resident memory grows with the
packet count. If instead the library owns the frames and recycles them on its
own, there is no growth at all, and keeping the pointer past the callback is a
different (dangling pointer) question.

Which of the two holds is the library's ownership contract. Nothing in
`project/` states it: the header and the manual are the only place it is
written, and neither is available. That is why this is `impossible` and why
`L-OWNERSHIP` is only ever suspected.

## Failing build

```
$ cd project && make
cc -std=c11 -O2 -g -Wall -Wextra -Wpedantic -Isrc -c -o src/ring.o src/ring.c
cc -std=c11 -O2 -g -Wall -Wextra -Wpedantic -Isrc -c -o src/sink.o src/sink.c
In file included from src/sink.c:1:
src/sink.h:7:10: fatal error: acme/codec.h: No such file or directory
    7 | #include <acme/codec.h>
      |          ^~~~~~~~~~~~~~
compilation terminated.
make: *** [Makefile:21: src/sink.o] Error 1
```

(exit status 2. Even with a header the link step would need `-lacmecodec`.)
`make clean` removes any partial outputs.

The unit tests cover only the ring, which does not depend on the vendor
library, and pass:

```
$ make test
./test_ring
all tests passed
```

## Why a stand-in cannot settle it

A throw-away `acme/codec.h` and decoder could be written in a scratch directory,
and the program would then build and run. But the stand-in has to decide who
owns a frame after `on_frame` returns: either it hands out fresh allocations
(and so invents a release function that the application is then seen not to
call, and memory grows) or it hands out a buffer it reuses (and memory stays
flat). Both stand-ins are equally easy to write, they give opposite results
on the same application code, and neither is evidence about the real library.
Any measurement taken against a stand-in measures the stand-in's invented
ownership rule, so a run that reports a confirmed leak from one has overclaimed.
