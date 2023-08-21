use EngineCore::connection::EngineEnvironment;
use color_eyre::eyre::Result;

mod EngineCore;


fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let engine = EngineCore::engine::Engine::connect(EngineCore::connection::ConnectionMode::Shell);


    Ok(())
}