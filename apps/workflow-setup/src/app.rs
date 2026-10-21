//! The window: the library in a sidebar, its skills and plugins in the
//! middle, and where each one is installed. Changes collect in a draft of
//! manifest.toml until Apply writes it and makes the disk match.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    feedback::{Callout, EmptyState, Toast, ToastViewport, Toaster},
    forms::{Input, InputEvent, SearchInput, TextInput},
    layout::ScrollArea,
    overlays::{ConfirmDialog, Dialog},
    primitives::{FocusScope, Icon, IconName, Severity, Tooltip},
    theme::{ActiveTheme, ControlSize, IconSize, Mode, Palette, Theme},
};
use gpui::{
    AnyElement, App, AppContext, ClickEvent, Context, Div, Entity, FocusHandle, FontWeight, Hsla,
    InteractiveElement, IntoElement, ParentElement, PathPromptOptions, Render, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Subscription, Window, actions, div, prelude::FluentBuilder, px, rgb, rgba,
};

use crate::{
    agents::{self, Agent},
    apply::{self, Applied, Env},
    config::Config,
    installed, library,
    manifest::{self, Entry, Scope},
    ops, plugins,
    scan::{self, home},
    sources::{self, Diff, Source},
};

actions!(workflow_setup, [OpenProject, ChooseLibrary, Rescan, Quit]);

const SIDEBAR_WIDTH: f32 = 236.0;
const DETAIL_WIDTH: f32 = 372.0;
const CONTENT_WIDTH: f32 = 860.0;
/// The width of one place's column in the coverage grid.
const SOCKET_WIDTH: f32 = 34.0;

// ---------- look ----------

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

/// The color that means "installed here".
fn signal() -> Hsla {
    hex(0x2f5bea)
}

/// A cool, quiet palette in place of Ely's warm one. Buttons stay ink.
pub fn install_theme(cx: &mut App) {
    let mut palette = Palette::light(false);
    palette.bg = hex(0xf4f5f7);
    palette.surface = hex(0xffffff);
    palette.sunken = hex(0xeceef2);
    palette.overlay = hex(0xffffff);
    palette.hover = hex(0xe8eaef);
    palette.active = hex(0xdfe3ea);
    palette.border = hex(0xe1e4ea);
    palette.border_strong = hex(0xc6ccd5);
    palette.fg = hex(0x1a1f2b);
    palette.fg_muted = hex(0x596171);
    palette.fg_subtle = hex(0x7a8291);
    palette.fg_disabled = hex(0xb2b8c2);
    palette.accent = hex(0x1a1f2b);
    palette.accent_hover = hex(0x2d3445);
    palette.on_accent = hex(0xffffff);
    palette.focus = signal();
    palette.link = signal();
    palette.info = signal();
    palette.info_subtle = hex(0xeaf0fe);
    palette.selection = rgba(0x2f5bea33).into();
    palette.glass = rgba(0xf4f5f7c8).into();
    palette.shadow = hex(0x1a1f2b);
    palette.tooltip_bg = hex(0x1a1f2b);
    palette.ink = hex(0x1a1f2b);
    Theme::set_palette(Mode::Light, Some(palette), cx);
    Theme::set_mode_now(Mode::Light, cx);
}

/// Each agent keeps one color everywhere it shows.
fn agent_color(agent: &Agent) -> Hsla {
    let name = agent.display.to_lowercase();
    if name.contains("claude") {
        hex(0xc0650f)
    } else if name.contains("codex") {
        hex(0x0b8576)
    } else if name.contains("opencode") {
        hex(0x6a4dd6)
    } else {
        hex(0x596171)
    }
}

/// A tile color for a project, the same on every run.
fn tint(label: &str) -> (Hsla, Hsla) {
    const TINTS: [(u32, u32); 6] = [
        (0xe4ecff, 0x2f5bea),
        (0xdff3ef, 0x0b7468),
        (0xfcecda, 0x9c520b),
        (0xece6fd, 0x5a3dc6),
        (0xfde6eb, 0xb0334d),
        (0xe5f1df, 0x3d7527),
    ];
    let sum = label.bytes().fold(0usize, |sum, byte| sum.wrapping_mul(31).wrapping_add(byte as usize));
    let (bg, fg) = TINTS[sum % TINTS.len()];
    (hex(bg), hex(fg))
}

#[derive(Clone, PartialEq)]
enum View {
    Skills,
    Plugins,
    Sources,
    Settings,
    /// Global, or one project, by its root.
    Place(PathBuf),
}

/// One thing the library can install: a whole plugin, or one of its skills.
#[derive(Clone)]
struct Item {
    entry: Entry,
    name: String,
    plugin: String,
    description: String,
    dir: PathBuf,
}

impl Item {
    fn is_skill(&self) -> bool {
        matches!(self.entry, Entry::Skill { .. })
    }
}

/// How one place has an item.
#[derive(Clone, Copy, PartialEq)]
enum Has {
    No,
    Direct,
    /// The whole plugin of the skill is installed.
    ViaPlugin,
}

#[derive(Clone)]
struct Place {
    root: PathBuf,
    label: String,
}

impl Place {
    fn global(&self) -> bool {
        self.root == home()
    }
}

/// What the last check of a source found.
enum Check {
    Busy,
    Ready(Diff),
    Current,
    Failed(String),
}

/// A new source, fetched, before it goes into sources.toml.
struct Fetched {
    source: Source,
    skills: Vec<String>,
}

enum Modal {
    AddSource { fetched: Option<Fetched>, picks: BTreeSet<String>, busy: bool, error: Option<String> },
    Review(String),
    RemoveSource(String),
    Foreign { root: PathBuf, name: String, paths: Vec<PathBuf> },
    Library,
}

struct SourceEdit {
    name: String,
    /// Every skill of the fetched repository. None while it fetches.
    skills: Option<Result<Vec<String>, String>>,
    picks: BTreeSet<String>,
    busy: bool,
}

pub struct Manager {
    focus: FocusHandle,
    config: Config,
    items: Vec<Item>,
    sources: Vec<Source>,
    /// manifest.toml as on disk.
    disk: String,
    /// manifest.toml with the changes that Apply has not written yet.
    draft: String,
    scopes: Vec<Scope>,
    /// The disk changes that Apply would make, when the draft differs.
    pending: Option<Result<usize, String>>,
    /// The roots whose disk does not match manifest.toml.
    drift: BTreeSet<PathBuf>,
    view: View,
    origin: Option<String>,
    selected: Option<Entry>,
    /// The place that the Skills and Plugins views add to.
    target: Option<PathBuf>,
    checks: BTreeMap<String, Check>,
    edit: Option<SourceEdit>,
    modal: Option<Modal>,
    query: Entity<TextInput>,
    spec: Entity<TextInput>,
    new_name: Entity<TextInput>,
    repo: Entity<TextInput>,
    branch: Entity<TextInput>,
    toaster: Entity<Toaster>,
    _subscriptions: Vec<Subscription>,
}

impl Manager {
    pub fn new(project: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut input = |placeholder: &'static str| cx.new(|cx| TextInput::new(window, cx).placeholder(placeholder));
        let (query, spec, new_name, repo, branch) = (
            input("Search skills"),
            input("owner/repo, a git URL, or a folder"),
            input("Name"),
            input("owner/repo, a git URL, or a folder"),
            input("Default branch"),
        );
        let subscriptions = vec![cx.subscribe(&query, |_, _, event: &InputEvent, cx| {
            if *event == InputEvent::Changed {
                cx.notify();
            }
        })];
        let mut config = Config::load();
        let mut view = View::Skills;
        if let Some(root) = project.and_then(|path| path.canonicalize().ok()).filter(|path| path.is_dir()) {
            if root != home() {
                config.opened(root.clone());
                config.save().ok();
            }
            view = View::Place(root);
        }
        let mut manager = Self {
            focus,
            config,
            items: Vec::new(),
            sources: Vec::new(),
            disk: String::new(),
            draft: String::new(),
            scopes: Vec::new(),
            pending: None,
            drift: BTreeSet::new(),
            view,
            origin: None,
            selected: None,
            target: None,
            checks: BTreeMap::new(),
            edit: None,
            modal: None,
            query,
            spec,
            new_name,
            repo,
            branch,
            toaster: cx.new(|_| Toaster::default()),
            _subscriptions: subscriptions,
        };
        manager.reload();
        manager
    }

    // ---------- state ----------

    fn library(&self) -> Option<PathBuf> {
        self.config.library.clone().filter(|library| library.is_dir())
    }

    fn env(&self) -> Option<Env> {
        self.library().map(Env::new)
    }

    /// Reads the library and manifest.toml again. A draft with changes stays.
    fn reload(&mut self) {
        let Some(env) = self.env() else {
            self.items.clear();
            self.sources.clear();
            self.scopes.clear();
            self.pending = None;
            self.drift.clear();
            return;
        };
        let library = env.library.clone();
        self.sources = sources::load(&library).unwrap_or_default();
        let plugins = library::plugins(&library);
        let skills = library::skills(&library);
        let mut items: Vec<Item> = plugins
            .iter()
            .map(|plugin| {
                let count = skills.iter().filter(|skill| skill.origin == *plugin).count();
                Item {
                    entry: Entry::Plugin(plugin.clone()),
                    name: plugin.clone(),
                    plugin: plugin.clone(),
                    description: format!("{}. {count} skills.", self.origin_label(plugin)),
                    dir: plugins::dir(&library, plugin),
                }
            })
            .collect();
        for skill in skills {
            let text = fs::read_to_string(skill.dir.join("SKILL.md")).unwrap_or_default();
            let description = scan::frontmatter(&text).remove("description").unwrap_or_default();
            items.push(Item {
                entry: Entry::Skill { plugin: skill.origin.clone(), skill: skill.name.clone() },
                name: skill.name,
                plugin: skill.origin,
                description,
                dir: skill.dir,
            });
        }
        self.items = items;
        let disk = manifest::read(&library);
        if self.draft == self.disk {
            self.draft = disk.clone();
        }
        self.disk = disk;
        self.drift = match apply::plan_all(&env) {
            Ok(plans) => plans.into_iter().filter(|plan| !plan.in_sync()).map(|plan| plan.scope.root).collect(),
            Err(_) => BTreeSet::new(),
        };
        if self.origin.as_ref().is_some_and(|origin| !plugins.contains(origin)) {
            self.origin = None;
        }
        self.settle_draft(&env);
    }

