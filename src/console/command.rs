use std::str::Split;

use crate::{console::writer::ConsoleWriter, game::resources::GameResources};

type ArgsSplit<'a> = Split<'a, char>;
type CommandFn = fn(&mut GameResources, &mut ConsoleWriter, ArgsSplit);

pub struct Command {
    name: &'static str,
    help: &'static str,
    func: CommandFn,
}

impl Command {
    const fn new(name: &'static str, help: &'static str, func: CommandFn) -> Self {
        Self { name, help, func }
    }

    pub fn execute(&self, data: &mut GameResources, out: &mut ConsoleWriter, args: ArgsSplit<'_>) {
        (self.func)(data, out, args)
    }
}

fn cmd_help(_: &mut GameResources, out: &mut ConsoleWriter, mut args: ArgsSplit<'_>) {
    let cmd = args.next();
    match cmd {
        Some(cmd) => match find(cmd) {
            Some(c) => help_exact(c, out),
            None => out.write(&format!("help: unknown command {cmd}")),
        },
        None => help_exact(&COMMANDS[0], out),
    }
}

fn cmd_exit(g: &mut GameResources, _: &mut ConsoleWriter, _: ArgsSplit<'_>) {
    g.running = false;
}

fn cmd_commit(_: &mut GameResources, out: &mut ConsoleWriter, _: ArgsSplit<'_>) {
    out.write(env!("BUILD_COMMIT_HASH"));
}

fn cmd_test_args(_: &mut GameResources, out: &mut ConsoleWriter, args: ArgsSplit<'_>) {
    for (i, arg) in args.enumerate() {
        out.write(&format!("{i}: {arg}\n"));
    }
}

fn cmd_get_error(_: &mut GameResources, out: &mut ConsoleWriter, _: ArgsSplit<'_>) {
    out.write(&format!("\"{}\"", unsafe { halcyon::error::get_str() }))
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

fn help_exact(cmd: &Command, out: &mut ConsoleWriter) {
    out.write(&format!("help: {} => {}", cmd.name, cmd.help))
}
