import asyncio, functools
class A:
    @functools.lru_cache(maxsize=None)
    def f(self, x): return x
    @functools.cache
    def g(self): return 1
async def m():
    asyncio.create_task(m())
def r():
    f = open("x"); return f.read()
