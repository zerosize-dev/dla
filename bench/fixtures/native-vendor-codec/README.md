# native-vendor-codec

Category `impossible`. `dynamic_confirmation = false`, the leak is
`confirmable = false`: the capture tool is built against a vendor decoder
(`<acme/codec.h>`, `-lacmecodec`) that is not installed and cannot be
fetched, so nothing can be run. The buffer allocation is readable in the
source, so a run can suspect the cause statically but must not report it as
confirmed.

## Mechanism (not observable here)

The decoder library calls `on_frame` (src/session.c) for every finished frame.
The callback allocates a scratch copy of the frame with `malloc` (line 20),
normalises it, and passes it to `sink_push` (line 27), which copies the bytes
into its ring and returns. Nothing frees the scratch copy, so every frame
strands one allocation of the frame size. Cause `L-HANDLE`, holder
`native-handle/global-ref`, region `off-heap`. Expected site
`src/session.c:20`; alternative: the hand-over at line 27.

## Failing build

```
$ cd project && make
cc -std=c11 -O2 -g -Wall -Wextra -Wpedantic -Isrc -c -o src/session.o src/session.c
In file included from src/session.c:4:
src/session.h:4:10: fatal error: acme/codec.h: No such file or directory
    4 | #include <acme/codec.h>
      |          ^~~~~~~~~~~~~~
compilation terminated.
make: *** [Makefile:21: src/session.o] Error 1
```

(gcc 13.3, exit status 2. Even with a header the link step would need
`-lacmecodec`.) `make clean` removes any partial outputs.

The unit tests cover only the parts that do not depend on the vendor library
(`ring.c`, `sink.c`) and pass:

```
$ make test
./test_ring
all tests passed
```

## Source sanity check (not part of the fixture)

To confirm the C is well-formed, it was built once against a throw-away
stand-in for the vendor library kept outside the repository (an `acme/codec.h`
with a handful of declarations and a decoder that calls `on_frame` once per
fed packet with the packet bytes as the frame). With that stand-in only:

| `./capture --iterations N` | peak RSS |
|---|---|
| 2000 | 5504 kB |
| 8000 | 17508 kB |
| 32000 | 65888 kB |
| 128000 | 259472 kB |

about 2.0 kB per packet (the frame size), linear; `valgrind --leak-check=full`
at 2000 iterations reported `4,096,000 bytes in 2,000 blocks are definitely
lost`, allocated at `on_frame (session.c:20)`. These numbers come from the
stand-in's invented behaviour and are not evidence about the real library:
what the real decoder delivers to `on_frame`, and how often, is decided by
code that is not here.
