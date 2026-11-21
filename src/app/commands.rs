use crossterm::event::{KeyCode::{self, Char}, KeyEvent, KeyModifiers};

pub enum Command{
    Quit,
    Next,
    Previous,
    NextPage,
    PreviousPage
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
                _ => Err(format!("Unsupported Control+{code:?} combination"))
            }
        }
        else{
            Err(format!("Unsupported key code {code:?} or modifier {modifiers:?}"))
        }
    }
}