use std::str::Split;

use crate::game::GameData;

type CommandFn = fn(&mut GameData, Split<'_, char>);

pub struct Command {
    pub name: &'static str,
    pub help: &'static str,
    pub func: CommandFn,
}

impl Command {
    pub const fn new(name: &'static str, help: &'static str, func: CommandFn) -> Self {
        Self { name, help, func }
    }
}

pub const COMMANDS: [Command; 4] = [
    Command::new("exit", "Exit the game.", |g, _| g.running = false),
    Command::new("commit", "Print the commit hash.", |_, _| {
        println!("{}", env!("BUILD_COMMIT_HASH"))
    }),
    Command::new("test-args", "Print all arguments.", |_, args| {
        for (i, arg) in args.enumerate() {
            println!("{i}: {arg}");
        }
    }),
    Command::new(
        "get-error",
        "Print the return value of SDL_GetError().",
        |_, _| println!("\"{}\"", unsafe { halcyon::error::get_str() }),
    ),
];
