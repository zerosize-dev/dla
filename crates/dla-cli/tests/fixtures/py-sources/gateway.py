"""A session gateway that never lets go of what it opens.

    python3 gateway.py <sessions>
"""

import asyncio
import sys
import threading
import time

_SESSIONS = []
_LOOP = asyncio.new_event_loop()


def _heartbeat():
    time.sleep(3600)


async def _watch():
    await asyncio.sleep(3600)


def open_session(loop):
    beat = threading.Thread(target=_heartbeat, name=f"heartbeat-{len(_SESSIONS)}", daemon=True)
    beat.start()
    _SESSIONS.append(beat)
    loop.create_task(_watch())
    audit = open(__file__, encoding="utf-8")
    audit.readline()


async def serve(loop, sessions):
    for _ in range(sessions):
        open_session(loop)
    await asyncio.sleep(0)


def main():
    sessions = int(sys.argv[1]) if len(sys.argv) > 1 else 40
    _LOOP.run_until_complete(serve(_LOOP, sessions))
    print(f"{len(_SESSIONS)} sessions open")


if __name__ == "__main__":
    main()