    /// Reads the draft: its scopes, and how many disk changes it brings.
    fn settle_draft(&mut self, env: &Env) {
        match manifest::parse(&self.draft, &env.home) {
            Ok(parsed) => {
                self.pending = (self.draft != self.disk).then(|| {
                    let draft = manifest::Manifest { scopes: parsed.scopes.clone() };
                    apply::plan_manifest(env, draft)
                        .map(|plans| plans.iter().map(|plan| plan.changes.len()).sum())
                        .map_err(|error| format!("{error:#}"))
                });
                self.scopes = parsed.scopes;
            }
            Err(error) => self.pending = Some(Err(format!("{error:#}"))),
        }
    }

    fn origin_label(&self, plugin: &str) -> String {
        if plugin == library::OWN {
            return "Your own skills, edited in this library".to_string();
        }
        match self.sources.iter().find(|source| source.name == plugin) {
            Some(source) => format!("From {}", short_repo(&source.repository)),
            None => "A plugin folder of the library".to_string(),
        }
    }

    /// Global, each project of the manifest, then each project opened here.
    fn places(&self) -> Vec<Place> {
        let home = home();
        let mut places = vec![Place { root: home.clone(), label: "Global".to_string() }];
        let manifest_roots = self.scopes.iter().filter(|scope| !scope.global).map(|scope| scope.root.clone());
        let opened = self.config.recent.iter().filter(|root| root.is_dir() && **root != home).cloned();
        for root in manifest_roots.chain(opened) {
            if !places.iter().any(|place| place.root == root) {
                places.push(Place { label: name_of(&root), root });
            }
        }
        places
    }

    fn place_label(&self, root: &Path) -> String {
        if root == home() { "Global".to_string() } else { name_of(root) }
    }

    fn scope(&self, root: &Path) -> Option<&Scope> {
        self.scopes.iter().find(|scope| scope.root == root)
    }

