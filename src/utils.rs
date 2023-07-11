use std::fmt::Write;
use crate::{notes::SessionEntry, Error};

pub fn build_session_notes_output(entries: &Vec<SessionEntry>) -> Result<String, Error> {
    let mut  out = String::new();

    let mut last_author: Option<u64> = None;
    for entry in entries {
        if last_author.unwrap_or_default() != entry.author {
            writeln!(out, "\n:<@{}>", entry.author)?;
            last_author = Some(entry.author);
        }
        writeln!(out, "{}", entry.content)?;
    }

    let out = out.trim().to_string();

    Ok(out)
}