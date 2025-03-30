use color_eyre::Result;

mod api;
mod creature;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    // color_eyre::install()?;
    // let terminal = ratatui::init();
    // let app_result = ui::App::default().run(terminal);
    // ratatui::restore();
    // app_result
    let test = api::search_for_creature("observer").await;
    println!("{:#?}", test);
    Ok(())
}
