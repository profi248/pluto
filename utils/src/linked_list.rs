use std::{ ptr, sync::atomic::Ordering, marker::PhantomData };

#[cfg(not(loom))]
use std::sync::atomic::AtomicPtr;
#[cfg(loom)]
use loom::sync::atomic::AtomicPtr;

/// An atomic, stack-like singly-linked list for concurrent use.
///
/// Inserting items to the linked list is atomic, and therefore
/// multiple threads can insert at the same time. However,
/// the exact order may vary.
///
/// Removing items requires a mutable reference, which ensures
/// that no threads have references to nodes in the linked list.
pub struct LinkedList<T> {
    root: AtomicPtr<Node<T>>,

    // Transfer Send- and Sync-ness.
    phantom: PhantomData<Node<T>>,
}

// todo: support Arc/Weak for node references

/// A node on a [`LinkedList`].
pub struct Node<T> {
    pub value: T,
    next: AtomicPtr<Node<T>>,

    // Remove Send and Sync
    phantom: PhantomData<*mut ()>,
}

unsafe impl<T: Send + Sync> Send for Node<T> { }
unsafe impl<T: Send + Sync> Sync for Node<T> { }

impl<T> LinkedList<T> {
    /// Creates a new empty linked list.
    pub fn new() -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
            phantom: PhantomData,
        }
    }

    fn init(&self, value: Node<T>) -> Result<*mut Node<T>, T> {
        let pointer = Box::leak(Box::new(value)) as *mut _;

        match self.root.compare_exchange(
            ptr::null_mut(),
            pointer,
            Ordering::Release,
            Ordering::Relaxed
        ) {
            Ok(_) => Ok(pointer),
            Err(_) => Err(unsafe { Box::from_raw(pointer).extract() }),
        }
    }

    /// Inserts a value to the front of the linked list (pushes onto stack).
    pub fn push_front(&self, value: T) -> &Node<T> {
        // Relaxed ordering for null checking.
        let mut root = self.root.load(Ordering::Relaxed);

        let mut value = Some(value);

        // Check if null.
        if root == ptr::null_mut() {
            // Initialise value.
            match self.init(Node::new(value.take().unwrap())) {
                // Return node if successful.
                Ok(node) => return unsafe { &*node },
                // Root was changed, continue with default behaviour.
                Err(v) => value = Some(v),
            }
        }

        loop {
            // Grab value with Acquire.
            root = self.root.load(Ordering::Acquire);

            // Initialise new node.
            let node = Node::new(value.take().unwrap());
            node.next.store(root, Ordering::Relaxed);
            let pointer = Box::leak(Box::new(node));

            // Exchange with root.
            let result = self.root.compare_exchange(
                root,
                pointer,
                Ordering::Release,
                Ordering::Relaxed
            );

            match result {
                // Return node if successful.
                Ok(_) => return &*pointer,
                // Root was changed, retry.
                Err(_) => {
                    value = unsafe { Some(Box::from_raw(pointer).extract()) };

                    #[cfg(loom)]
                    loom::thread::yield_now();

                    continue;
                },
            }
        }
    }

    /// Removes the first value in the linked list (pops the stack).
    pub fn pop_front(&mut self) -> Option<T> {
        let mut front;

        loop {
            front = self.root.load(Ordering::Acquire);

            if front == ptr::null_mut() {
                return None;
            }

            let new_front = unsafe { (*front).next.load(Ordering::Relaxed) };

            let result = self.root.compare_exchange(
                front,
                new_front,
                Ordering::Release,
                Ordering::Relaxed
            );

            match result {
                Ok(_) => return unsafe { Some(Box::from_raw(front).extract()) },
                Err(_) => {
                    #[cfg(loom)]
                    loom::thread::yield_now();

                    continue
                },
            }
        }
    }

    /// Gets a reference to the first node.
    pub fn front(&self) -> Option<&Node<T>> {
        unsafe { self.root.load(Ordering::Acquire).as_ref() }
    }

    /// Gets a mutable reference to the first node.
    pub fn front_mut(&mut self) -> Option<&mut Node<T>> {
        unsafe { self.root.load(Ordering::Acquire).as_mut() }
    }

    /// Iterates over the linked list from the current front node.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            node: self.front()
        }
    }

    /// Iterates over the linked list mutably from the current front node.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            node: self.front_mut().map(|p| p as *mut _),
            phantom: PhantomData
        }
    }
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            next: AtomicPtr::new(ptr::null_mut()),
            phantom: PhantomData,
        }
    }

    fn extract(self: Box<Self>) -> T {
        (*self).value
    }

    /// Pushes a new node after this node.
    pub fn push_next(&self, value: T) -> &Node<T> {
        let new = Box::leak(Box::new(Self::new(value)));

        loop {
            let next = self.next.load(Ordering::Acquire);

            new.next.store(next, Ordering::Release);

            let result = self.next.compare_exchange(
                next,
                new,
                Ordering::Release,
                Ordering::Relaxed
            );

            match result {
                Ok(_) => return &*new,
                Err(_) => {
                    #[cfg(loom)]
                    loom::thread::yield_now();

                    continue
                },
            }
        }
    }

    /// Gets a reference to the next node.
    pub fn next(&self) -> Option<&Node<T>> {
        unsafe { self.next.load(Ordering::Acquire).as_ref() }
    }

    /// Gets a mutable reference to the next node.
    pub fn next_mut(&mut self) -> Option<&mut Node<T>> {
        unsafe { self.next.load(Ordering::Acquire).as_mut() }
    }

    /// Removes the value after this node from the linked list.
    pub fn pop_next(&mut self) -> Option<T> {
        let mut next;

        loop {
            next = self.next.load(Ordering::Acquire);

            if next == ptr::null_mut() {
                return None;
            }

            let new_next = unsafe { (*next).next.load(Ordering::Relaxed) };

            let result = self.next.compare_exchange(
                next,
                new_next,
                Ordering::Release,
                Ordering::Relaxed
            );

            match result {
                Ok(_) => return unsafe { Some(Box::from_raw(next).extract()) },
                Err(_) => {
                    #[cfg(loom)]
                    loom::thread::yield_now();

                    continue
                },
            }
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let root = self.root.load(Ordering::Acquire);

        if root == ptr::null_mut() {
            return;
        }

        let node = unsafe { Box::from_raw(root) };

        let mut next = node.next.load(Ordering::Acquire);

        while next != ptr::null_mut() {
            let n = unsafe { Box::from_raw(next) };
            next = n.next.load(Ordering::Acquire);
        }
    }
}

