//! A heading in a document outline, linked to its parent and its children.

use std::cell::RefCell;
use std::rc::Rc;

pub struct Section {
    pub title: String,
    pub level: usize,
    pub line: usize,
    parent: RefCell<Option<Rc<Section>>>,
    children: RefCell<Vec<Rc<Section>>>,
}

impl Section {
    pub fn new(title: &str, level: usize, line: usize) -> Rc<Section> {
        Rc::new(Section {
            title: title.to_string(),
            level,
            line,
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
        })
    }

    /// Adds `child` under `parent` and records the back-link used for breadcrumbs.
    pub fn append(parent: &Rc<Section>, child: Rc<Section>) {
        *child.parent.borrow_mut() = Some(Rc::clone(parent));
        parent.children.borrow_mut().push(child);
    }

    pub fn parent(&self) -> Option<Rc<Section>> {
        self.parent.borrow().clone()
    }

    pub fn children(&self) -> Vec<Rc<Section>> {
        self.children.borrow().clone()
    }

    /// Titles from the top-level heading down to this one, excluding the document root.
    pub fn breadcrumb(&self) -> Vec<String> {
        let mut trail = vec![self.title.clone()];
        let mut cursor = self.parent();
        while let Some(section) = cursor {
            if section.level > 0 {
                trail.push(section.title.clone());
            }
            cursor = section.parent();
        }
        trail.reverse();
        trail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_walks_up_to_the_first_heading() {
        let root = Section::new("document", 0, 0);
        let chapter = Section::new("Setup", 1, 1);
        let part = Section::new("Install", 2, 4);
        Section::append(&root, chapter.clone());
        Section::append(&chapter, part.clone());
        assert_eq!(part.breadcrumb(), vec!["Setup", "Install"]);
        assert_eq!(chapter.breadcrumb(), vec!["Setup"]);
    }

    #[test]
    fn children_keep_insertion_order() {
        let root = Section::new("document", 0, 0);
        for (i, name) in ["A", "B", "C"].iter().enumerate() {
            Section::append(&root, Section::new(name, 1, i + 1));
        }
        let names: Vec<String> = root.children().iter().map(|c| c.title.clone()).collect();
        assert_eq!(names, vec!["A", "B", "C"]);
    }
}
