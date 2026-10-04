//! workflow-setup: see and change what a project gives to its coding agents.
//!
//!   workflow-setup [project-path]   opens the window
//!   workflow-setup help             lists the commands

mod adapters;
mod app;
mod apply;
mod cli;
mod config;
mod library;
mod manifest;
mod ops;
mod plugins;
mod scan;
mod sources;

use std::path::PathBuf;

use ely_gpui_component::Assets;
use gpui::{App, AppContext, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use app::{ChooseLibrary, Manager, OpenProject, Quit, Rescan, Save};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|first| cli::COMMANDS.contains(&first.as_str())) {
        if let Err(error) = cli::run(&args) {
            eprintln!("error: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    let project = args.first().map(PathBuf::from);
    gpui_platform::application().with_assets(Assets).run(move |cx: &mut App| {
        ely_gpui_component::init(cx);
        cx.bind_keys([
            KeyBinding::new("cmd-s", Save, None),
            KeyBinding::new("cmd-o", OpenProject, None),
            KeyBinding::new("cmd-shift-l", ChooseLibrary, None),
            KeyBinding::new("cmd-r", Rescan, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(1240.0), px(820.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("workflow-setup".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        cx.open_window(options, |window, cx| cx.new(|cx| Manager::new(project, window, cx)))
            .expect("the window failed to open");
        cx.activate(true);
    });
}
