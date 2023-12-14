use std::fmt::{self, Debug};
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl<T:fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    if result.len() == 0 {
                        result = format!("{}", node.value);
                    } else {
                        result = format!("{} {}", result, node.value);
                    }
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut ret = Self::new();
        ret.size = self.size;
        ret.head = match &self.head {
            Some(ptr) => {
                Some(Box::new(Node::new(ptr.value.clone(), None)))
            }
            None => return ret
        };

        let mut old_ptr = &self.head.as_ref().unwrap().next;
        let mut new_ptr = &mut ret.head;

        loop {
            match old_ptr {
                Some(node) => {
                    new_ptr.as_mut().unwrap().next = Some(Box::new(Node::new(node.value.clone(), None)));
                    new_ptr = &mut new_ptr.as_mut().unwrap().next;
                    old_ptr = &node.next;
                },
                None => break,
            }
        }

        ret
    }
}



impl<T: std::fmt::Debug + std::fmt::Display> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_f = &mut f.debug_struct("LinkedList");
        debug_f.field("size", &self.size);
        debug_f.field("value", &format!("{}", self));
        debug_f.finish()
    }
}
impl<T: std::cmp::PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }
        if self.size == 0 {
            return true;
        }
        let mut ptr1 = &self.head;
        let mut ptr2 = &other.head;
        
        for _ in 0..self.size {
            let v1 = &ptr1.as_ref().unwrap().value;
            let v2 = &ptr2.as_ref().unwrap().value;
            if *v1 != *v2 {
                return false;
            }
            ptr1 = &ptr1.as_ref().unwrap().next;
            ptr2 = &ptr2.as_ref().unwrap().next;
        }
        return true;
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clone() {
        let mut l1: LinkedList<i32> = LinkedList::new();
        l1.push_front(1);
        l1.push_front(2);
        l1.push_front(4);
        l1.push_front(2);
        let l2 = l1.clone();
        println!("size: {} values: {}\nsize: {} values: {}\n", l1.size, l1, l2.size, l2);
    }
    #[test]
    fn test_euqal() {
        let mut l1: LinkedList<i32> = LinkedList::new();
        l1.push_front(1);
        l1.push_front(2);
        l1.push_front(4);
        l1.push_front(2);
        let l2 = l1.clone();
        assert_eq!(l1, l2);
    }
    #[test]
    fn test_debug() {
        let mut l1: LinkedList<i32> = LinkedList::new();
        l1.push_front(1);
        l1.push_front(2);
        l1.push_front(4);
        l1.push_front(2);
        let mut l2 = l1.clone();
        l2.push_front(3);
        assert_eq!(l1, l2);
    }
    #[test]
    fn test_iter() {
        let mut l1: LinkedList<i32> = LinkedList::new();
        l1.push_front(1);
        l1.push_front(2);
        l1.push_front(4);
        l1.push_front(2);
        let mut vec1 = vec![];
        let mut vec2 = vec![];
        for v in &l1 {
            vec1.push(*v);
        }
        for v in &l1 {
            vec2.push(*v);
        }
        assert_eq!(vec1, vec2);
    }
    #[test]
    fn test_into_iter() {
        let mut l1: LinkedList<i32> = LinkedList::new();
        l1.push_front(1);
        l1.push_front(2);
        l1.push_front(4);
        l1.push_front(2);
        let mut vec1 = vec![];
        let mut vec2 = vec![];
        for v in l1 {
            vec1.push(v);
        }
        // no!!!
        // for v in l1 {
        //     vec2.push(v);
        // }
        assert_ne!(vec1, vec2);
        println!("vec1: {:?}\nvec2: {:?}", vec1, vec2);
    }
}

pub struct LinkedListIter<'a, T> {
    current: &'a Option<Box<Node<T>>>,
}

pub struct LinkedListIntoIter<T> {
    current: Option<Box<Node<T>>>,
}

impl<T> Iterator for LinkedListIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(node) => {
                let data = node.value;
                self.current = node.next;
                Some(data)
            }
            None => None
        }
    }
}

impl<'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(node) => {
                self.current =  &node.next;
                Some(&node.value)
            },
            None => None
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = LinkedListIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {current: &self.head}
    }
}

impl <T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = LinkedListIntoIter<T>;
    fn into_iter(mut self) -> Self::IntoIter {
        println!("change!");
        Self::IntoIter {current: self.head.take()}
    }
}
