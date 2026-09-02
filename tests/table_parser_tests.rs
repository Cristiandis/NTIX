use ntix_rs::package_manager::table_parser::parse_table;

fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
}

#[test]
fn parse_table_basic_two_columns() {
    let input = "Name        Id\n----        --\nvim         Vim.Vim\npython      Python.Python";
    let (columns, cells) = parse_table(lines(input)).expect("table should parse");
    assert_eq!(columns, 2);
    assert_eq!(cells, vec!["Name", "Id", "vim", "Vim.Vim", "python", "Python.Python"]);
}

#[test]
fn parse_table_skips_blank_lines() {
    let input = "\nName  Id\n--    --\n\nvim   Vim.Vim\n";
    let (columns, cells) = parse_table(lines(input)).expect("table should parse");
    assert_eq!(columns, 2);
    assert_eq!(cells, vec!["Name", "Id", "vim", "Vim.Vim"]);
}

#[test]
fn parse_table_empty_leading_cells_rendered_as_empty() {
    let input = "Name  Id\n--    --\n       x";
    let (_, cells) = parse_table(lines(input)).expect("table should parse");
    // Row shorter than the second column start: first cell empty, no second cell.
    assert!(cells.contains(&String::new()));
}

#[test]
fn parse_table_single_column() {
    let input = "Name\n----\nvim";
    let (columns, cells) = parse_table(lines(input)).expect("table should parse");
    assert_eq!(columns, 1);
    assert_eq!(cells, vec!["Name", "vim"]);
}

#[test]
fn parse_table_multibyte_characters_use_byte_offsets() {
    let input = "名字    版本\n----    --\n维姆    1.0.0";
    let (columns, cells) = parse_table(lines(input)).expect("table should parse");
    assert_eq!(columns, 2);
    assert_eq!(cells, vec!["名字", "版本", "维姆", "1.0.0"]);
}

#[test]
fn parse_table_skips_non_separator_lines_until_dashes() {
    let input = "Some intro line\nHeader  Col\n------  ---\na       b";
    let (_, cells) = parse_table(lines(input)).expect("table should parse");
    assert_eq!(cells, vec!["Header", "Col", "a", "b"]);
}

#[test]
fn parse_table_error_when_no_first_line() {
    let err = parse_table(lines("")).expect_err("empty input should error");
    assert!(err.to_string().contains("Invalid output"));
}

#[test]
fn parse_table_error_when_no_separator() {
    let err = parse_table(lines("Header\n")).expect_err("no separator should error");
    assert!(err.to_string().contains("Invalid output"));
}
