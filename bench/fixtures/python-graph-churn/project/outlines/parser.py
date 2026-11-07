from .models import Node, Tree

INDENT = 2


class OutlineError(ValueError):
    pass


def parse(name, text):
    tree = Tree(name)
    stack = [tree.root]
    for number, raw in enumerate(text.splitlines(), 1):
        if not raw.strip():
            continue
        spaces = len(raw) - len(raw.lstrip(" "))
        if spaces % INDENT:
            raise OutlineError(f"line {number}: odd indentation")
        depth = spaces // INDENT + 1
        if depth > len(stack):
            raise OutlineError(f"line {number}: heading skips a level")
        del stack[depth:]
        node = Node(raw.strip().lstrip("-").strip(), depth, tree, stack[-1])
        stack.append(node)
        tree.register(node)
    return tree
