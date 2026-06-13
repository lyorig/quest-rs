use std::str::Split;

use crate::{
    atlas::viewer::Viewer, console::writer::ConsoleWriter, game::resources::GameResources,
};

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

    pub fn execute(&self, data: &mut GameResources, out: &mut ConsoleWriter, args: ArgsSplit) {
        (self.func)(data, out, args)
    }
}

fn cmd_help(_: &mut GameResources, out: &mut ConsoleWriter, mut args: ArgsSplit) {
    let cmd = args.next();
    match cmd {
        Some(cmd) => match find(cmd) {
            Some(c) => help_exact(c, out),
            None => {
                let fmt = format!("help: unknown command {cmd}");
                out.write(&fmt);
            }
        },
        // No command provided, print help for the command itself.
        None => help_exact(&COMMANDS[0], out),
    }
}

fn cmd_commit(_: &mut GameResources, out: &mut ConsoleWriter, _: ArgsSplit) {
    out.write(env!("BUILD_COMMIT_HASH"));
}

fn cmd_test_args(_: &mut GameResources, out: &mut ConsoleWriter, args: ArgsSplit) {
    for (i, arg) in args.enumerate() {
        let fmt = format!("{i}: {arg}");
        out.write(&fmt);
    }
}

fn cmd_font_gc(game: &mut GameResources, _: &mut ConsoleWriter, _: ArgsSplit) {
    game.font_gc_all();
}

fn cmd_atlas(game: &mut GameResources, out: &mut ConsoleWriter, mut args: ArgsSplit) {
    let Some(arg) = args.next() else {
        out.write("usage: atlas <subcommand>");
        return;
    };

    match arg {
        "open" => {
            if let Some(surf) = game.read_atlas_pixels() {
                let viewer = Viewer::new().expect("Cannot init atlas viewer");
                let s = surf.expect("Cannot read atlas pixels");
                viewer.update(s, game.atlas.areas());

                game.atlas.viewer = Some(viewer);
            }
        }
        "close" => game.atlas.viewer = None,
        _ => {
            let fmt = format!("atlas: unknown subcommand \"{arg}\"");
            out.write(&fmt);
        }
    }
}

const COMMANDS: [Command; 5] = [
    Command::new("atlas", "Manipulate the texture atlas.", cmd_atlas),
    Command::new("help", "Print a command's provided help text.", cmd_help),
    Command::new("commit", "Print the commit hash.", cmd_commit),
    Command::new("test-args", "Print all arguments.", cmd_test_args),
    Command::new("font-gc", "Perform GC on all game fonts.", cmd_font_gc),
];

pub fn find(name: &str) -> Option<&Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

fn help_exact(cmd: &Command, out: &mut ConsoleWriter) {
    out.write(&format!("help: {} => {}", cmd.name, cmd.help))
}
