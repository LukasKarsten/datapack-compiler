use std::{
    ops::Range,
    path::{Path, PathBuf},
};

pub struct SourceFile {
    path: Option<PathBuf>,
    text: String,
    line_endings: Vec<usize>,
}

impl SourceFile {
    pub fn new(path: Option<PathBuf>, text: String) -> Self {
        let line_endings = find_line_endings(&text).collect();
        Self {
            path,
            text,
            line_endings,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn replace_range(&mut self, range: Range<usize>, new_text: &str) {
        let line = self.byte_to_line(range.start).unwrap();
        self.text.replace_range(range.clone(), new_text);
        self.line_endings.drain(line..);
        self.line_endings
            .extend(find_line_endings(&self.text[range.start..]).map(|off| off + range.start));
    }

    pub fn byte_to_line(&self, idx: usize) -> Option<usize> {
        (idx <= self.text.len()).then(|| match self.line_endings.binary_search(&idx) {
            Ok(line) => line,
            Err(line) => line,
        })
    }

    pub fn line_to_byte(&self, line: usize) -> Option<usize> {
        (line == 0)
            .then_some(0)
            .or_else(|| self.line_endings.get(line - 1).copied())
    }
}

fn find_line_endings(string: &str) -> impl Iterator<Item = usize> + use<'_> {
    string
        .char_indices()
        .filter(|(_, chr)| *chr == '\n')
        .map(|(idx, _)| idx)
}
