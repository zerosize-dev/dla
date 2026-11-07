from dataclasses import dataclass


class Node:
    """One heading of an outline, linked to its parent and children."""

    def __init__(self, title, depth, tree, parent=None):
        self.title = title
        self.depth = depth
        self.tree = tree
        self.parent = parent
        self.children = []
        if parent is not None:
            parent.children.append(self)

    def path(self):
        names = []
        node = self
        while node is not None and node.parent is not None:
            names.append(node.title)
            node = node.parent
        return "/".join(reversed(names))

    def walk(self):
        yield self
        for child in self.children:
            yield from child.walk()


class Tree:
    """The parsed outline: a synthetic root plus a lookup of headings by path."""

    def __init__(self, name):
        self.name = name
        self.root = Node(name, 0, self)
        self.by_path = {}

    def register(self, node):
        self.by_path[node.path()] = node

    def size(self):
        return len(self.by_path)


@dataclass(frozen=True)
class OutlineRequest:
    name: str
    text: str


@dataclass(frozen=True)
class OutlineResult:
    name: str
    headings: int
    toc: str
    checksum: int