    fn agents_of(&self, root: &Path) -> Vec<&'static Agent> {
        self.scope(root).map(Scope::agents).filter(|agents| !agents.is_empty()).unwrap_or_else(agents::default)
    }

    fn has(&self, entry: &Entry, root: &Path, agent: &Agent) -> Has {
        let Some(scope) = self.scope(root) else { return Has::No };
        let given = scope.entries_for(agent);
        if given.contains(entry) {
            return Has::Direct;
        }
        match entry {
            Entry::Skill { plugin, .. } if given.contains(&Entry::Plugin(plugin.clone())) => Has::ViaPlugin,
            _ => Has::No,
        }
    }

    /// How a place has an item, over all its agents.
    fn has_in(&self, entry: &Entry, root: &Path) -> Has {
        let all: Vec<Has> = self.agents_of(root).iter().map(|agent| self.has(entry, root, agent)).collect();
        if all.contains(&Has::Direct) {
            Has::Direct
        } else if all.contains(&Has::ViaPlugin) {
            Has::ViaPlugin
        } else {
            Has::No
        }
    }

    /// Puts the item in the place, or takes it out. With `only`, for those agents.
    fn change(&mut self, entry: &Entry, root: &Path, only: &[&'static Agent], add: bool, cx: &mut Context<Self>) {
        let Some(env) = self.env() else { return };
        match manifest::edit(&self.draft, &env.home, root, std::slice::from_ref(entry), only, add) {
            Ok(text) => {
                self.draft = text;
                self.settle_draft(&env);
            }
            Err(error) => self.toast(Severity::Danger, "Not changed", format!("{error:#}"), cx),
        }
        cx.notify();
    }

    fn apply_draft(&mut self, cx: &mut Context<Self>) {
        let Some(env) = self.env() else { return };
        let result = manifest::write(&env.library, &self.draft)
            .and_then(|()| apply::plan_all(&env))
            .and_then(|plans| apply::apply(&env, &plans, false))
            .map(|(applied, _)| format!("Applied. {}", done(&applied)));
        self.finish(result, cx);
    }

    fn discard(&mut self, cx: &mut Context<Self>) {
        self.draft = self.disk.clone();
        if let Some(env) = self.env() {
            self.settle_draft(&env);
        }
        cx.notify();
    }

    fn toast(&mut self, severity: Severity, title: impl Into<SharedString>, body: String, cx: &mut Context<Self>) {
        self.toaster.update(cx, |toaster, cx| {
            let toast = Toast::new(title).severity(severity);
            toaster.push(if body.is_empty() { toast } else { toast.body(body) }, cx);
        });
    }

    /// Tells the user how a change went, then reads the library again.
    fn finish(&mut self, result: Result<String>, cx: &mut Context<Self>) {
        match result {
            Ok(done) => self.toast(Severity::Success, done, String::new(), cx),
            Err(error) => self.toast(Severity::Danger, "Not done", format!("{error:#}"), cx),
        }
        self.reload();
        cx.notify();
    }

    fn save_config(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.config.save() {
            self.toast(Severity::Danger, "Settings were not saved", format!("{error:#}"), cx);
        }
    }

    fn go(&mut self, view: View, cx: &mut Context<Self>) {
        if !matches!(view, View::Skills | View::Plugins) {
            self.target = None;
        }
        if view != self.view {
            self.selected = None;
            self.edit = None;
        }
        self.view = view;
        cx.notify();
    }

    /// Runs `work` off the main thread, then `then` with its result.
    fn background<T: Send + 'static>(
        &mut self,
        work: impl FnOnce() -> T + Send + 'static,
        then: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let task = cx.background_executor().spawn(async move { work() });
        cx.spawn(async move |this, cx| {
            let out = task.await;
            this.update(cx, |this, cx| {
                then(this, out, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn pick_folder(&mut self, library: bool, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(if library { "Use as library" } else { "Add project" }.into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            let Some(path) = paths.pop() else { return };
            let path = path.canonicalize().unwrap_or(path);
            this.update(cx, |this, cx| {
                if library {
                    this.config.library = Some(path);
                    this.draft.clear();
                    this.disk.clear();
                    this.checks.clear();
                    this.save_config(cx);
                    this.reload();
                    this.go(View::Settings, cx);
                } else {
                    if path != home() {
                        this.config.opened(path.clone());
                        this.save_config(cx);
                    }
                    this.go(View::Place(path), cx);
                }
            })
            .ok();
        })
        .detach();
    }

    // ---------- sources ----------

    fn source(&self, name: &str) -> Option<Source> {
        self.sources.iter().find(|source| source.name == name).cloned()
    }

    fn check_source(&mut self, name: String, cx: &mut Context<Self>) {
        let (Some(library), Some(source)) = (self.library(), self.source(&name)) else { return };
        self.checks.insert(name.clone(), Check::Busy);
        self.background(
            move || sources::fetch(&library, &source).and_then(|_| sources::diff(&library, &source)),
            move |this, result, _| {
                let check = match result {
                    Ok(diff) if diff.is_empty() => Check::Current,
                    Ok(diff) => Check::Ready(diff),
                    Err(error) => Check::Failed(format!("{error:#}")),
                };
                this.checks.insert(name, check);
            },
            cx,
        );
    }

    fn accept_source(&mut self, name: String, cx: &mut Context<Self>) {
        let (Some(library), Some(source)) = (self.library(), self.source(&name)) else { return };
        self.modal = None;
        self.checks.insert(name.clone(), Check::Busy);
        self.background(
            move || sources::accept(&library, &source),
            move |this, result, cx| {
                this.checks.remove(&name);
                this.finish(result.map(|commit| format!("Updated {name} to {}", short(&commit))), cx);
            },
            cx,
        );
    }

    fn start_edit(&mut self, name: String, cx: &mut Context<Self>) {
        let (Some(library), Some(source)) = (self.library(), self.source(&name)) else { return };
        self.repo.update(cx, |input, cx| input.set_text(source.repository.clone(), cx));
        self.branch.update(cx, |input, cx| input.set_text(source.branch.clone().unwrap_or_default(), cx));
        let picks = self.items.iter().filter(|item| item.is_skill() && item.plugin == name).map(|item| item.name.clone()).collect();
        self.edit = Some(SourceEdit { name: name.clone(), skills: None, picks, busy: false });
        if source.plugin {
            return cx.notify();
        }
        self.background(
            move || sources::fetch(&library, &source).and_then(|_| sources::available(&library, &source)),
            move |this, result, _| {
                if let Some(edit) = this.edit.as_mut().filter(|edit| edit.name == name) {
                    edit.skills = Some(result.map_err(|error| format!("{error:#}")));
                }
            },
            cx,
        );
    }

    fn save_edit(&mut self, cx: &mut Context<Self>) {
        let Some(library) = self.library() else { return };
        let Some(name) = self.edit.as_ref().map(|edit| edit.name.clone()) else { return };
        let Some(source) = self.source(&name) else { return };
        let spec = self.repo.read(cx).text().trim().to_string();
        let branch = self.branch.read(cx).text().trim().to_string();
        let Some(edit) = self.edit.as_mut() else { return };
        let all = edit.skills.as_ref().and_then(|skills| skills.as_ref().ok()).cloned();
        let skills = match (&all, source.plugin) {
            (_, true) => None,
            (Some(all), false) if all.iter().all(|skill| edit.picks.contains(skill)) => None,
            _ => Some(edit.picks.iter().cloned().collect::<Vec<_>>()),
        };
        edit.busy = true;
        self.background(
            move || -> Result<String> {
                if skills.as_ref().is_some_and(Vec::is_empty) {
                    bail!("Pick one skill or more");
                }
                let repository = if spec == source.repository { spec } else { sources::new_source(&spec, Some(&source.name), &[], &home())?.repository };
                let branch = (!branch.is_empty()).then_some(branch);
                sources::set_fields(&library, &source.name, &repository, branch.as_deref(), skills.as_deref())?;
                let source = sources::load(&library)?.into_iter().find(|known| known.name == source.name).context("the source is missing")?;
                sources::fetch(&library, &source)?;
                sources::diff(&library, &source)?;
                let commit = sources::accept(&library, &source)?;
                Ok(format!("Saved {} at {}", source.name, short(&commit)))
            },
            move |this, result, cx| {
                if result.is_ok() {
                    this.edit = None;
                    this.checks.remove(&name);
                } else if let Some(edit) = this.edit.as_mut() {
                    edit.busy = false;
                }
                this.finish(result, cx);
            },
            cx,
        );
    }

    fn remove_source(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(env) = self.env() else { return };
        self.modal = None;
        if self.draft != self.disk {
            self.toast(Severity::Warning, "Apply or discard your changes first", String::new(), cx);
            return cx.notify();
        }
        let result = apply::remove_source(&env, &name).map(|applied| format!("Removed {name}. {}", done(&applied)));
        if result.is_ok() {
            self.draft = String::new();
            self.disk = String::new();
            self.checks.remove(&name);
        }
        self.finish(result, cx);
    }

    fn open_add_source(&mut self, cx: &mut Context<Self>) {
        for input in [&self.spec, &self.new_name] {
            input.update(cx, |input, cx| input.set_text("", cx));
        }
        self.modal = Some(Modal::AddSource { fetched: None, picks: BTreeSet::new(), busy: false, error: None });
        cx.notify();
    }

    fn fetch_new(&mut self, cx: &mut Context<Self>) {
        let Some(library) = self.library() else { return };
        let spec = self.spec.read(cx).text().trim().to_string();
        let Some(Modal::AddSource { busy, error, .. }) = self.modal.as_mut() else { return };
        (*busy, *error) = (true, None);
        self.background(
            move || -> Result<Fetched> {
                let source = sources::new_source(&spec, None, &[], &home())?;
                if sources::check_new(&library, &source)? {
                    bail!("{} is a source already", source.name);
                }
                sources::fetch(&library, &source)?;
                let skills = sources::available(&library, &source)?;
                if skills.is_empty() {
                    bail!("{} has no folder with a SKILL.md", source.repository);
                }
                Ok(Fetched { source, skills })
            },
            move |this, result, cx| {
                let Some(Modal::AddSource { fetched, picks, busy, error }) = this.modal.as_mut() else { return };
                *busy = false;
                match result {
                    Ok(found) => {
                        *picks = found.skills.iter().cloned().collect();
                        let name = found.source.name.clone();
                        *fetched = Some(found);
                        this.new_name.update(cx, |input, cx| input.set_text(name, cx));
                    }
                    Err(failure) => *error = Some(format!("{failure:#}")),
                }
            },
            cx,
        );
    }

    fn add_new(&mut self, cx: &mut Context<Self>) {
        let Some(library) = self.library() else { return };
        let name = self.new_name.read(cx).text().trim().to_string();
        let Some(Modal::AddSource { fetched: Some(found), picks, busy, error }) = self.modal.as_mut() else { return };
        let mut source = found.source.clone();
        source.skills = (picks.len() < found.skills.len()).then(|| picks.iter().cloned().collect());
        (*busy, *error) = (true, None);
        self.background(
            move || -> Result<String> {
                source.name = sources::new_source(&source.repository, Some(&name), &[], &home())?.name;
                sources::write_source(&library, &source)?;
                let source = sources::load(&library)?.into_iter().find(|known| known.name == source.name).context("the new source is missing")?;
                sources::fetch(&library, &source)?;
                sources::diff(&library, &source)?;
                sources::accept(&library, &source)?;
                Ok(source.name)
            },
            move |this, result, cx| match result {
                Ok(name) => {
                    this.modal = None;
                    this.finish(Ok(format!("Added {name}")), cx);
                    this.go(View::Skills, cx);
                    this.origin = Some(name);
                }
                Err(failure) => {
                    if let Some(Modal::AddSource { busy, error, .. }) = this.modal.as_mut() {
                        (*busy, *error) = (false, Some(format!("{failure:#}")));
                    }
                }
            },
            cx,
        );
    }

    // ---------- drawing: frame ----------

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let updates = self.checks.values().filter(|check| matches!(check, Check::Ready(_))).count();
        let skills = self.items.iter().filter(|item| item.is_skill()).count();
        let library_name = self.library().map(|library| name_of(&library)).unwrap_or_else(|| "No library".to_string());

        let identity = div()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2()
            .pb_2()
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(30.0))
                    .rounded_lg()
                    .bg(colors.fg)
                    .child(Icon::new(IconName::Layers).size(IconSize::Sm).color(colors.on_accent)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("Workflow Setup"))
                    .child(div().text_xs().text_color(colors.fg_subtle).truncate().child(format!("Library {library_name}"))),
            );

        let mut browse = div().flex().flex_col().gap_0p5();
        for (id, icon, label, view, count) in [
            ("nav-skills", IconName::Sparkles, "Skills", View::Skills, skills.to_string()),
            ("nav-plugins", IconName::Package, "Plugins", View::Plugins, (self.items.len() - skills).to_string()),
            (
                "nav-sources",
                IconName::GitBranch,
                "Sources",
                View::Sources,
                match updates {
                    0 => String::new(),
                    1 => "1 update".to_string(),
                    count => format!("{count} updates"),
                },
            ),
        ] {
            let tint = (updates > 0 && view == View::Sources).then_some(signal());
            browse = browse.child(self.nav(id, nav_icon(icon), label, count, tint, view, cx));
        }
        for (icon, label) in [(IconName::Server, "MCP servers"), (IconName::Bot, "Agents"), (IconName::FileText, "Instructions")] {
            browse = browse.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .px_2()
                    .h(px(30.0))
                    .text_sm()
                    .text_color(colors.fg_disabled)
                    .child(div().flex().items_center().justify_center().size(px(20.0)).child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_disabled)))
                    .child(div().flex_1().child(label))
                    .child(div().text_xs().child("Soon")),
            );
        }

        let mut places = div().flex().flex_col().gap_0p5();
        for (ix, place) in self.places().into_iter().enumerate() {
            let drift = self.drift.contains(&place.root);
            let tile = monogram(&place, 20.0, &colors).into_any_element();
            let label = place.label.clone();
            let mut row = self.nav(("nav-place", ix), tile, &label, String::new(), None, View::Place(place.root.clone()), cx);
            if drift {
                row = row.child(div().size(px(7.0)).rounded_full().bg(colors.warning));
            }
            places = places.child(row.tooltip(Tooltip::text(if drift {
                format!("{}. The disk does not match manifest.toml.", apply::tilde(&home(), &place.root))
            } else {
                apply::tilde(&home(), &place.root)
            })));
        }
        places = places.child(
            div()
                .id("add-project")
                .flex()
                .items_center()
                .gap_2p5()
                .px_2()
                .h(px(30.0))
                .rounded_md()
                .text_sm()
                .text_color(colors.fg_muted)
                .cursor_pointer()
                .hover(|style| style.bg(colors.hover))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(20.0))
                        .rounded_md()
                        .border_1()
                        .border_dashed()
                        .border_color(colors.border_strong)
                        .child(Icon::new(IconName::Plus).size(IconSize::Xs).color(colors.fg_muted)),
                )
                .child("Add project")
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.pick_folder(false, window, cx))),
        );

        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .px_3()
            .pt_4()
            .pb_3()
            .gap_5()
            .border_r_1()
            .border_color(colors.border)
            .bg(colors.sunken)
            .child(identity)
            .child(div().flex().flex_col().gap_1().child(group_label("Library", &colors)).child(browse))
            .child(
                ScrollArea::new("places")
                    .flex_1()
                    .min_h_0()
                    .child(div().flex().flex_col().gap_1().child(group_label("Installed in", &colors)).child(places)),
            )
            .child(self.nav("nav-settings", nav_icon(IconName::Settings), "Settings", String::new(), None, View::Settings, cx))
    }

    #[allow(clippy::too_many_arguments)]
    fn nav(
        &self,
        id: impl Into<gpui::ElementId>,
        lead: AnyElement,
        label: &str,
        trailing: String,
        tint: Option<Hsla>,
        view: View,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = cx.theme().colors.clone();
        let active = self.view == view;
        div()
            .id(id.into())
            .flex()
            .items_center()
            .gap_2p5()
            .px_2()
            .h(px(30.0))
            .rounded_md()
            .text_sm()
            .cursor_pointer()
            .border_1()
            .when(active, |row| row.bg(colors.surface).border_color(colors.border).shadow_xs().font_weight(FontWeight::MEDIUM))
            .when(!active, |row| row.border_color(gpui::transparent_black()).text_color(colors.fg_muted).hover(|style| style.bg(colors.hover)))
            .child(lead)
            .child(div().flex_1().min_w_0().truncate().when(active, |text| text.text_color(colors.fg)).child(label.to_string()))
            .child(div().text_xs().text_color(tint.unwrap_or(colors.fg_subtle)).child(trailing))
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.go(view.clone(), cx)))
    }

    /// The draft's changes, floating over the bottom of the main column.
    fn apply_bar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let pending = self.pending.as_ref()?;
        let colors = cx.theme().colors.clone();
        let (text, ok) = match pending {
            Ok(0) => ("manifest.toml changed. The disk matches already.".to_string(), true),
            Ok(1) => ("1 change not applied".to_string(), true),
            Ok(count) => (format!("{count} changes not applied"), true),
            Err(error) => (format!("This change cannot apply. {}", clip(error, 120)), false),
        };
        let pill = |id: &'static str, label: &'static str, strong: bool| {
            div()
                .id(id)
                .px_3()
                .py_1()
                .rounded_full()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .cursor_pointer()
                .when(strong, |pill| pill.bg(colors.on_accent).text_color(colors.fg).hover(|style| style.bg(hex(0xdfe3ea))))
                .when(!strong, |pill| pill.text_color(colors.on_accent.opacity(0.75)).hover(|style| style.bg(colors.on_accent.opacity(0.12))))
                .child(label)
        };
        Some(
            div()
                .absolute()
                .bottom_5()
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .pl_4()
                        .pr_1p5()
                        .py_1p5()
                        .rounded_full()
                        .bg(colors.fg)
                        .shadow_lg()
                        .child(div().size(px(8.0)).rounded_full().bg(if ok { hex(0x7ea2ff) } else { colors.danger }))
                        .child(div().max_w(px(420.0)).truncate().text_sm().text_color(colors.on_accent).child(text))
                        .child(pill("discard", "Discard", false).on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.discard(cx))))
                        .when(ok, |bar| bar.child(pill("apply", "Apply", true).on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.apply_draft(cx))))),
                )
                .into_any_element(),
        )
    }

    // ---------- drawing: browse ----------

    fn browse(&self, skills: bool, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let query = self.query.read(cx).text().trim().to_lowercase();
        let places = self.places();
        let of_kind: Vec<&Item> = self.items.iter().filter(|item| item.is_skill() == skills).collect();
        let rows: Vec<&Item> = of_kind
            .iter()
            .copied()
            .filter(|item| !skills || self.origin.as_ref().is_none_or(|origin| item.plugin == *origin))
            .filter(|item| query.is_empty() || format!("{} {} {}", item.name, item.plugin, item.description).to_lowercase().contains(&query))
            .collect();

        let title = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap_2()
                    .flex_1()
                    .child(div().text_2xl().font_weight(FontWeight::SEMIBOLD).child(if skills { "Skills" } else { "Plugins" }))
                    .child(div().text_lg().text_color(colors.fg_subtle).child(of_kind.len().to_string())),
            )
            .child(div().w(px(260.0)).child(SearchInput::new("search", &self.query)))
            .child(Button::new("add-source", "Add source").icon(IconName::Plus).on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.open_add_source(cx))));

        let filters = skills.then(|| {
            let mut origins: Vec<(Option<String>, usize)> = vec![(None, of_kind.len())];
            for plugin in self.items.iter().filter(|item| !item.is_skill()) {
                origins.push((Some(plugin.plugin.clone()), of_kind.iter().filter(|item| item.plugin == plugin.plugin).count()));
            }
            let mut segments = div().flex().items_center().gap_0p5().p_0p5().rounded_lg().bg(colors.sunken);
            for (ix, (origin, count)) in origins.into_iter().enumerate() {
                let on = self.origin == origin;
                let label = origin.clone().unwrap_or_else(|| "All".to_string());
                segments = segments.child(
                    div()
                        .id(("origin", ix))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .text_sm()
                        .cursor_pointer()
                        .when(on, |segment| segment.bg(colors.surface).shadow_xs().font_weight(FontWeight::MEDIUM))
                        .when(!on, |segment| segment.text_color(colors.fg_muted).hover(|style| style.text_color(colors.fg)))
                        .child(label)
                        .child(div().text_xs().text_color(colors.fg_subtle).child(count.to_string()))
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.origin = origin.clone();
                            cx.notify();
                        })),
                );
            }
            segments
        });

        let banner = self.target.as_ref().map(|root| {
            let place = places.iter().find(|place| place.root == *root).cloned().unwrap_or(Place { root: root.clone(), label: self.place_label(root) });
            div()
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(colors.info_subtle)
                .child(monogram(&place, 22.0, &colors))
                .child(div().flex_1().text_sm().child(format!("Adding to {}. Click a circle in its column to add a skill.", place.label)))
                .child(Button::new("done-adding", "Done").size(ControlSize::Sm).primary().on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    if let Some(root) = this.target.take() {
                        this.go(View::Place(root), cx);
                    }
                })))
        });

        let mut columns = div().flex().flex_none();
        for (ix, place) in places.iter().enumerate() {
            let target = self.target.as_ref() == Some(&place.root);
            columns = columns.child(
                div()
                    .id(("column", ix))
                    .flex()
                    .justify_center()
                    .w(px(SOCKET_WIDTH))
                    .py_1()
                    .rounded_t_md()
                    .when(target, |column| column.bg(colors.info_subtle))
                    .child(monogram(place, 20.0, &colors))
                    .tooltip(Tooltip::text(place.label.clone())),
            );
        }
        let shown = if rows.len() == of_kind.len() {
            format!("{} {}", rows.len(), if skills { "skills" } else { "plugins" })
        } else {
            format!("{} of {}", rows.len(), of_kind.len())
        };
        let grid_head = div()
            .flex()
            .items_end()
            .pl_4()
            .pr_3()
            .child(div().flex_1().text_xs().text_color(colors.fg_subtle).pb_1().child(shown))
            .child(columns);

        let mut list = div().flex().flex_col().rounded_xl().border_1().border_color(colors.border).bg(colors.surface).overflow_hidden();
        let last = rows.len().saturating_sub(1);
        for (ix, item) in rows.iter().enumerate() {
            list = list.child(self.row(ix, item, &places, ix == last, cx));
        }
        if rows.is_empty() {
            list = list.child(div().p_8().child(EmptyState::new("no-items", IconName::Sparkles, "Nothing matches").body("Clear the search, or pick All.")));
        }

        let head = div()
            .flex()
            .flex_col()
            .gap_4()
            .px_8()
            .pt_6()
            .pb_2()
            .child(title)
            .when_some(filters, |head, filters| head.child(div().flex().child(filters)))
            .children(banner);

        let main = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(head)
            .child(div().px_8().child(grid_head))
            .child(ScrollArea::new(("rows", skills as usize)).flex_1().min_h_0().child(div().px_8().pb_24().child(list)))
            .children(self.apply_bar(cx));
        let detail = self.selected.as_ref().and_then(|entry| self.items.iter().find(|item| item.entry == *entry)).map(|item| self.detail(item, cx));
        div().flex().size_full().min_h_0().child(main).children(detail).into_any_element()
    }

    fn row(&self, ix: usize, item: &Item, places: &[Place], last: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let selected = self.selected.as_ref() == Some(&item.entry);
        let entry = item.entry.clone();
        let mut sockets = div().flex().flex_none();
        for (jx, place) in places.iter().enumerate() {
            let has = self.has_in(&item.entry, &place.root);
            let target = self.target.as_ref() == Some(&place.root);
            let tip = match has {
                Has::Direct => format!("In {}. Click to remove.", place.label),
                Has::ViaPlugin => format!("In {} with the whole {} plugin.", place.label, item.plugin),
                Has::No => format!("Add to {}", place.label),
            };
            let (entry, root) = (item.entry.clone(), place.root.clone());
            sockets = sockets.child(
                div()
                    .id(("socket", ix * 100 + jx))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(SOCKET_WIDTH))
                    .h_full()
                    .group("socket")
                    .when(target, |cell| cell.bg(colors.info_subtle))
                    .when(has != Has::ViaPlugin, |cell| cell.cursor_pointer())
                    .child(socket(has, &colors))
                    .tooltip(Tooltip::text(tip))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        cx.stop_propagation();
                        if has != Has::ViaPlugin {
                            this.change(&entry, &root, &[], has == Has::No, cx);
                        }
                    })),
            );
        }
        div()
            .id(("row", ix))
            .flex()
            .min_h(px(60.0))
            .pl_4()
            .pr_3()
            .cursor_pointer()
            .when(!last, |row| row.border_b_1().border_color(colors.border))
            .when(selected, |row| row.bg(colors.info_subtle))
            .when(!selected, |row| row.hover(|style| style.bg(colors.bg)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .flex_1()
                    .min_w_0()
                    .py_2p5()
                    .pr_4()
                    .gap_0p5()
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap_2()
                            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(item.name.clone()))
                            .when(item.is_skill(), |line| line.child(div().text_xs().text_color(colors.fg_subtle).child(item.plugin.clone()))),
                    )
                    .child(div().text_sm().text_color(colors.fg_muted).truncate().child(item.description.clone())),
            )
            .child(sockets)
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.selected = if this.selected.as_ref() == Some(&entry) { None } else { Some(entry.clone()) };
                cx.notify();
            }))
    }

    fn detail(&self, item: &Item, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let mut places = div().flex().flex_col().rounded_xl().border_1().border_color(colors.border).bg(colors.surface);
        let all = self.places();
        let last = all.len().saturating_sub(1);
        for (ix, place) in all.into_iter().enumerate() {
            let mut agents = div().flex().flex_wrap().gap_1p5();
            for (jx, agent) in self.agents_of(&place.root).into_iter().enumerate() {
                let has = self.has(&item.entry, &place.root, agent);
                let (entry, root) = (item.entry.clone(), place.root.clone());
                let mut toggle = agent_toggle(("agent", ix * 1000 + jx), agent, has, &colors);
                if has != Has::ViaPlugin {
                    toggle = toggle.on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.change(&entry, &root, &[agent], has == Has::No, cx)));
                }
                agents = agents.child(toggle);
            }
            places = places.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .px_3()
                    .py_2p5()
                    .when(ix != last, |row| row.border_b_1().border_color(colors.border))
                    .child(div().flex().items_center().gap_2().child(monogram(&place, 18.0, &colors)).child(div().text_sm().font_weight(FontWeight::MEDIUM).child(place.label)))
                    .child(agents),
            );
        }
        let open = if item.is_skill() { item.dir.join("SKILL.md") } else { item.dir.clone() };
        let folder = item.dir.clone();
        let kind = if item.is_skill() { format!("Skill from {}", item.plugin) } else { "Plugin".to_string() };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(DETAIL_WIDTH))
            .h_full()
            .border_l_1()
            .border_color(colors.border)
            .bg(colors.bg)
            .child(
                ScrollArea::new("detail").flex_1().min_h_0().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_5()
                        .p_5()
                        .child(
                            div()
                                .flex()
                                .items_start()
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .gap_1()
                                        .child(div().text_xs().text_color(colors.fg_subtle).child(kind))
                                        .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child(item.name.clone())),
                                )
                                .child(IconButton::new("close-detail", IconName::X).size(ControlSize::Sm).tooltip("Close").on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                    this.selected = None;
                                    cx.notify();
                                }))),
                        )
                        .when(!item.description.is_empty(), |pane| pane.child(div().text_sm().line_height(px(21.0)).text_color(colors.fg_muted).child(item.description.clone())))
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .when(item.is_skill(), |row| row.child(Button::new("open-item", "Open SKILL.md").size(ControlSize::Sm).icon(IconName::FileText).on_click(move |_, _, _| reveal(&open))))
                                .child(Button::new("finder-item", "Show in Finder").size(ControlSize::Sm).variant(ButtonVariant::Ghost).icon(IconName::FolderOpen).on_click(move |_, _, _| reveal(&folder))),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("Where it is installed"))
                                .child(div().text_xs().text_color(colors.fg_subtle).child("Click an agent to turn it on or off in that place."))
                                .child(places),
                        ),
                ),
            )
            .into_any_element()
    }

    // ---------- drawing: a place ----------

    fn place(&self, root: &Path, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let mono = cx.theme().mono_family.clone();
        let Some(env) = self.env() else { return div().into_any_element() };
        let global = root == env.home;
        let agents = self.agents_of(root);
        let scope = self.scope(root);
        let place = Place { root: root.to_path_buf(), label: self.place_label(root) };
        let mut entries: Vec<Entry> = Vec::new();
        if let Some(scope) = scope {
            for agent in &agents {
                for entry in scope.entries_for(agent) {
                    if !entries.contains(&entry) {
                        entries.push(entry);
                    }
                }
            }
        }

        let entry_rows = |plugins: bool, this: &Self, cx: &mut Context<Self>| -> (usize, AnyElement) {
            let chosen: Vec<&Entry> = entries.iter().filter(|entry| matches!(entry, Entry::Plugin(_)) == plugins).collect();
            if chosen.is_empty() {
                let text = if plugins { "No plugins here yet." } else { "No single skills here yet." };
                return (0, div().px_4().py_3().text_sm().text_color(colors.fg_subtle).child(text).into_any_element());
            }
            let mut list = div().flex().flex_col();
            let last = chosen.len() - 1;
            for (ix, entry) in chosen.iter().enumerate() {
                let item = this.items.iter().find(|item| item.entry == **entry);
                let who: Vec<&'static Agent> = agents.iter().copied().filter(|agent| this.has(entry, root, agent) == Has::Direct).collect();
                let (entry, root) = ((*entry).clone(), root.to_path_buf());
                let (name, from) = match &entry {
                    Entry::Plugin(plugin) => (plugin.clone(), None),
                    Entry::Skill { plugin, skill } => (skill.clone(), Some(plugin.clone())),
                };
                let mut dots = div().flex().items_center().gap_1();
                for agent in who {
                    dots = dots.child(agent_tag(agent, &colors));
                }
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .py_2p5()
                        .when(ix != last, |row| row.border_b_1().border_color(colors.border))
                        .child(tile(if plugins { IconName::Package } else { IconName::Sparkles }, &colors))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .gap_2()
                                        .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(name))
                                        .when_some(from, |line, from| line.child(div().text_xs().text_color(colors.fg_subtle).child(from))),
                                )
                                .child(div().text_sm().text_color(colors.fg_muted).truncate().child(item.map(|item| item.description.clone()).unwrap_or_else(|| "Not in the library".to_string()))),
                        )
                        .child(dots)
                        .child(
                            IconButton::new(("take-out", ix + if plugins { 0 } else { 10_000 }), IconName::X)
                                .size(ControlSize::Sm)
                                .tooltip(format!("Remove from {}", this.place_label(&root)))
                                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.change(&entry, &root, &[], false, cx))),
                        ),
                );
            }
            (chosen.len(), list.into_any_element())
        };
        let (plugin_count, plugin_rows) = entry_rows(true, self, cx);
        let (skill_count, skill_rows) = entry_rows(false, self, cx);

        let mut by_name: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        for folder in installed::folders(&env, root, global, &[]) {
            for item in folder.items.into_iter().filter(|item| !item.owned) {
                by_name.entry(item.name).or_default().push(item.path);
            }
        }
        let foreign_count = by_name.len();
        let mut foreign: BTreeMap<String, Vec<(String, Vec<PathBuf>)>> = BTreeMap::new();
        for (name, paths) in by_name {
            let folder = paths.first().and_then(|path| path.parent()).map(|parent| self.show(root, parent)).unwrap_or_default();
            foreign.entry(folder).or_default().push((name, paths));
        }
        let mut foreign_rows = div().flex().flex_col();
        let mut n = 0;
        for (folder, names) in &foreign {
            foreign_rows = foreign_rows.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_1p5()
                    .bg(colors.bg)
                    .border_b_1()
                    .border_color(colors.border)
                    .child(Icon::new(IconName::Folder).size(IconSize::Xs).color(colors.fg_subtle))
                    .child(div().flex_1().text_xs().font_family(mono.clone()).text_color(colors.fg_muted).child(folder.clone()))
                    .child(div().text_xs().text_color(colors.fg_subtle).child(names.len().to_string())),
            );
            for (name, paths) in names {
                n += 1;
                let (name, paths, root) = (name.clone(), paths.clone(), root.to_path_buf());
                let also = (paths.len() > 1).then(|| format!("and {} more", paths.len() - 1));
                foreign_rows = foreign_rows.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .pl_9()
                        .pr_4()
                        .py_1()
                        .when(n != foreign_count, |row| row.border_b_1().border_color(colors.border))
                        .child(div().flex_1().text_sm().child(name.clone()))
                        .when_some(also, |row, also| row.child(div().text_xs().text_color(colors.fg_subtle).child(also)))
                        .child(IconButton::new(("foreign", n), IconName::Trash2).size(ControlSize::Sm).tooltip("Remove").on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.modal = Some(Modal::Foreign { root: root.clone(), name: name.clone(), paths: paths.clone() });
                            cx.notify();
                        }))),
                );
            }
        }

        let target = root.to_path_buf();
        let mut agent_tags = div().flex().flex_wrap().items_center().gap_1p5();
        for agent in &agents {
            agent_tags = agent_tags.child(agent_tag(agent, &colors));
        }
        let drift = self.drift.contains(root).then(|| {
            Callout::new(Severity::Warning)
                .title("The disk does not match manifest.toml")
                .child("Apply writes what manifest.toml gives this place. It leaves files that workflow-setup did not make.")
        });
        let path_line = if global { "Every project on this Mac".to_string() } else { apply::tilde(&env.home, root) };
        let body = div()
            .flex()
            .flex_col()
            .gap_7()
            .w_full()
            .max_w(px(CONTENT_WIDTH))
            .mx_auto()
            .px_8()
            .pt_8()
            .pb_24()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(monogram(&place, 52.0, &colors))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap_1p5()
                            .child(div().text_2xl().font_weight(FontWeight::SEMIBOLD).child(place.label.clone()))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(div().text_xs().text_color(colors.fg_muted).when(!global, |path| path.font_family(mono.clone())).child(path_line))
                                    .child(agent_tags),
                            ),
                    )
                    .when(self.drift.contains(root) && self.pending.is_none(), |head| {
                        head.child(Button::new("apply-place", "Apply").icon(IconName::RefreshCw).on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.apply_draft(cx))))
                    })
                    .child(Button::new("add-skills", "Add skills").primary().icon(IconName::Plus).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.go(View::Skills, cx);
                        this.target = Some(target.clone());
                    }))),
            )
            .children(drift)
            .when(scope.is_none() && !global, |body| body.child(div().text_sm().text_color(colors.fg_muted).child("manifest.toml has no entry for this project yet. Add skills to start one.")))
            .child(section("Plugins", plugin_count, None, panel(plugin_rows, &colors), &colors))
            .child(section("Single skills", skill_count, None, panel(skill_rows, &colors), &colors))
            .when(foreign_count > 0, |body| {
                body.child(section(
                    "Not from your library",
                    foreign_count,
                    Some("Other tools put these here. Remove unlinks a link, or moves a real folder to the Trash."),
                    panel(foreign_rows.into_any_element(), &colors),
                    &colors,
                ))
            });
        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .child(ScrollArea::new("place").flex_1().min_h_0().child(body))
            .children(self.apply_bar(cx))
            .into_any_element()
    }

    /// A path as the user reads it: from the place root, or from `~`.
    fn show(&self, root: &Path, path: &Path) -> String {
        match path.strip_prefix(root) {
            Ok(rel) if root != home() => rel.display().to_string(),
            _ => apply::tilde(&home(), path),
        }
    }

    // ---------- drawing: sources and settings ----------

    fn page(&self, id: &'static str, title: &str, lead: &str, action: Option<AnyElement>, body: AnyElement, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .child(
                ScrollArea::new(id).flex_1().min_h_0().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_6()
                        .w_full()
                        .max_w(px(CONTENT_WIDTH))
                        .mx_auto()
                        .px_8()
                        .pt_8()
                        .pb_24()
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap_4()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .gap_1()
                                        .child(div().text_2xl().font_weight(FontWeight::SEMIBOLD).child(title.to_string()))
                                        .child(div().max_w(px(560.0)).text_sm().text_color(colors.fg_muted).child(lead.to_string())),
                                )
                                .children(action),
                        )
                        .child(body),
                ),
            )
            .children(self.apply_bar(cx))
            .into_any_element()
    }

    fn sources_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let Some(library) = self.library() else { return div().into_any_element() };
        let own = self.items.iter().filter(|item| item.is_skill() && item.plugin == library::OWN).count();
        let own_dir = plugins::dir(&library, library::OWN);
        let mut list = div().flex().flex_col().child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .px_5()
                .py_4()
                .when(!self.sources.is_empty(), |row| row.border_b_1().border_color(colors.border))
                .child(source_tile(IconName::Pencil, true, &colors))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .gap_1()
                        .child(div().flex().items_baseline().gap_2().child(div().font_weight(FontWeight::SEMIBOLD).child(library::OWN)).child(div().text_xs().text_color(colors.success).child("Edited here")))
                        .child(div().text_sm().text_color(colors.fg_muted).child(format!("Your own {own} skills. You edit them in place, so there is nothing to fetch."))),
                )
                .child(Button::new("own-finder", "Show in Finder").size(ControlSize::Sm).variant(ButtonVariant::Ghost).icon(IconName::FolderOpen).on_click(move |_, _, _| reveal(&own_dir))),
        );
        let last = self.sources.len().saturating_sub(1);
        for (ix, source) in self.sources.iter().enumerate() {
            list = list.child(self.source_row(ix, source, ix == last, cx));
        }
        let action = Button::new("add-source-top", "Add source").primary().icon(IconName::Plus).on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.open_add_source(cx))).into_any_element();
        self.page(
            "sources",
            "Sources",
            "Where the skills in your library come from. Each outside source is a copy of a git repository, pinned to one commit until you accept an update.",
            Some(action),
            panel(list.into_any_element(), &colors),
            cx,
        )
    }

    fn source_row(&self, ix: usize, source: &Source, last: bool, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let mono = cx.theme().mono_family.clone();
        let count = self.items.iter().filter(|item| item.is_skill() && item.plugin == source.name).count();
        let check = self.checks.get(&source.name);
        let status = match check {
            Some(Check::Busy) => Some(("Checking…".to_string(), colors.fg_muted)),
            Some(Check::Ready(diff)) => Some((format!("Update ready, {} skills changed", diff.added.len() + diff.removed.len() + diff.changed.len()), signal())),
            Some(Check::Current) => Some(("Up to date".to_string(), colors.success)),
            Some(Check::Failed(_)) => Some(("Check failed".to_string(), colors.danger)),
            None => None,
        };
        let github = source.repository.starts_with("https://github.com/");
        let name = source.name.clone();
        let skills = if source.plugin {
            format!("Whole plugin, {count} skills")
        } else if source.skills.is_some() {
            format!("{count} picked skills")
        } else {
            format!("Every skill, {count}")
        };
        let busy = matches!(check, Some(Check::Busy));
        let fact = |label: &str, value: AnyElement| div().flex().items_center().gap_1p5().child(div().text_xs().text_color(colors.fg_subtle).child(label.to_string())).child(value);
        let value = |text: String, code: bool| {
            div().px_1p5().rounded_sm().bg(colors.sunken).text_xs().when(code, |chip| chip.font_family(mono.clone())).child(text).into_any_element()
        };
        let main_action = match check {
            Some(Check::Ready(_)) => Button::new(("review", ix), "Review update").primary().size(ControlSize::Sm).on_click(cx.listener({
                let name = name.clone();
                move |this, _: &ClickEvent, _, cx| {
                    this.modal = Some(Modal::Review(name.clone()));
                    cx.notify();
                }
            })),
            _ => Button::new(("check", ix), if busy { "Checking…" } else { "Check for updates" }).icon(IconName::RefreshCw).size(ControlSize::Sm).disabled(busy).on_click(cx.listener({
                let name = name.clone();
                move |this, _: &ClickEvent, _, cx| this.check_source(name.clone(), cx)
            })),
        };
        let editing = self.edit.as_ref().filter(|edit| edit.name == source.name);
        let failed = match check {
            Some(Check::Failed(error)) => Some(error.clone()),
            _ => None,
        };
        div()
            .flex()
            .flex_col()
            .gap_3()
            .px_5()
            .py_4()
            .when(!last, |row| row.border_b_1().border_color(colors.border))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_4()
                    .child(source_tile(if github { IconName::GitBranch } else { IconName::Folder }, false, &colors))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap_1p5()
                            .child(
                                div()
                                    .flex()
                                    .items_baseline()
                                    .gap_2()
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(source.name.clone()))
                                    .when_some(status, |line, (text, color)| line.child(div().text_xs().text_color(color).child(text))),
                            )
                            .child(div().text_sm().text_color(colors.fg_muted).truncate().child(short_repo(&source.repository)))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_center()
                                    .gap_4()
                                    .child(fact("Folder", value(if source.path.is_empty() { "/".to_string() } else { source.path.clone() }, true)))
                                    .child(fact("Branch", value(source.branch.clone().unwrap_or_else(|| "default".to_string()), true)))
                                    .child(fact("Pinned to", value(source.commit.as_deref().map(short).unwrap_or("not copied").to_string(), true)))
                                    .child(fact("Skills", value(skills, false))),
                            ),
                    )
                    .when(editing.is_none(), |row| {
                        row.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(main_action)
                                .child(IconButton::new(("edit", ix), IconName::Pencil).size(ControlSize::Sm).tooltip("Edit source").disabled(busy).on_click(cx.listener({
                                    let name = name.clone();
                                    move |this, _: &ClickEvent, _, cx| this.start_edit(name.clone(), cx)
                                })))
                                .child(IconButton::new(("remove-source", ix), IconName::Trash2).size(ControlSize::Sm).tooltip("Remove source").disabled(busy).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                    this.modal = Some(Modal::RemoveSource(name.clone()));
                                    cx.notify();
                                }))),
                        )
                    }),
            )
            .when_some(failed, |row, error| row.child(div().pl(px(52.0)).child(Callout::new(Severity::Danger).title("The check failed").child(clip(&error, 300)))))
            .when_some(editing.map(|edit| self.edit_form(source, edit, cx)), |row, form| row.child(div().pl(px(52.0)).child(form)))
            .into_any_element()
    }

    fn edit_form(&self, source: &Source, edit: &SourceEdit, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let skills: AnyElement = if source.plugin {
            hint("This source copies a whole plugin, so it takes every skill.", &colors).into_any_element()
        } else {
            match &edit.skills {
                None => hint("Fetching the list of skills…", &colors).into_any_element(),
                Some(Err(error)) => Callout::new(Severity::Danger).title("Cannot list the skills").child(clip(error, 300)).into_any_element(),
                Some(Ok(all)) => {
                    let mut grid = div().flex().flex_wrap().gap_1p5();
                    for (ix, skill) in all.iter().enumerate() {
                        let on = edit.picks.contains(skill);
                        let skill = skill.clone();
                        grid = grid.child(chip(("pick", ix), skill.clone(), on, &colors).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            if let Some(edit) = this.edit.as_mut() {
                                if !edit.picks.remove(&skill) {
                                    edit.picks.insert(skill.clone());
                                }
                            }
                            cx.notify();
                        })));
                    }
                    div().flex().flex_col().gap_2().child(field_label(format!("Skills to copy, {} of {}", edit.picks.len(), all.len()), &colors)).child(grid).into_any_element()
                }
            }
        };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded_lg()
            .bg(colors.bg)
            .border_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().flex_1().child(field("Repository", Input::new(&self.repo).into_any_element(), &colors)))
                    .child(div().w(px(180.0)).child(field("Branch", Input::new(&self.branch).into_any_element(), &colors))),
            )
            .child(skills)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().text_xs().text_color(colors.fg_subtle).child("Save writes sources.toml, fetches the source, and replaces its copy in the library."))
                    .child(Button::new("cancel-edit", "Cancel").size(ControlSize::Sm).variant(ButtonVariant::Ghost).disabled(edit.busy).on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.edit = None;
                        cx.notify();
                    })))
                    .child(
                        Button::new("save-edit", if edit.busy { "Saving…" } else { "Save and fetch" })
                            .primary()
                            .size(ControlSize::Sm)
                            .disabled(edit.busy || (!source.plugin && edit.skills.is_none()))
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.save_edit(cx))),
                    ),
            )
            .into_any_element()
    }

    fn settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors.clone();
        let mono = cx.theme().mono_family.clone();
        let library = self.library();
        let shown = library.as_ref().map(|library| apply::tilde(&home(), library)).unwrap_or_else(|| "Not chosen".to_string());
        let own = library.as_ref().map(|library| plugins::dir(library, library::OWN));
        let own_count = self.items.iter().filter(|item| item.is_skill() && item.plugin == library::OWN).count();
        let row = |title: &str, value: AnyElement, detail: Option<&str>, actions: Option<AnyElement>, last: bool| {
            div()
                .flex()
                .items_center()
                .gap_4()
                .px_5()
                .py_4()
                .when(!last, |row| row.border_b_1().border_color(colors.border))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(title.to_string()))
                        .child(value)
                        .when_some(detail, |column, detail| column.child(div().text_xs().text_color(colors.fg_subtle).child(detail.to_string()))),
                )
                .children(actions)
        };
        let path = |text: String| div().text_sm().text_color(colors.fg_muted).font_family(mono.clone()).truncate().child(text).into_any_element();

        let library_actions = div()
            .flex()
            .gap_1()
            .when_some(library.clone(), |row, library| row.child(IconButton::new("library-finder", IconName::FolderOpen).size(ControlSize::Sm).tooltip("Show in Finder").on_click(move |_, _, _| reveal(&library))))
            .child(Button::new("change-library", "Change…").size(ControlSize::Sm).on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                if this.library().is_some() {
                    this.modal = Some(Modal::Library);
                    cx.notify();
                } else {
                    this.pick_folder(true, window, cx);
                }
            })))
            .into_any_element();
        let own_actions = own.clone().map(|own| {
            let editor = own.clone();
            div()
                .flex()
                .gap_1()
                .child(IconButton::new("own-finder-2", IconName::FolderOpen).size(ControlSize::Sm).tooltip("Show in Finder").on_click(move |_, _, _| reveal(&own)))
                .child(Button::new("own-editor", "Open in editor").size(ControlSize::Sm).on_click(move |_, _, _| open_in_editor(&editor)))
                .into_any_element()
        });
        let mut default_tags = div().flex().flex_wrap().gap_1p5();
        for agent in agents::default() {
            default_tags = default_tags.child(agent_tag(agent, &colors));
        }
        let library_panel = panel(
            div()
                .flex()
                .flex_col()
                .child(row(
                    "Library folder",
                    path(shown),
                    Some("Holds your skills, sources.toml, and manifest.toml. Another folder brings its own record of what is installed."),
                    Some(library_actions),
                    own.is_none(),
                ))
                .when_some(own.clone(), |list, own| {
                    list.child(row(
                        &format!("Your skills, {own_count}"),
                        path(apply::tilde(&home(), &own)),
                        Some("Write a new skill as a folder with a SKILL.md. Press Cmd-R to see it here."),
                        own_actions,
                        true,
                    ))
                })
                .into_any_element(),
            &colors,
        );
        let agents_panel = panel(
            row(
                "New places get",
                default_tags.into_any_element(),
                Some("A place in manifest.toml can name its own agents with harnesses. Click an agent in a skill's panel to add it to one place."),
                Some(div().text_xs().text_color(colors.fg_subtle).child(format!("{} agents known", agents::all().len())).into_any_element()),
                true,
            )
            .into_any_element(),
            &colors,
        );
        self.page(
            "settings",
            "Settings",
            "Where the library lives, and which agents a new place gets.",
            None,
            div()
                .flex()
                .flex_col()
                .gap_6()
                .child(section("Library", 0, None, library_panel, &colors))
                .child(section("Agents", 0, None, agents_panel, &colors))
                .into_any_element(),
            cx,
        )
    }

    fn dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let colors = cx.theme().colors.clone();
        let this = cx.entity();
        let close = {
            let this = this.clone();
            move |_: &mut Window, cx: &mut App| {
                this.update(cx, |this, cx| {
                    if !matches!(this.modal, Some(Modal::AddSource { busy: true, .. })) {
                        this.modal = None;
                    }
                    cx.notify();
                })
            }
        };
        Some(match self.modal.as_ref()? {
            Modal::AddSource { fetched, picks, busy, error } => {
                let mut body = div().flex().flex_col().gap_4().w(px(480.0));
                body = match fetched {
                    None => body
                        .child(field("Repository", Input::new(&self.spec).into_any_element(), &colors))
                        .child(hint("Examples: mattpocock/skills, https://github.com/owner/repo.git, ~/code/my-skills", &colors)),
                    Some(found) => {
                        let mut grid = div().flex().flex_wrap().gap_1p5();
                        for (ix, skill) in found.skills.iter().enumerate() {
                            let skill = skill.clone();
                            grid = grid.child(chip(("new-pick", ix), skill.clone(), picks.contains(&skill), &colors).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                if let Some(Modal::AddSource { picks, .. }) = this.modal.as_mut() {
                                    if !picks.remove(&skill) {
                                        picks.insert(skill.clone());
                                    }
                                }
                                cx.notify();
                            })));
                        }
                        body.child(hint(&format!("Fetched {}. Pick the skills to copy into your library.", short_repo(&found.source.repository)), &colors))
                            .child(field("Name in your library", Input::new(&self.new_name).into_any_element(), &colors))
                            .child(field_label(format!("Skills, {} of {}", picks.len(), found.skills.len()), &colors))
                            .child(ScrollArea::new("new-picks").max_h(px(260.0)).child(grid))
                    }
                };
                if let Some(error) = error {
                    body = body.child(Callout::new(Severity::Danger).title("Not added").child(clip(error, 400)));
                }
                let step_two = fetched.is_some();
                let (busy, count) = (*busy, picks.len());
                let label = match (step_two, busy) {
                    (false, true) => "Fetching…".to_string(),
                    (false, false) => "Fetch".to_string(),
                    (true, true) => "Adding…".to_string(),
                    (true, false) => format!("Add {count} skill{}", if count == 1 { "" } else { "s" }),
                };
                Dialog::new("add-source", "Add a source", close)
                    .detail("A GitHub repository, a git URL, or a folder on this Mac.")
                    .child(body)
                    .action(|close| Button::new("cancel-add", "Cancel").variant(ButtonVariant::Ghost).on_click(move |_, window, cx| close(window, cx)))
                    .action(move |_| {
                        Button::new("go-add", label).primary().disabled(busy || (step_two && count == 0)).on_click(move |_, _, cx| {
                            this.update(cx, |this, cx| if step_two { this.add_new(cx) } else { this.fetch_new(cx) });
                        })
                    })
                    .into_any_element()
            }
            Modal::Review(name) => {
                let Some(Check::Ready(diff)) = self.checks.get(name) else { return None };
                let source = self.sources.iter().find(|source| source.name == *name)?;
                let mono = cx.theme().mono_family.clone();
                let mut body = div().flex().flex_col().gap_4().w(px(540.0)).child(hint(
                    &format!("From {} to {}. Read the changes before they enter your library.", source.commit.as_deref().map(short).unwrap_or("no pin"), short(&diff.to)),
                    &colors,
                ));
                for (label, names, color) in [("Added", &diff.added, colors.success), ("Changed", &diff.changed, signal()), ("Removed", &diff.removed, colors.danger)] {
                    if !names.is_empty() {
                        let mut tags = div().flex().flex_wrap().gap_1p5();
                        for name in names.iter() {
                            tags = tags.child(div().px_2().py_0p5().rounded_md().bg(color.opacity(0.1)).text_xs().text_color(color).child(name.clone()));
                        }
                        body = body.child(div().flex().flex_col().gap_1p5().child(field_label(format!("{label}, {}", names.len()), &colors)).child(tags));
                    }
                }
                if !diff.edited_here.is_empty() {
                    body = body.child(Callout::new(Severity::Warning).title("Edited in your library").child(format!("Accept overwrites your edits to {}.", diff.edited_here.join(", "))));
                }
                if !diff.log.is_empty() {
                    body = body.child(field_label("Commits".to_string(), &colors)).child(
                        ScrollArea::new("log")
                            .max_h(px(180.0))
                            .child(div().p_3().rounded_lg().bg(colors.sunken).text_xs().text_color(colors.fg_muted).font_family(mono).child(diff.log.clone())),
                    );
                }
                let name = name.clone();
                Dialog::new("review", format!("Update {name}"), close)
                    .child(body)
                    .action(|close| Button::new("not-now", "Not now").variant(ButtonVariant::Ghost).on_click(move |_, window, cx| close(window, cx)))
                    .action(move |_| {
                        Button::new("accept", "Accept update").primary().on_click(move |_, _, cx| {
                            this.update(cx, |this, cx| this.accept_source(name.clone(), cx));
                        })
                    })
                    .into_any_element()
            }
            Modal::RemoveSource(name) => {
                let count = self.items.iter().filter(|item| item.is_skill() && item.plugin == *name).count();
                let used: Vec<String> = self
                    .places()
                    .into_iter()
                    .filter(|place| self.scope(&place.root).is_some_and(|scope| scope.agents().iter().any(|agent| scope.entries_for(agent).iter().any(|entry| entry.plugin() == name))))
                    .map(|place| place.label)
                    .collect();
                let mut message = format!("Its {count} skills leave your library, and its entry leaves sources.toml. The copy goes to the Trash.");
                if !used.is_empty() {
                    message.push_str(&format!(" It also comes out of {}, now.", used.join(", ")));
                }
                let name = name.clone();
                ConfirmDialog::new("remove-source", format!("Remove {name}?"), message, close)
                    .confirm("Remove source")
                    .destructive()
                    .on_confirm(move |_, cx| this.update(cx, |this, cx| this.remove_source(name.clone(), cx)))
                    .into_any_element()
            }
            Modal::Foreign { root, name, paths } => {
                let shown: Vec<String> = paths.iter().map(|path| self.show(root, path)).collect();
                let paths = paths.clone();
                let name = name.clone();
                ConfirmDialog::new(
                    "remove-foreign",
                    format!("Remove {name}?"),
                    format!("workflow-setup did not make {}. A link is unlinked. A real folder moves to the Trash.", shown.join(", ")),
                    close,
                )
                .confirm("Remove")
                .destructive()
                .on_confirm(move |_, cx| {
                    this.update(cx, |this, cx| {
                        this.modal = None;
                        let result = paths.iter().try_for_each(|path| ops::remove(path, &home()).map(drop)).map(|()| format!("Removed {name}"));
                        this.finish(result, cx);
                    })
                })
                .into_any_element()
            }
            Modal::Library => ConfirmDialog::new(
                "library",
                "Change the library folder?",
                "The app reads skills, sources, and manifest.toml from the folder you choose. Places then show what that manifest.toml records. Links already on disk stay.",
                close,
            )
            .confirm("Choose folder…")
            .on_confirm(move |window, cx| {
                this.update(cx, |this, cx| {
                    this.modal = None;
                    this.pick_folder(true, window, cx);
                })
            })
            .into_any_element(),
        })
    }
}

