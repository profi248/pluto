use super::linked_list::LinkedList;

pub struct Tree<T> {
    pub value: T,
    pub children: LinkedList<Tree<T>>,
}

impl<T> Tree<T> {
    /// Creates a new tree with a given node value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            children: LinkedList::new(),
        }
    }

    /// Adds a new node as a child to the current node, with a given node value.
    pub fn add_child(&self, value: T) -> &Tree<T> {
        &self.children.push_front(Tree::new(value)).value
    }

    /// Returns true if the current node is a leaf.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.front().is_none()
    }

    /// Returns true if the current node is a branch.
    #[inline]
    pub fn is_branch(&self) -> bool {
        !self.is_leaf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(loom))]
    #[test]
    fn test_tree() {
        let tree = Tree::new("/");

        let usr = tree.add_child("usr");
        let home = tree.add_child("home");
        let _dev = tree.add_child("dev");

        let _lib = usr.add_child("lib");
        let _share = usr.add_child("share");
        let _local = usr.add_child("local");

        let _pluto = home.add_child("pluto");

        fn tree_walk(tree: &Tree<&str>, path: String) {
            for child in tree.children.iter() {
                let s = format!("{path}/{}", child.value);
                println!("{s}");
                if child.is_leaf() { continue; }
                tree_walk(child, s);
            }
        }
        tree_walk(&tree, "".to_owned());
    }


}
