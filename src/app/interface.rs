use crossterm::{
    Command, 
    execute, 
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use crossterm::event::{KeyEvent, KeyCode::{Char}, KeyModifiers};
use std::io::{stdout, Error, Stdout};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState, Widget, canvas::{Canvas, Map, MapResolution, Points}},
};

// Internal imports 
pub mod coordinates;
use coordinates::LongLatCoordinates;

use crate::app::parser::TleData;

pub enum System{
    Quit,
}
pub struct Interface{
    terminal: Terminal<CrosstermBackend<Stdout>>, 
    table_state: TableState
}


impl Interface{
    pub fn new() -> Result<Self, Error>{
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        let backend = CrosstermBackend::new(stdout()); 
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Ok(Interface { terminal, table_state })
    }
    
    pub fn terminate(mut self) -> Result<(), Error>{
        Self::leave_alternate_screen()?;
        disable_raw_mode()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw(&mut self, pos:LongLatCoordinates, target: &TleData, history: &[LongLatCoordinates]) -> Result<(), Error>{
        
        self.terminal.draw(|frame| {
            let area = frame.area();

            let horizontal_layouts  = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1) 
                .constraints(vec![Constraint::Percentage(70), Constraint::Length(1), Constraint::Percentage(30)]) // Length 1 is a spacer
                .split(area);

            let canvas = Self::get_map_canvas(pos, history);
            let infos = Self::get_table(pos, target);
            
            frame.render_widget(canvas, horizontal_layouts[0]);
            // frame.render_widget(infos, horizontal_layouts[2]);
            frame.render_stateful_widget(infos, horizontal_layouts[2], &mut self.table_state);
            // frame.render_widget(Paragraph::new("Name satellite"), horizontal_layouts[1]);
        })?;
        Ok(())
    }

    fn get_table(pos:LongLatCoordinates, target: &TleData) -> Table<'static>{
        let rows = [Row::new(vec![
            target.name.clone(),
            target.norad_id.clone(), 
           format!("{:.2}", pos.longitude), 
           format!("{:.2}", pos.latitude)
            ])
        ];
        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ];
        Table::new(rows, widths)
            .block(Block::default()
                .title(" Satellites Menu ")
                .padding(Padding::uniform(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded))
    
            .column_spacing(1)
            .header(
                Row::new(vec!["Name", "Norad ID", "Longitude", "Latitude"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1)
                
            )
            // The selected row, column, cell and its content can also be styled.
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
        
    }

    fn get_map_canvas(pos:LongLatCoordinates, history: &[LongLatCoordinates]) -> impl Widget{
        Canvas::default()
                .block(Block::default()
                    .title(" Orbital Tracker - Radar 2D (CTRL+q to quit) ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                )
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .paint(move |ctx| {
                    // 1. Draw World Map as background
                    ctx.draw(&Map {
                        color: Color::White,
                        resolution: MapResolution::High,
                    });

                    let coords_tuples : Vec<(f64,f64)> = history
                        .iter()
                        .map(|coord| (coord.longitude, coord.latitude))
                        .collect();

                         
                    ctx.draw(&Points{
                        coords: &coords_tuples,
                        color:Color::Cyan
                    });
                    

                    // Draw satellite
                    ctx.print(pos.longitude, pos.latitude, "🛰️");
                })
        
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