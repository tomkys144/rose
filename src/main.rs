use anyhow::Result;

mod gui;
mod models;
mod proc;

fn main() -> Result<()> {
    gui::run_app()?;

    Ok(())
}
