use std::fmt::{Debug, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

use crate::entry::Entry;
use anyhow::{anyhow, Result};
use log::debug;

use crate::input::Input;
use crate::makelist;

pub enum Task {
    Stream { fh: Box<dyn BufRead>, ft: Input },
    Files { list: Vec<Entry> },
}

impl Task {
    /// Create a task with input as a set of files possibly with different formats
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::input::Input;
    /// # use dmarc_rs::task::Task;
    /// # let ft = Input::Xml;
    /// # use std::path::PathBuf;
    ///
    /// // Possibly from CLI
    /// let fl = vec![PathBuf::from("foo.xml"), PathBuf::from("bar.zip"), PathBuf::from("baz.gz")];
    ///
    /// let t = Task::from_list(fl);
    /// ```
    ///
    pub fn from_list(l: Vec<PathBuf>) -> Self {
        Task::Files {
            list: l.iter().map(|&p| Entry::new(p)).collect(),
        }
    }

    /// Create a task with input from a buffered stream like stdin
    ///
    /// Example:
    /// ```
    /// # fn main() -> Result<(), std::io::Error> {
    /// # use std::io::BufReader;
    /// # use dmarc_rs::input::Input;
    /// # use dmarc_rs::task::Task;
    /// # let ft = Input::Xml;
    ///
    /// let fh = BufReader::new(std::io::stdin())?;
    ///
    /// let t = Task::from_reader(fh, ft);
    /// # }
    /// ```
    ///
    pub fn from_reader<T: Read + 'static>(fh: T, ft: Input) -> Self {
        Task::Stream {
            fh: Box::new(BufReader::new(fh)),
            ft,
        }
    }

    /// Execute the task
    ///
    pub fn run(&self) -> Result<Vec<String>> {
        let res = match self {
            // Read from stream
            //
            Task::Stream { mut fh, ft } => {
                let mut res = String::new();
                let n = fh.read_to_string(&mut res)?;
                debug!("Read {} bytes from stdin", n);

                vec![res]
            }
            // Read from list of files and process them
            //
            Task::Files { list } => list.iter().filter(|&f| !f.res.is_empty()).collect(),
        };
        Ok(res)
    }

    /// Return the entries to be processed
    ///
    pub fn list(self) -> Vec<Entry> {
        match self {
            Task::Stream { .. } => vec![Entry::new("<STDIN>")],
            Task::Files { list } => list,
            Task::Null => vec![],
        }
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Stream { ft, fh } => f
                .debug_struct("Task::Stream")
                .field("ft", ft)
                .field("fh", &"<STREAM>")
                .finish(),
            Task::Files { list } => f.debug_struct("Task::Files").field("list", list).finish(),
            Task::Null => f.write_str("Nothing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;
    use std::io::stdin;

    #[test]
    fn test_task_debug() {
        let t = Task::default();

        let s = format!("{:?}", t);
        dbg!(&s);
        assert_eq!("Nothing", s);
    }

    #[test]
    fn test_from_files() {
        let t = Task::from_list(vec![]);

        assert!(t.list().is_empty());
    }

    #[test]
    fn test_from_reader() {
        let t = Task::from_reader(stdin(), Input::Xml);

        assert!(match t {
            Task::Stream { .. } => true,
            _ => false,
        });
        assert_eq!(1, t.list().len())
    }
}
