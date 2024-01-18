use dialoguer::console::{Emoji, Style, Term};
use tracing::error;

fn handle_err(r: std::io::Result<()>, original_message: &str) {
    match r {
        Ok(_) => (),
        Err(e) => {
            error!(
                message = "Error writing to console",
                original_message,
                error = format!("{}", e)
            )
        }
    }
}
pub fn success(msg: &str) {
    let str = format!("{} {}", Emoji("👍", ""), Style::new().blue().apply_to(msg));
    handle_err(Term::stdout().write_line(&str), &str)
}

pub fn trying(msg: &str) {
    let str = format!(
        "{} {}...",
        Emoji("🕛", ">>"),
        Style::new().yellow().apply_to(msg)
    );
    handle_err(Term::stderr().write_line(&str), &str)
}

pub fn error(msg: &str) {
    let str = format!("{} {}", Emoji("💣", "!!"), Style::new().red().apply_to(msg));
    handle_err(Term::stderr().write_line(&str), &str)
}

pub fn instruct(msg: &str) {
    let str = format!("{} {}", Emoji("ℹ️", ""), Style::new().cyan().apply_to(msg));
    handle_err(Term::stdout().write_line(&str), &str)
}

pub fn banner(msg: &str) {
    let width = Term::stdout().size().1.into();
    let wd = width - 10;
    let str = format!(
        "{:-^width$}\n{:^wd$}\n{:-^width$}",
        "-",
        Style::new().green().apply_to(format!("✨ {} ✨", msg)),
        "-",
        width = width,
        wd = wd
    );
    handle_err(Term::stdout().write_line(&str), &str)
}

pub(crate) fn command_out(msg: &str) {
    let str = format!("{:<2}{:>2}", ">", Style::new().green().for_stdout().apply_to(msg));
    handle_err(Term::stderr().write_line(&str), &str)
}

pub(crate) fn command_err(msg: &str) {
    let str = format!("{:<2}{:>2}", ">", Style::new().red().for_stdout().apply_to(msg));
    handle_err(Term::stderr().write_line(&str), &str)
}
