use crossterm::{
    Command, 
    execute, 
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use crossterm::event::{KeyEvent, KeyCode::{Char}, KeyModifiers};
use std::io::{stdout, Error, Stdout};
use ratatui::{
    Terminal, 
    backend::CrosstermBackend, 
    layout::Position, 
    style::{Color}, 
    widgets::{Block, Borders, canvas::{Canvas, Map, MapResolution, Points, Line}},
};

// Internal imports 
pub mod coordinates;
use coordinates::LongLatCoordinates;

pub enum System{
    Quit,
}
pub struct Interface{
    terminal: Terminal<CrosstermBackend<Stdout>>
}


impl Interface{
    pub fn new() -> Result<Self, Error>{
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        let backend = CrosstermBackend::new(stdout()); 
        let terminal = Terminal::new(backend)?;
        Ok(Interface { terminal: terminal })
    }
    
    pub fn terminate() -> Result<(), Error>{
        Self::leave_alternate_screen()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw(&mut self, pos:LongLatCoordinates, text:&str, history: &[(f64, f64)]) -> Result<(), Error>{
        self.terminal.draw(|frame| {
            let area = frame.area();
            let canvas = Canvas::default()
                .block(Block::default()
                    .title(" Orbital Tracker - Radar 2D (CTRL+q to quit) ")
                    .borders(Borders::ALL)
                )
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .paint(|ctx| {
                    // 1. Draw World Map as background
                    ctx.draw(&Map {
                        color: Color::White,
                        resolution: MapResolution::High,
                    });

                   
                    ctx.draw(&Points{
                        coords: history,
                        color:Color::Cyan
                    });
                    

                    // Draw satellite
                    ctx.print(pos.longitude, pos.latitude, text.to_string());
                });
            frame.render_widget(canvas, area);
            frame.set_cursor_position(Position{ x:0, y: 0});
        })?;
        Ok(())
    }
    
    pub fn enter_alternate_screen() -> Result<(), Error>{
        Self::execute_command(EnterAlternateScreen)?;
        Ok(())
    }
    
    pub fn leave_alternate_screen()->Result<(), Error>{
        Self::execute_command(LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn execute_command<T: Command>(command: T)-> Result<(), Error>{
        execute!(stdout(), command)?;
        Ok(())
    }

} 


impl TryFrom<KeyEvent> for System{
    type Error=String; 
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent{
            code, modifiers,..
        } = event;

        if modifiers == KeyModifiers::CONTROL{
            match code{
                Char('q') => Ok(Self::Quit),
                _=> Err(format!("Unsupported Control+{code:?} combination")),
            }
        }
        else{
            Err(format!("Unsupported key code {code:?} or modifier {modifiers:?}"))
        }
    }
}