// ---------- drawing: parts ----------

fn nav_icon(icon: IconName) -> AnyElement {
    div().flex().items_center().justify_center().size(px(20.0)).child(Icon::new(icon).size(IconSize::Sm)).into_any_element()
}

fn group_label(text: &str, colors: &Palette) -> impl IntoElement {
    div().px_2().pb_0p5().text_xs().font_weight(FontWeight::MEDIUM).text_color(colors.fg_subtle).child(text.to_string())
}

/// A place's tile: a globe for Global, else its first letter on its own color.
fn monogram(place: &Place, size: f32, colors: &Palette) -> Div {
    let tile = div().flex().flex_none().items_center().justify_center().size(px(size)).rounded(px(size * 0.28));
    if place.global() {
        let icon = if size > 30.0 { IconSize::Lg } else { IconSize::Xs };
        return tile.bg(colors.fg).child(Icon::new(IconName::Globe).size(icon).color(colors.on_accent));
    }
    let (bg, fg) = tint(&place.label);
    let letter = place.label.chars().find(|c| c.is_alphanumeric()).map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    tile.bg(bg).text_color(fg).font_weight(FontWeight::BOLD).text_size(px(size * 0.5)).child(letter)
}

/// One cell of the coverage grid: filled when installed, a ring with a dot
/// when its plugin brings it, an empty ring when absent.
fn socket(has: Has, colors: &Palette) -> impl IntoElement {
    let ring = div().flex().items_center().justify_center().size(px(16.0)).rounded_full();
    match has {
        Has::Direct => ring.bg(signal()).child(Icon::new(IconName::Check).size(IconSize::Xs).color(colors.on_accent)),
        Has::ViaPlugin => ring.border_2().border_color(signal().opacity(0.35)).child(div().size(px(6.0)).rounded_full().bg(signal().opacity(0.55))),
        Has::No => {
            let hover = colors.info_subtle;
            ring.border(px(1.5)).border_color(colors.border_strong).group_hover("socket", move |style| style.border_color(signal()).bg(hover))
        }
    }
}

