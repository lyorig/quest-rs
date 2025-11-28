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

fn cmd_help(_: &mut GameData, mut args: ArgsSplit<'_>) {
    help(args.next());
}

fn cmd_exit(g: &mut GameData, _: ArgsSplit<'_>) {
    g.running = false;
}

fn cmd_commit(_: &mut GameData, _: ArgsSplit<'_>) {
    println!("{}", env!("BUILD_COMMIT_HASH"))
}

fn cmd_test_args(_: &mut GameData, args: ArgsSplit<'_>) {
    for (i, arg) in args.enumerate() {
        println!("{i}: {arg}");
    }
}

fn cmd_get_error(_: &mut GameData, _: ArgsSplit<'_>) {
    println!("\"{}\"", unsafe { halcyon::error::get_str() })
}

const COMMANDS: [Command; 5] = [
    Command::new("help", "Print a command's provided help text.", cmd_help),
    Command::new("exit", "Exit the game.", cmd_exit),
    Command::new("commit", "Print the commit hash.", cmd_commit),
    Command::new("test-args", "Print all arguments.", cmd_test_args),
    Command::new("get-error", "Print SDL_GetError().", cmd_get_error),
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
