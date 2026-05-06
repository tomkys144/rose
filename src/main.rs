use anyhow::Result;

mod gui;
mod models;

fn main() -> Result<()> {
    gui::run_app()?;

    Ok(())
}
