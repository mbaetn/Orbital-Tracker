use crossterm::{
    Command,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use std::io::{stdout, Error, Stdout};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState, Widget, canvas::{Canvas, Map, MapResolution, Points}},
};

// Internal imports
pub mod coordinates;
use coordinates::LongLatCoordinates;

use crate::app::{SatelliteState};

pub struct Interface{
    terminal: Terminal<CrosstermBackend<Stdout>>,
    table_state: TableState,
    visible_rows: usize,
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
        let visible_rows = 0;
        Ok(Interface { terminal, table_state , visible_rows})
    }

    pub fn terminate(&mut self) -> Result<(), Error>{
        Self::leave_alternate_screen()?;
        disable_raw_mode()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn get_visible_rows(&self)-> usize{
        self.visible_rows
    }


    pub fn set_selected_row(&mut self, index:usize){
        self.table_state.select(Some(index));
    }

    // region drawing
    pub fn draw(&mut self, tracked_positions: Vec<SatelliteState>,  history: &[LongLatCoordinates], current_page:usize, nb_pages:usize) -> Result<(), Error>{

        self.terminal.draw(|frame| {
            let area = frame.area();

            let outer_layout  = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(vec![Constraint::Percentage(70), Constraint::Length(1), Constraint::Percentage(30)]) // Length 1 is a spacer
                .split(area);
            let inner_layer = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer_layout[2]);

            self.visible_rows = (inner_layer[0].height as usize).saturating_sub(7);

            let current_selection= self.table_state.selected().expect("No satellite selected");
            let pos  = if tracked_positions.is_empty(){
                LongLatCoordinates::default()
            } else {
                tracked_positions[current_selection].pos
            };
            let canvas = Self::get_map_canvas(&tracked_positions, history, current_selection);
            let infos = Self::get_table(&tracked_positions, current_page, nb_pages);
            let zoomed_map = Self::get_zoomed_canvas(pos);

            frame.render_widget(canvas, outer_layout[0]);
            frame.render_stateful_widget(infos, inner_layer[0], &mut self.table_state);
            frame.render_widget(zoomed_map, inner_layer[1]);
        })?;
        Ok(())
    }

    fn get_table(data: &[SatelliteState], current_page:usize, nb_pages:usize) -> Table<'static>{

        let rows:Vec<_> = data.iter().map(|target| Row::new(
            [
            target.name.clone(),
            format!("{:.2}",target.pos.longitude),
            format!("{:.2}",target.pos.latitude),
            ])
        ).collect();

        let widths = [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ];
        Table::new(rows, widths)
            .block(Block::default()
                .title(format!(" Satellites Menu - {}/{} ", current_page, nb_pages))
                .padding(Padding::uniform(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded))

            .column_spacing(1)
            .header(
                Row::new(vec!["Name","Longitude", "Latitude"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1)

            )
            // The selected row, column, cell and its content can also be styled.
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>")

    }

    fn get_zoomed_canvas(pos:LongLatCoordinates) -> impl Widget{
        let zoom_space = 20.0;
        Canvas::default()
            .block(Block::default()
                .title(" Zoom ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
            )
            .x_bounds([pos.longitude - zoom_space, pos.longitude + zoom_space])
            .y_bounds([pos.latitude - zoom_space*0.5, pos.latitude + zoom_space*0.5])
            .paint(move |ctx| {
                ctx.draw(&Map{
                    color: Color::White,
                    resolution: MapResolution::High,
                });

                ctx.print(pos.longitude, pos.latitude, "🛰️");
            })

    }

    fn get_map_canvas(tracked_positions: &[SatelliteState], history: &[LongLatCoordinates], current_selection:usize) -> impl Widget{
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
                        color:Color::Blue
                    });

                    for target in tracked_positions {
                        ctx.print(
                            target.pos.longitude,
                            target.pos.latitude,
                            ratatui::text::Span::styled("+", Style::new().fg(Color::Yellow)),
                        );
                    }


                    let pos = if tracked_positions.is_empty(){
                        LongLatCoordinates::default()
                    }else {
                        tracked_positions[current_selection].pos
                    };
                    // Draw satellite
                    ctx.print(pos.longitude, pos.latitude, "🛰️");
                })

        }



    // region system handling
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


impl Drop for Interface{
    fn drop(&mut self){
        let _= self.terminate();
    }
}