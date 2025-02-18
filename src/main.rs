use anyhow::Ok;
use app::App;
use clap::Parser;

mod app;

fn main() -> anyhow::Result<()> {
    let app = App::parse();

    let context = app.run()?;
    println!("{}", serde_json::to_string(&context)?);

    Ok(())
}
