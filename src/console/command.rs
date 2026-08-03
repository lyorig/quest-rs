use std::{fmt::Write, str::Split};

use halcyon::{error::Error, resource::Resource};

use crate::{
    atlas::viewer::Viewer,
    console::CachedData,
    game::{Game, resources::Resources},
};

type ArgsSplit<'a> = Split<'a, char>;
type CommandFn = fn(&mut Resources, &mut CachedData, ArgsSplit);

pub struct Command {
    name: &'static str,
    help: &'static str,
    func: CommandFn,
}

impl Command {
    const fn new(name: &'static str, help: &'static str, func: CommandFn) -> Self {
        Self { name, help, func }
    }

    pub fn execute(&self, data: &mut Resources, out: &mut CachedData, args: ArgsSplit) {
        (self.func)(data, out, args)
    }
}

fn cmd_help(_: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    let cmd = args.next();
    match cmd {
        Some(cmd) => match find(cmd) {
            Some(c) => help_exact(c, out),
            None => {
                _ = writeln!(out.writer, "help: unknown command {cmd}");
            }
        },
        // No command provided, print help for the command itself.
        None => help_exact(&COMMANDS[0], out),
    }
}

fn cmd_exit(_: &mut Resources, _: &mut CachedData, _: ArgsSplit) {
    Game::quit();
}

fn cmd_commit(_: &mut Resources, out: &mut CachedData, _: ArgsSplit) {
    _ = writeln!(out.writer, env!("BUILD_COMMIT_HASH"));
}

fn cmd_test_args(_: &mut Resources, out: &mut CachedData, args: ArgsSplit) {
    for (i, arg) in args.enumerate() {
        _ = writeln!(out.writer, "{i}: {arg}");
    }
}

fn cmd_font(game: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        _ = writeln!(out.writer, "usage: font <subcommand>");
        return;
    };

    match arg {
        "list" => {
            for (i, f) in game.fonts.iter().map(|f| f.font.as_ref()).enumerate() {
                let fam = f.family();
                let mono = if f.is_mono() { " (mono)" } else { "" };
                _ = writeln!(out.writer, "{i}: {fam}{mono}");
            }
        }
        _ => _ = writeln!(out.writer, "font: unknown subcommand \"{arg}\""),
    }
}

fn cmd_atlas(game: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        _ = writeln!(out.writer, "usage: atlas <subcommand>");
        return;
    };

    match arg {
        "open" => {
            if let Some(surf) = game.read_atlas_pixels() {
                match Viewer::new() {
                    Ok(viewer) => {
                        let s = surf.expect("Cannot read atlas pixels");
                        viewer.update(s, game.atlas.areas());

                        game.atlas.viewer = Some(viewer);
                    }
                    Err(e) => _ = writeln!(out.writer, "Cannot init viewer: {e}"),
                }
            }
        }
        "close" => game.atlas.viewer = None,
        "list" => {
            for (i, data) in game.atlas.data().enumerate() {
                _ = writeln!(out.writer, "{i}: {data}");
            }
        }
        _ => _ = writeln!(out.writer, "atlas: unknown subcommand \"{arg}\""),
    }
}

fn cmd_clear(game: &mut Resources, cd: &mut CachedData, _: ArgsSplit) {
    cd.clear(game);
}

fn cmd_commands(_: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    _ = writeln!(out.writer, "available commands:");
    if args.next().is_some_and(|c| c == "--with-help") {
        COMMANDS.iter().for_each(|c| {
            _ = writeln!(out.writer, "{}: {}", c.name, c.help);
        });
    } else {
        COMMANDS
            .iter()
            .map(|c| c.name)
            .for_each(|n| _ = writeln!(out.writer, "{n}"));
    }
}

fn cmd_last_error(_: &mut Resources, out: &mut CachedData, _: ArgsSplit) {
    _ = writeln!(out.writer, "\"{}\"", Error::current());
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
