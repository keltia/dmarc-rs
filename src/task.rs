use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

use crate::entry::Entry;
use anyhow::Result;
use log::{debug, info};

use crate::filetype::Input;

pub enum Task {
    Stream { fh: Box<dyn BufRead>, ft: Input },
    Files { list: Vec<String> },
}

impl Task {
    /// Create a task with input as a set of files possibly with different formats
    ///
    /// Example:
    /// ```
    /// # use std::path::PathBuf;
    /// use dmarc_rs::Task;
    ///
    /// // Possibly from CLI
    /// let fl = vec!["foo.xml", "bar.zip", "baz.gz"];
    ///
    /// let t = Task::from_list(fl);
    /// ```
    ///
    pub fn from_list(l: Vec<&String>) -> Self {
        Task::Files {
            list: l.into_iter().map(|p| p.to_owned()).collect(),
        }
    }

    /// Create a task with input from a buffered stream like stdin
    ///
    /// Example:
    /// ```
    /// # fn main() -> Result<(), std::io::Error> {
    /// # use std::io::BufReader;
    /// use dmarc_rs::{Input, Task};
    ///
    /// let fh = BufReader::new(std::io::stdin())?;
    ///
    /// let t = Task::from_reader(fh, Input::Xml);
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
    pub fn run(&mut self) -> Result<Vec<String>> {
        let res = match self {
            // Read from stream
            //
            Task::Stream { fh, ft } => {
                let mut res = String::new();
                let n = &fh.read_to_string(&mut res)?;
                debug!("Read {} bytes from stdin", n);

                vec![res]
            }
            // Read from list of files and process them
            //
            Task::Files { list } => {
                let res = list
                    .iter()
                    .inspect(|f| info!("Processing {:?}", *f))
                    .map(|p| {
                        let p = PathBuf::from(p);
                        let mut res = String::new();

                        let n = File::open(p).ok()?.read_to_string(&mut res).ok()?;
                        debug!("Read {} bytes from {:?}", n, p);

                        if n == 0 {
                            None
                        } else {
                            Some(res)
                        }
                    })
                    .filter(|f| f.is_some())
                    .map(|f| f.unwrap())
                    .collect::<Vec<String>>();
                res
            }
        };
        Ok(res)
    }

    /// Return the entries to be processed
    ///
    pub fn list(self) -> Vec<PathBuf> {
        match self {
            Task::Stream { .. } => {
                let v = vec![PathBuf::from("<STDIN>").to_owned()];
                v
            }
            Task::Files { list } => list,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;
    use std::io::stdin;

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
