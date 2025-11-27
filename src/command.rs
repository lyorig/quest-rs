use std::str::Split;

use crate::game::GameData;

type ArgsSplit<'a> = Split<'a, char>;
type CommandFn = fn(&mut GameData, ArgsSplit);

pub struct Command {
    name: &'static str,
    help: &'static str,
    func: CommandFn,
}

impl Command {
    const fn new(name: &'static str, help: &'static str, func: CommandFn) -> Self {
        Self { name, help, func }
    }

    pub fn execute(&self, data: &mut GameData, args: ArgsSplit<'_>) {
        (self.func)(data, args)
    }
}

const COMMANDS: [Command; 5] = [
    Command::new(
        "help",
        "Print a command's provided help text.",
        |_, mut args| help(args.next()),
    ),
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

pub fn find(name: &str) -> Option<&Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

fn help(cmd: Option<&str>) {
    match cmd {
        Some(cmd) => match find(cmd) {
            Some(c) => help_exact(c),
            None => println!("help: unknown command {cmd}"),
        },
        None => help_exact(&COMMANDS[0]),
    }
}

fn help_exact(cmd: &Command) {
    println!("help: {} => {}", cmd.name, cmd.help)
}