/// An agent with its color, read-only.
fn agent_tag(agent: &Agent, colors: &Palette) -> impl IntoElement {
    let color = agent_color(agent);
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .px_2()
        .py_0p5()
        .rounded_full()
        .bg(color.opacity(0.09))
        .text_xs()
        .text_color(colors.fg)
        .child(div().size(px(7.0)).rounded_full().bg(color))
        .child(agent.display.clone())
}

/// An agent that turns on or off for one place.
fn agent_toggle(id: impl Into<gpui::ElementId>, agent: &Agent, has: Has, colors: &Palette) -> Stateful<Div> {
    let color = agent_color(agent);
    let hover = colors.hover;
    div()
        .id(id.into())
        .flex()
        .items_center()
        .gap_1p5()
        .px_2p5()
        .py_1()
        .rounded_md()
        .border_1()
        .text_xs()
        .child(
            div()
                .size(px(8.0))
                .rounded_full()
                .when(has == Has::No, |dot| dot.border(px(1.5)).border_color(color.opacity(0.6)))
                .when(has != Has::No, |dot| dot.bg(color)),
        )
        .child(agent.display.clone())
        .when(has == Has::Direct, |toggle| toggle.bg(color.opacity(0.1)).border_color(color.opacity(0.45)).text_color(colors.fg).font_weight(FontWeight::MEDIUM).cursor_pointer())
        .when(has == Has::ViaPlugin, |toggle| {
            toggle.bg(color.opacity(0.05)).border_color(gpui::transparent_black()).text_color(colors.fg_muted).child(div().text_color(colors.fg_subtle).child("via plugin"))
        })
        .when(has == Has::No, |toggle| toggle.border_color(colors.border).text_color(colors.fg_muted).cursor_pointer().hover(move |style| style.bg(hover)))
}

