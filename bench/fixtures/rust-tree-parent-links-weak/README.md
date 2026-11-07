# rust-tree-parent-links-weak

Category `decoy-control` (partner `rust-tree-parent-links`). Cause `N-ALLOC`: the
resident set settles after start-up and every request's data is freed.

## Mechanism

Same code as `rust-tree-parent-links`, except that a section's link to its parent
is a `Weak` reference (`Rc::downgrade`, `src/section.rs:27`), so the only strong
edges point from parent to child. When `Service::handle` returns, the root's
count reaches zero and the whole tree is freed. A run that reports a confirmed
growth here is a false alarm. Both fixtures share the same `task.prompt`.

Non-leak site: `src/section.rs:27`.

## Minimal edit from the trap

`diff -ru` of the two `project/` directories, copied outside the repository so the
paths are neutral (`trap` is `rust-tree-parent-links`, `control` is this fixture).
Output is 42 lines, 10 of them added or removed lines (5 changed lines in one file).

```
diff -ru trap/src/section.rs control/src/section.rs
--- trap/src/section.rs
+++ control/src/section.rs
@@ -1,13 +1,13 @@
 //! A heading in a document outline, linked to its parent and its children.
 
 use std::cell::RefCell;
-use std::rc::Rc;
+use std::rc::{Rc, Weak};
 
 pub struct Section {
     pub title: String,
     pub level: usize,
     pub line: usize,
-    parent: RefCell<Option<Rc<Section>>>,
+    parent: RefCell<Weak<Section>>,
     children: RefCell<Vec<Rc<Section>>>,
 }
 
@@ -17,19 +17,19 @@
             title: title.to_string(),
             level,
             line,
-            parent: RefCell::new(None),
+            parent: RefCell::new(Weak::new()),
             children: RefCell::new(Vec::new()),
         })
     }
 
     /// Adds `child` under `parent` and records the back-link used for breadcrumbs.
     pub fn append(parent: &Rc<Section>, child: Rc<Section>) {
-        *child.parent.borrow_mut() = Some(Rc::clone(parent));
+        *child.parent.borrow_mut() = Rc::downgrade(parent);
         parent.children.borrow_mut().push(child);
     }
 
     pub fn parent(&self) -> Option<Rc<Section>> {
-        self.parent.borrow().clone()
+        self.parent.borrow().upgrade()
     }
 
     pub fn children(&self) -> Vec<Rc<Section>> {
```

## Measurements

Same external counting-allocator harness as the trap (see its README). The
workload is `cargo run --release -- --iterations N` (0.07 s at 2000, 0.21 s at
8000); `cargo test --release` passes with 10 tests.

| documents processed | live bytes | live blocks | RSS (2000-document run) | RSS (8000-document run) |
|---|---|---|---|---|
| 0     | 548 | 2 | 2.1 MB | 2.1 MB |
| 500   | 558 | 3 | 2.30 MB | |
| 1000  | 558 | 3 | 2.32 MB | |
| 1500  | 558 | 3 | 2.34 MB | |
| 2000  | 558 | 3 | 2.36 MB (end) | 2.33 MB |
| 4000  | 558 | 3 | | 2.34 MB |
| 6000  | 558 | 3 | | 2.35 MB |
| 8000  | 548 (end) | 2 | | 2.35 MB (end) |

The live heap returns to its baseline after every document. RSS moves by about
0.02 MB between 2000 and 8000 documents: a plateau that is reached within the
first 500 documents (the allocator keeps its first arena pages).

RSS figures exclude launcher overhead (they are the process's own `VmHWM`, sampled
while it runs). Re-measured: 2.34 MB at 500 documents, 2.40 MB at 2000, 2.42 to
2.49 MB at 8000. Live allocations are 2 to 3 blocks and under 600 bytes after every
document (548 bytes and 2 blocks once the request text is released; 558 bytes and 3
blocks while it is still held).
