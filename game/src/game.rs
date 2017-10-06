use super::SHADER_ROOT;
use super::errors::Result;
use super::hud::{Hud, Bindings as HudBindings};
use super::level::{Level, Config as LevelConfig};
use super::player::{Player, Config as PlayerConfig, Bindings as PlayerBindings};
use super::wad_system::{WadSystem, Config as WadConfig};
use engine::{Input, Window, Projections, FrameTimers, Uniforms, Materials, Shaders, Renderer,
             Meshes, Entities, Transforms, TextRenderer, System, Context, ContextBuilder,
             WindowConfig, ShaderConfig};
use engine::type_list::Peek;
use std::marker::PhantomData;
use std::path::PathBuf;


pub struct Game(Box<AbstractGame>);

impl Game {
    pub fn new(config: GameConfig) -> Result<Self> {
        let context = ContextBuilder::new()
            .inject(WindowConfig {
                width: config.width,
                height: config.height,
                title: format!("Rusty Doom v{}", config.version),
            })
            .inject(ShaderConfig { root_path: SHADER_ROOT.into() })
            .inject(WadConfig {
                wad_path: config.wad_file.clone(),
                metadata_path: config.metadata_file.clone(),
            })
            .inject(HudBindings::default())
            .inject(LevelConfig { index: config.initial_level_index })
            .inject(PlayerBindings::default())
            .inject(PlayerConfig::default())
            .system(Window::bind())?
            .system(Input::bind())?
            .system(TextRenderer::bind())?
            .system(Entities::bind())?
            .system(FrameTimers::bind())?
            .system(Transforms::bind())?
            .system(Projections::bind())?
            .system(Shaders::bind())?
            .system(Uniforms::bind())?
            .system(Meshes::bind())?
            .system(Materials::bind())?
            .system(Renderer::bind())?
            .system(WadSystem::bind())?
            .system(Level::bind())?
            .system(Hud::bind())?
            .system(Player::bind())?
            .build()?;

        Ok(Game(ContextWrapper::boxed(context)))
    }

    pub fn run(&mut self) -> Result<()> {
        let result = self.0.run();
        result.and(self.0.destroy())
    }

    pub fn num_levels(&self) -> usize {
        self.0.num_levels()
    }

    pub fn load_level(&mut self, level_index: usize) -> Result<()> {
        self.0.load_level(level_index)
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        let _ = self.0.destroy();
    }
}

pub struct ContextWrapper<LevelIndexT, WadIndexT, ContextT> {
    context: ContextT,
    phantom: PhantomData<(LevelIndexT, WadIndexT)>,
}

impl<LevelIndexT, WadIndexT, ContextT> ContextWrapper<LevelIndexT, WadIndexT, ContextT>
where
    ContextT: Context + Peek<Level, LevelIndexT> + Peek<WadSystem, WadIndexT> + 'static,
    WadIndexT: 'static,
    LevelIndexT: 'static,
{
    fn boxed(context: ContextT) -> Box<AbstractGame> {
        Box::new(ContextWrapper {
            context,
            phantom: PhantomData,
        })
    }
}

pub trait AbstractGame {
    fn run(&mut self) -> Result<()>;
    fn destroy(&mut self) -> Result<()>;
    fn num_levels(&self) -> usize;
    fn load_level(&mut self, level_index: usize) -> Result<()>;
}

impl<LevelIndexT, WadIndexT, ContextT> AbstractGame
    for ContextWrapper<LevelIndexT, WadIndexT, ContextT>
where
    ContextT: Context + Peek<Level, LevelIndexT> + Peek<WadSystem, WadIndexT> + 'static,
{
    fn num_levels(&self) -> usize {
        let wad: &WadSystem = self.context.peek();
        wad.archive.num_levels()
    }

    fn load_level(&mut self, level_index: usize) -> Result<()> {
        {
            let level: &mut Level = self.context.peek_mut();
            level.change_level(level_index);
        }
        self.context.step()?;
        self.context.step()?;
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        self.context.run()?;
        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        self.context.destroy()?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct GameConfig {
    pub wad_file: PathBuf,
    pub metadata_file: PathBuf,
    pub fov: f32,
    pub width: u32,
    pub height: u32,
    pub version: &'static str,
    pub initial_level_index: usize,
}
