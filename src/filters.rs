use minijinja::{Error, State};
use textwrap::{wrap, Options};

const LINE_WIDTH: usize = 79;

pub(crate) fn comment_license(
    _state: &State,
    value: String,
    comment_char: String,
) -> Result<String, Error> {
    let sep = &format!("{} ", comment_char);
    Ok(wrap(
        &value,
        Options::new(LINE_WIDTH)
            .initial_indent(sep)
            .subsequent_indent(sep),
    )
    .join("\n"))
}

pub(crate) fn hypens_to_underscores(_state: &State, value: String) -> Result<String, Error> {
    Ok(value.replace("-", "_"))
}
