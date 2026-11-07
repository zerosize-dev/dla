'use strict';

// Singly linked list of work waiting for its due time. New entries are added
// at the tail; callers walk it from the head and unlink what they run.
class PendingList {
  constructor() {
    this.head = null;
    this.tail = null;
    this.length = 0;
  }

  append(node) {
    node.next = null;
    if (this.tail) this.tail.next = node;
    else this.head = node;
    this.tail = node;
    this.length += 1;
    return node;
  }

  // Remove `node`, whose predecessor is `prev` (null when it is the head).
  unlink(prev, node) {
    if (prev) prev.next = node.next;
    else this.head = node.next;
    if (this.tail === node) this.tail = prev;
    node.next = null;
    this.length -= 1;
  }

  *[Symbol.iterator]() {
    for (let node = this.head; node; node = node.next) yield node;
  }
}

module.exports = { PendingList };
