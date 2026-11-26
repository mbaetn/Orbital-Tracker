use crossterm::event::{KeyCode::{self, Char}, KeyEvent, KeyModifiers};

/// Interaction with user
pub enum Command{
    // Table Navigation
    Quit,
    Next,
    Previous,
    NextPage,
    PreviousPage,
    // Search query to filter satellites
    InputChar(char),
    ToggleSearch,
    ClearSearch,
    RemoveChar,

}



impl TryFrom<KeyEvent> for Command{
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
        else if modifiers == KeyModifiers::NONE{
            match code{
                KeyCode::Down => Ok(Self::Next),
                KeyCode::Up => Ok(Self::Previous),
                KeyCode::Right => Ok(Self::NextPage),
                KeyCode::Left => Ok(Self::PreviousPage),
                KeyCode::Esc => Ok(Self::ClearSearch),
                KeyCode::Backspace => Ok(Self::RemoveChar),
                Char('/') => Ok(Self::ToggleSearch),
                Char(c) => Ok(Self::InputChar(c)),
                _ => Err(format!("Unsupported Control+{code:?} combination"))
            }
        }
        else{
            Err(format!("Unsupported key code {code:?} or modifier {modifiers:?}"))
        }
    }
}