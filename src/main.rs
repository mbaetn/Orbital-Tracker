// External imports

// Local imports
mod app;
use app::App;



fn main() -> Result<(), Box<dyn std::error::Error>>{

    let mut app = App::init()?;
    let _= app.run()?;

    Ok(())


}

