use color_eyre::eyre::Result;

mod EngineCore;


fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;



    Ok(())
}