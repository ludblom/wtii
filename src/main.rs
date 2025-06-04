use color_eyre::Result;
use wtii::ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = ui::App::default().run(terminal);
    ratatui::restore();
    app_result
    // let test = api::search_for_creature("Bone Swarm").await;
    // println!("{:#?}", test);
    // Ok(())
}
