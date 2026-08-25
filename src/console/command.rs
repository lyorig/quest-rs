use std::{fmt::Write, str::Split};

use halcyon::{error::Error, resource::Resource};

use crate::{
    atlas::viewer::Viewer,
    console::cache::CachedData,
    game::{Game, resources::Resources},
};

type ArgsSplit<'a> = Split<'a, char>;
type CommandFn = fn(&mut Resources, ArgsSplit);

pub struct Command {
    name: &'static str,
    help: &'static str,
    func: CommandFn,
}

impl Command {
    const fn new(name: &'static str, help: &'static str, func: CommandFn) -> Self {
        Self { name, help, func }
    }

    pub fn execute(&self, res: &mut Resources, args: ArgsSplit) {
        (self.func)(res, args)
    }
}

fn cmd_help(res: &mut Resources, mut args: ArgsSplit) {
    let cmd = args.next();
    match cmd {
        Some(cmd) => match find(cmd) {
            Some(c) => help_exact(c, &mut res.console_cache),
            None => {
                _ = writeln!(res.writer(), "help: unknown command {cmd}");
            }
        },
        // No command provided, print help for the command itself.
        None => help_exact(&COMMANDS[0], &mut res.console_cache),
    }
}

fn cmd_exit(_: &mut Resources, _: ArgsSplit) {
    Game::quit();
}

fn cmd_commit(res: &mut Resources, _: ArgsSplit) {
    _ = writeln!(res.writer(), env!("BUILD_COMMIT_HASH"));
}

fn cmd_test_args(res: &mut Resources, args: ArgsSplit) {
    for (i, arg) in args.enumerate() {
        _ = writeln!(res.writer(), "{i}: {arg}");
    }
}

fn cmd_font(res: &mut Resources, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        _ = writeln!(res.writer(), "usage: font <subcommand>");
        return;
    };

    match arg {
        "list" => {
            for (i, f) in res.fonts.iter().map(|f| f.font.as_ref()).enumerate() {
                let fam = f.family();
                let mono = if f.is_mono() { " (mono)" } else { "" };
                _ = writeln!(res.console_cache.writer, "{i}: {fam}{mono}");
            }
        }
        _ => _ = writeln!(res.writer(), "font: unknown subcommand \"{arg}\""),
    }
}

fn cmd_atlas(res: &mut Resources, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        _ = writeln!(res.writer(), "usage: atlas <subcommand>");
        return;
    };

    match arg {
        "open" => {
            if let Some(surf) = res.read_atlas_pixels() {
                match Viewer::new() {
                    Ok(viewer) => {
                        let s = surf.expect("Cannot read atlas pixels");
                        viewer.update(s, res.atlas.areas());

                        res.atlas.viewer = Some(viewer);
                    }
                    Err(e) => _ = writeln!(res.writer(), "Cannot init viewer: {e}"),
                }
            }
        }
        "close" => res.atlas.viewer = None,
        "list" => {
            for (i, data) in res.atlas.data().enumerate() {
                _ = writeln!(res.console_cache.writer, "{i}: {data}");
            }
        }
        _ => _ = writeln!(res.writer(), "atlas: unknown subcommand \"{arg}\""),
    }
}

fn cmd_clear(game: &mut Resources, _: ArgsSplit) {
    game.console_cache.clear(&mut game.fonts);
}

fn cmd_commands(res: &mut Resources, mut args: ArgsSplit) {
    _ = writeln!(res.writer(), "available commands:");
    if args.next().is_some_and(|c| c == "--with-help") {
        COMMANDS.iter().for_each(|c| {
            _ = writeln!(res.writer(), "{}: {}", c.name, c.help);
        });
    } else {
        COMMANDS
            .iter()
            .map(|c| c.name)
            .for_each(|n| _ = writeln!(res.writer(), "{n}"));
    }
}

fn cmd_last_error(res: &mut Resources, _: ArgsSplit) {
    _ = writeln!(res.writer(), "\"{}\"", Error::current());
}

const COMMANDS: [Command; 9] = [
    // NOTE: `help` needs to be the first command.
    Command::new("help", "Print a command's provided help text.", cmd_help),
    Command::new("exit", "Exit the game (push a quit event).", cmd_exit),
    Command::new("atlas", "Manipulate the texture atlas.", cmd_atlas),
    Command::new("commit", "Print the commit hash.", cmd_commit),
    Command::new("test-args", "Enumerate all arguments.", cmd_test_args),
    Command::new("font", "Manipulate game fonts.", cmd_font),
    Command::new("clear", "Clear the console.", cmd_clear),
    Command::new("commands", "List all commands.", cmd_commands),
    Command::new("last-error", "Prints SDL_GetError().", cmd_last_error),
];

pub fn find(name: &str) -> Option<&Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

pub fn help_iter() -> impl Iterator<Item = &'static str> {
    COMMANDS.iter().map(|c| c.help)
}

fn help_exact(cmd: &Command, out: &mut CachedData) {
    _ = writeln!(out.writer, "help: {} => {}", cmd.name, cmd.help);
}