/// A small square tile with an icon, for a row in a place.
fn tile(icon: IconName, colors: &Palette) -> impl IntoElement {
    div().flex().flex_none().items_center().justify_center().size(px(30.0)).rounded_lg().bg(colors.sunken).child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_muted))
}

fn source_tile(icon: IconName, own: bool, colors: &Palette) -> impl IntoElement {
    let tile = div().flex().flex_none().items_center().justify_center().size(px(36.0)).rounded_lg();
    if own {
        tile.bg(colors.fg).child(Icon::new(icon).size(IconSize::Sm).color(colors.on_accent))
    } else {
        tile.bg(colors.sunken).child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_muted))
    }
}

/// A white box with a hairline, for a list of rows.
fn panel(body: AnyElement, colors: &Palette) -> AnyElement {
    div().rounded_xl().border_1().border_color(colors.border).bg(colors.surface).overflow_hidden().child(body).into_any_element()
}

fn section(title: &str, count: usize, lead: Option<&str>, body: AnyElement, colors: &Palette) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_baseline()
                .gap_2()
                .child(div().text_base().font_weight(FontWeight::SEMIBOLD).child(title.to_string()))
                .when(count > 0, |line| line.child(div().text_sm().text_color(colors.fg_subtle).child(count.to_string()))),
        )
        .when_some(lead, |column, lead| column.child(div().text_sm().text_color(colors.fg_muted).child(lead.to_string())))
        .child(body)
}

