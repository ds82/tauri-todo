use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(skip)]
    inner: todo_txt::task::Simple,
    pub id: usize,
}

impl TodoItem {
    pub fn new(subject: &str) -> Self {
        let inner = todo_txt::task::Simple::from(subject.to_string());
        Self { inner, id: 0 }
    }

    pub fn subject(&self) -> &str {
        &self.inner.subject
    }

    pub fn set_subject(&mut self, subject: &str) {
        self.inner.subject = subject.to_string();
    }

    pub fn finished(&self) -> bool {
        self.inner.finished
    }

    pub fn complete(&mut self) {
        self.inner.complete();
    }

    pub fn uncomplete(&mut self) {
        self.inner.uncomplete();
    }

    pub fn priority(&self) -> u8 {
        self.inner.priority.clone().into()
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.inner.priority = priority.into();
    }

    pub fn contexts(&self) -> &[String] {
        &self.inner.contexts
    }

    pub fn projects(&self) -> &[String] {
        &self.inner.projects
    }

    pub fn raw(&self) -> String {
        self.inner.to_string()
    }
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct TodoList {
    items: Vec<TodoItem>,
    path: Option<PathBuf>,
    next_id: usize,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            path: None,
            next_id: 1,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let mut list = Self::new();
        list.path = Some(path.to_path_buf());

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let inner = todo_txt::task::Simple::from(line.to_string());
            let id = list.next_id;
            list.next_id += 1;
            list.items.push(TodoItem { inner, id });
        }

        Ok(list)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no file path set"))?;
        self.save_to(path.clone())
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let content: String = self
            .items
            .iter()
            .map(|item| item.inner.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, content)
    }

    pub fn set_path(&mut self, path: impl AsRef<Path>) {
        self.path = Some(path.as_ref().to_path_buf());
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn add(&mut self, subject: &str) -> usize {
        let inner = todo_txt::task::Simple::from(subject.to_string());
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(TodoItem { inner, id });
        id
    }

    pub fn remove(&mut self, id: usize) -> Option<TodoItem> {
        if let Some(pos) = self.items.iter().position(|item| item.id == id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    pub fn get(&self, id: usize) -> Option<&TodoItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut TodoItem> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    pub fn complete(&mut self, id: usize) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.complete();
            true
        } else {
            false
        }
    }

    pub fn uncomplete(&mut self, id: usize) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.uncomplete();
            true
        } else {
            false
        }
    }

    pub fn items(&self) -> &[TodoItem] {
        &self.items
    }

    pub fn pending(&self) -> impl Iterator<Item = &TodoItem> {
        self.items.iter().filter(|item| !item.finished())
    }

    pub fn done(&self) -> impl Iterator<Item = &TodoItem> {
        self.items.iter().filter(|item| item.finished())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut list = TodoList::new();
        let id = list.add("Buy milk @shopping +errands");
        let item = list.get(id).unwrap();
        assert_eq!(item.subject(), "Buy milk @shopping +errands");
        assert!(!item.finished());
        assert!(item.contexts().contains(&"shopping".to_string()));
        assert!(item.projects().contains(&"errands".to_string()));
    }

    #[test]
    fn test_complete_and_uncomplete() {
        let mut list = TodoList::new();
        let id = list.add("Do something");
        assert!(list.complete(id));
        assert!(list.get(id).unwrap().finished());
        assert!(list.uncomplete(id));
        assert!(!list.get(id).unwrap().finished());
    }

    #[test]
    fn test_remove() {
        let mut list = TodoList::new();
        let id = list.add("Temporary task");
        assert_eq!(list.len(), 1);
        list.remove(id);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_pending_and_done() {
        let mut list = TodoList::new();
        list.add("Task 1");
        let id2 = list.add("Task 2");
        list.add("Task 3");
        list.complete(id2);

        assert_eq!(list.pending().count(), 2);
        assert_eq!(list.done().count(), 1);
    }
}
