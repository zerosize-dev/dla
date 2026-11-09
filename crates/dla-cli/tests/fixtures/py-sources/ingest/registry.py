"""Lookup of callbacks that want to hear about pipeline events."""

_LISTENERS = {}


def subscribe(topic, listener):
    _LISTENERS.setdefault(topic, []).append(listener)


def fire(topic, event):
    delivered = 0
    for listener in _LISTENERS.get(topic, ()):
        listener(event)
        delivered += 1
    return delivered


def listener_count(topic):
    return len(_LISTENERS.get(topic, ()))


def reset():
    """Forget every listener; used by the tests between cases."""
    _LISTENERS.clear()