fn hint(text: &str, colors: &Palette) -> impl IntoElement {
    div().text_sm().text_color(colors.fg_muted).child(text.to_string())
}

fn field_label(text: String, colors: &Palette) -> impl IntoElement {
    div().text_xs().font_weight(FontWeight::MEDIUM).text_color(colors.fg_muted).child(text)
}

fn field(label: &str, input: AnyElement, colors: &Palette) -> impl IntoElement {
    div().flex().flex_col().gap_1p5().child(field_label(label.to_string(), colors)).child(input)
}

/// A toggle that reads as on or off.
fn chip(id: impl Into<gpui::ElementId>, label: String, on: bool, colors: &Palette) -> Stateful<Div> {
    let (hover, border) = (colors.hover, colors.border);
    div()
        .id(id.into())
        .flex()
        .items_center()
        .gap_1()
        .px_2p5()
        .py_1()
        .rounded_md()
        .text_xs()
        .cursor_pointer()
        .border_1()
        .when(on, |chip| {
            chip.bg(signal().opacity(0.1))
                .border_color(signal().opacity(0.5))
                .text_color(signal())
                .font_weight(FontWeight::MEDIUM)
                .child(Icon::new(IconName::Check).size(IconSize::Xs).color(signal()))
        })
        .when(!on, |chip| chip.border_color(border).text_color(colors.fg_muted).hover(move |style| style.bg(hover)))
        .child(label)
}

