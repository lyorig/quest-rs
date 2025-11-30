use crate::console::active::ActiveConsole;

pub enum ConsoleState {
    Disabled,
    Enabled(ActiveConsole),
}
