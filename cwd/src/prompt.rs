//! "dialoguer" is a good library for creating command line prompts, but I really dislike its
//! OOP-based patterns. This file basically provides a few helper functions wrapping the dialoguer
//! functions, so that instead of
//!
//! ```ignore
//! let proceed = Confirm::new().with_prompt("broadcast tx?").interact()?;
//! ```
//!
//! We simply do
//!
//! ```ignore
//! let proceed = prompt::confirm("broacast tx")?;
//! ```

use std::io;

pub fn confirm(prompt: impl Into<String>) -> io::Result<bool> {
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .interact()
}

pub fn input(prompt: impl Into<String>) -> io::Result<String> {
    dialoguer::Input::new()
        .with_prompt(prompt)
        .report(false)
        .interact_text()
}

pub fn password(prompt: impl Into<String>) -> io::Result<String> {
    dialoguer::Password::new()
        .with_prompt(prompt)
        .report(true)
        .interact()
}
