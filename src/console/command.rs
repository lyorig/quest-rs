use std::str::Split;

use halcyon::traits::Resource;

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
                let fmt = format!("help: unknown command {cmd}");
                out.writer.writeln(&fmt);
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
    out.writer.writeln(env!("BUILD_COMMIT_HASH"));
}

fn cmd_test_args(_: &mut Resources, out: &mut CachedData, args: ArgsSplit) {
    for (i, arg) in args.enumerate() {
        let fmt = format!("{i}: {arg}");
        out.writer.writeln(&fmt);
    }
}

fn cmd_font(game: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        out.writer.writeln("usage: font <subcommand>");
        return;
    };

    match arg {
        "gc" => {
            let msg = format!("freed {} glyphs", game.font_gc_all());
            out.writer.writeln(&msg);
        }
        "list" => {
            for (i, f) in game.fonts.iter().map(|f| f.font.as_ref()).enumerate() {
                let fam = f.family();
                let mono = if f.is_mono() { " (mono)" } else { "" };
                let msg = format!("{i}: {fam}{mono}");
                out.writer.writeln(&msg);
            }
        }
        _ => {
            let fmt = format!("font: unknown subcommand \"{arg}\"");
            out.writer.writeln(&fmt);
        }
    }
}

fn cmd_atlas(game: &mut Resources, out: &mut CachedData, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        out.writer.writeln("usage: atlas <subcommand>");
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
                    Err(e) => {
                        let msg = format!("Cannot init viewer: {e}");
                        out.writer.writeln(&msg)
                    }
                }
            }
        }
        "close" => game.atlas.viewer = None,
        "list" => {
            for (i, data) in game.atlas.data().enumerate() {
                let msg = format!("{i}: {data}");
                out.writer.writeln(&msg);
            }
        }
        _ => {
            let fmt = format!("atlas: unknown subcommand \"{arg}\"");
            out.writer.writeln(&fmt);
        }
    }
}

fn cmd_clear(game: &mut Resources, cd: &mut CachedData, _: ArgsSplit) {
    cd.clear(game);
}

const COMMANDS: [Command; 7] = [
    // NOTE: `help` needs to be the first command.
    Command::new("help", "Print a command's provided help text.", cmd_help),
    Command::new("exit", "Exit the game (push a quit event).", cmd_exit),
    Command::new("atlas", "Manipulate the texture atlas.", cmd_atlas),
    Command::new("commit", "Print the commit hash.", cmd_commit),
    Command::new("test-args", "Print all arguments.", cmd_test_args),
    Command::new("font", "Manipulate game fonts.", cmd_font),
    Command::new("clear", "Clear the console.", cmd_clear),
];

pub fn find(name: &str) -> Option<&Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

pub fn help_iter() -> impl Iterator<Item = &'static str> {
    COMMANDS.iter().map(|c| c.help)
}

fn help_exact(cmd: &Command, out: &mut CachedData) {
    let data = format!("help: {} => {}", cmd.name, cmd.help);
    out.writer.writeln(&data)
}