pub struct Iter<'a, T> {
    node: Option<&'a Node<T>>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = &self.node?.value;
        self.node = self.node?.next();

        Some(value)
    }
}

pub struct IterMut<'a, T> {
    node: Option<*mut Node<T>>,

    phantom: PhantomData<&'a mut Node<T>>
}

// todo: double check this lol
unsafe impl<'a, T: Send> Send for IterMut<'a, T> { }

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let value = &mut (*self.node?).value;
            self.node = (*self.node?).next_mut().map(|p| p as *mut _);

            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(loom))]
    #[test]
    fn test_linked_list() {
        let mut list: LinkedList<i32> = LinkedList::new();

        {
            let node_1 = list.push_front(1);
            assert_eq!(node_1.value, 1);
            let node_42 = list.push_front(42);
            assert_eq!(node_42.value, 42);
            let node_1337 = list.push_front(1337);
            assert_eq!(node_1337.value, 1337);

            let node_2 = node_1.push_next(2);
            assert_eq!(node_2.value, 2);
            let node_3 = node_2.push_next(3);
            assert_eq!(node_3.value, 3);
            let node_4 = node_1337.push_next(4);
            assert_eq!(node_4.value, 4);
        }

        // 1337 4 42 1 2 3

        let val = list.pop_front();
        assert_eq!(val, Some(1337));
        let val = list.pop_front();
        assert_eq!(val, Some(4));
        list.push_front(-1);
        let val = list.pop_front();
        assert_eq!(val, Some(-1));
        list.push_front(-2);
        list.push_front(-3);
        let val = list.pop_front();
        assert_eq!(val, Some(-3));
        let val = list.pop_front();
        assert_eq!(val, Some(-2));
        let val = list.pop_front();
        assert_eq!(val, Some(42));
        let val = list.pop_front();
        assert_eq!(val, Some(1));
        let val = list.pop_front();
        assert_eq!(val, Some(2));
        let val = list.pop_front();
        assert_eq!(val, Some(3));
        let val = list.pop_front();
        assert_eq!(val, None);
        let val = list.pop_front();
        assert_eq!(val, None);

        list.push_front(0);
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        list.push_front(4);
        list.push_front(5);

        let first = list.front_mut().unwrap();

        assert_eq!(first.pop_next(), Some(4));
        assert_eq!(first.pop_next(), Some(3));
        assert_eq!(list.pop_front(), Some(5));

        let first = list.front_mut().unwrap();

        assert_eq!(first.value, 2);
        assert_eq!(first.pop_next(), Some(1));
        assert_eq!(first.pop_next(), Some(0));
        assert_eq!(first.pop_next(), None);
        assert_eq!(first.pop_next(), None);


        let mut list: LinkedList<String> = LinkedList::new();

        let node = list.push_front("Hello".to_string());
        assert_eq!(node.value, "Hello".to_string());
        let node = list.push_front("World".to_string());
        assert_eq!(node.value, "World".to_string());

        let val = list.pop_front();
        assert_eq!(val, Some("World".to_string()));
        let val = list.pop_front();
        assert_eq!(val, Some("Hello".to_string()));
        let val = list.pop_front();
        assert_eq!(val, None);
        let val = list.pop_front();
        assert_eq!(val, None);
    }

    #[cfg(not(loom))]
    #[test]
    fn test_iter() {
        let list: LinkedList<i32> = LinkedList::new();

        (0..100).for_each(|i| { list.push_front(i); });

        let values: Vec<&i32> = list.iter().collect();

        assert_eq!(values.len(), 100);

        values.into_iter().rev().enumerate().for_each(|(i, &v)| assert_eq!(i as i32, v));
    }

    #[cfg(loom)]
    #[test]
    fn multithreaded_test() {
        use loom::thread;
        use loom::sync::Arc;

        loom::model(|| {
            let shared_list = Arc::new(LinkedList::<i32>::new());

            let clone1 = shared_list.clone();

            let thread_1 = thread::spawn(move || {
                let shared_list = clone1;

                let node_1 = shared_list.push_front(1);
                assert_eq!(node_1.value, 1);
                let node_2 = shared_list.push_front(2);
                assert_eq!(node_2.value, 2);
                let node_3 = shared_list.push_front(3);
                assert_eq!(node_3.value, 3);
                let node_4 = shared_list.push_front(4);
                assert_eq!(node_4.value, 4);
                let node_5 = shared_list.push_front(5);
                assert_eq!(node_5.value, 5);

                node_1.push_next(6);
                node_2.push_next(7);
                node_3.push_next(8);
                node_4.push_next(9);
                node_5.push_next(10);
            });

            {
                let node_1 = shared_list.push_front(-1);
                assert_eq!(node_1.value, -1);
                let node_2 = shared_list.push_front(-2);
                assert_eq!(node_2.value, -2);
                let node_3 = shared_list.push_front(-3);
                assert_eq!(node_3.value, -3);
                let node_4 = shared_list.push_front(-4);
                assert_eq!(node_4.value, -4);
                let node_5 = shared_list.push_front(-5);
                assert_eq!(node_5.value, -5);

                node_1.push_next(-6);
                node_2.push_next(-7);
                node_3.push_next(-8);
                node_4.push_next(-9);
                node_5.push_next(-10);
            }

            thread_1.join().unwrap();

            for n in shared_list.iter() {
                println!("{}", n);
            }
        });
    }


}