fn done(applied: &[Applied]) -> String {
    let (written, removed, left) = applied.iter().fold((0, 0, 0), |sum, one| (sum.0 + one.written, sum.1 + one.removed, sum.2 + one.left));
    let mut text = format!("{written} written, {removed} removed");
    if left > 0 {
        text.push_str(&format!(", {left} left because workflow-setup does not own them"));
    }
    text
}

fn reveal(path: &Path) {
    Command::new("open").arg(path).spawn().ok();
}

fn open_in_editor(path: &Path) {
    let opened = Command::new("open").args(["-a", "Zed"]).arg(path).status().is_ok_and(|status| status.success());
    if !opened {
        reveal(path);
    }
}

fn name_of(root: &Path) -> String {
    root.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| root.display().to_string())
}

fn short_repo(repository: &str) -> String {
    match repository.strip_prefix("https://github.com/") {
        Some(rest) => rest.trim_end_matches(".git").to_string(),
        None => apply::tilde(&home(), Path::new(repository)),
    }
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(7)]
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{}…", cut.trim_end())
}

impl Render for Manager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let main = if self.library().is_none() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .gap_4()
                .child(EmptyState::new("no-library", IconName::FolderOpen, "Choose a library").body("The library is the folder with plugins/, sources.toml, and manifest.toml."))
                .child(Button::new("pick-library", "Choose folder…").primary().on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.pick_folder(true, window, cx))))
                .into_any_element()
        } else {
            match self.view.clone() {
                View::Skills => self.browse(true, cx),
                View::Plugins => self.browse(false, cx),
                View::Sources => self.sources_view(cx),
                View::Settings => self.settings(cx),
                View::Place(root) => self.place(&root, cx),
            }
        };
        div()
            .size_full()
            .on_action(cx.listener(|this, _: &OpenProject, window, cx| this.pick_folder(false, window, cx)))
            .on_action(cx.listener(|this, _: &ChooseLibrary, window, cx| this.pick_folder(true, window, cx)))
            .on_action(cx.listener(|this, _: &Rescan, _, cx| {
                this.reload();
                cx.notify();
            }))
            .child(
                FocusScope::new(&self.focus)
                    .root()
                    .size_full()
                    .flex()
                    .bg(colors.bg)
                    .text_color(colors.fg)
                    .child(self.sidebar(cx))
                    .child(div().flex().flex_col().flex_1().min_w_0().h_full().child(main))
                    .children(self.dialog(cx))
                    .child(ToastViewport::new("toasts", &self.toaster)),
            )
    }
}
