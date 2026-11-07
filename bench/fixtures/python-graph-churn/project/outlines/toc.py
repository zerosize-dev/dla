def number_headings(tree):
    """Yield (label, node) pairs such as ('2.1.3', node) in document order."""
    counters = []
    for node in tree.root.walk():
        if node is tree.root:
            continue
        del counters[node.depth:]
        while len(counters) < node.depth:
            counters.append(0)
        counters[node.depth - 1] += 1
        yield ".".join(str(c) for c in counters), node


def render(tree, max_depth=3):
    lines = [tree.name.upper()]
    for label, node in number_headings(tree):
        if node.depth <= max_depth:
            lines.append("  " * (node.depth - 1) + f"{label} {node.title}")
    return "\n".join(lines)


def checksum(text):
    value = 5381
    for ch in text:
        value = ((value << 5) + value + ord(ch)) & 0xFFFFFFFF
    return value
