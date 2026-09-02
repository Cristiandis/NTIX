use anyhow::{Context, Result};

fn parse_header(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut in_word = false;
    for (idx, c) in text.chars().enumerate() {
        if c == ' ' {
            in_word = false;
        } else if !in_word {
            starts.push(idx);
            in_word = true;
        }
    }
    starts
}

fn parse_entry(column_starts: &[usize], line: &str) -> Vec<String> {
    let byte_offsets: Vec<usize> = column_starts
        .iter()
        .map(|&pos| char_index_to_byte(line, pos))
        .collect();

    byte_offsets
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = byte_offsets.get(i + 1).copied().unwrap_or(line.len());
            let end = end.min(line.len());
            if start < line.len() {
                line[start..end].trim().to_string()
            } else {
                String::new()
            }
        })
        .collect()
}

fn char_index_to_byte(line: &str, char_idx: usize) -> usize {
    line.char_indices()
        .nth(char_idx)
        .map(|(byte, _)| byte)
        .unwrap_or(line.len())
}

pub fn parse_table<'a>(lines: impl Iterator<Item = &'a str>) -> Result<(usize, Vec<String>)> {
    let mut lines = lines.filter(|x| !x.is_empty());
    let mut line1 = lines.next().context("Invalid output")?;
    let mut line2: &str;
    loop {
        line2 = lines.next().context("Invalid output")?;
        if line2.starts_with("--") {
            break;
        }
        line1 = line2;
    }

    let column_starts = parse_header(line1);

    let cells: Vec<String> = column_starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = column_starts.get(i + 1).copied().unwrap_or(line1.len());
            if start < line1.len() {
                line1[start..end.min(line1.len())].trim().to_string()
            } else {
                String::new()
            }
        })
        .chain(lines.flat_map(|line| parse_entry(&column_starts, line)))
        .collect();

    Ok((column_starts.len(), cells))
}
