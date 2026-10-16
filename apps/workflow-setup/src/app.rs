//! The window: tabs, a list of items, and the selected item with its editor.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    data_display::{Badge, DescriptionList, Tone},
    editor::{CodeEditor, EditorEvent},
    feedback::{Callout, EmptyState, Toast, ToastViewport, Toaster},
    forms::{Choice, Input, InputEvent, SearchInput, Switch, TextInput},
    layout::ScrollArea,
    lists::{ListItem, SelectableList},
    menus::{DropdownMenu, Menu, MenuItem},
    navigation::Tabs,
    overlays::{ConfirmDialog, Dialog},
    primitives::{FocusScope, IconName, Severity},
    tables::Table,
    theme::{ActiveTheme, ControlSize},
    typography::{Caption, Heading, Overline, Paragraph},
};
use anyhow::Context as _;
use gpui::{
    AnyElement, App, AppContext, ClickEvent, Context, Entity, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, PathPromptOptions, Render, SharedString, Styled, Subscription,
    Window, actions, div, prelude::FluentBuilder, px,
};

use crate::{
    apply::{self, Env, Plan},
    config::Config,
    library, manifest,
    ops::{self, Removed},
    scan::{self, Agent, Instruction, MCP_TARGETS, McpServer, Project, SKILL_DIRS, SKILL_TARGETS, Skill, Status},
};

actions!(workflow_setup, [Save, OpenProject, ChooseLibrary, Rescan, Quit]);

const LIST_WIDTH: f32 = 340.0;
/// A skill's install table and notes scroll past this height, so the editor keeps its room.
const SUMMARY_MAX_HEIGHT: f32 = 320.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Skills,
    Mcp,
    Agents,
    Instructions,
    /// The drift between the manifest and the disk, for every scope.
    Drift,
}

impl Tab {
    const ALL: [Tab; 5] = [Tab::Skills, Tab::Mcp, Tab::Agents, Tab::Instructions, Tab::Drift];

    fn key(self) -> &'static str {
        match self {
            Tab::Skills => "skills",
            Tab::Mcp => "mcp",
            Tab::Agents => "agents",
            Tab::Instructions => "instructions",
            Tab::Drift => "drift",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Tab::Skills => "Skills",
            Tab::Mcp => "MCP servers",
            Tab::Agents => "Agents",
            Tab::Instructions => "Instructions",
            Tab::Drift => "Drift",
        }
    }

    fn from_key(key: &str) -> Tab {
        Tab::ALL
            .into_iter()
            .find(|tab| tab.key() == key)
            .unwrap_or(Tab::Skills)
    }
}

/// A step that drops unsaved edits, so it asks first when there are some.
#[derive(Clone)]
enum Intent {
    Tab(Tab),
    Item(String),
    Open(PathBuf),
    Rescan,
}

#[derive(Clone)]
enum Removal {
    /// A skill folder or an agent file in the project.
    Path { title: String, path: PathBuf },
    Mcp { name: String, path: PathBuf, key: &'static str, file: String },
}

enum Prompt {
    Discard(Intent),
    Remove(Removal),
    NewServer,
    /// Make the disk match the manifest in the scope at this root.
    Apply { root: PathBuf, adopt: bool },
}

/// The file the editor holds, and its text as last read or saved.
struct Editing {
    path: PathBuf,
    saved: String,
}

struct Row {
    key: String,
    title: String,
    description: String,
    detail: String,
    warning: Option<&'static str>,
}

pub struct Manager {
    focus: FocusHandle,
    config: Config,
    project: Option<Project>,
    tab: Tab,
    /// The chosen item of each tab, by `Tab::ALL` order.
    selected: [Option<String>; 5],
    /// The drift of every scope, or why the engine cannot give it.
    plans: Result<Vec<Plan>, String>,
    /// The plugins the manifest assigns to the open project.
    assigned: BTreeSet<String>,
    filter: Entity<TextInput>,
    editor: Entity<CodeEditor>,
    editing: Option<Editing>,
    toaster: Entity<Toaster>,
    prompt: Option<Prompt>,
    server_name: Entity<TextInput>,
    server_command: Entity<TextInput>,
    _subscriptions: Vec<Subscription>,
}

impl Manager {
    pub fn new(project: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let filter = cx.new(|cx| TextInput::new(window, cx).placeholder("Filter"));
        let editor = cx.new(|cx| CodeEditor::new("", window, cx));
        let server_name = cx.new(|cx| TextInput::new(window, cx).placeholder("Name, such as github"));
        let server_command =
            cx.new(|cx| TextInput::new(window, cx).placeholder("Command or URL, such as npx -y some-mcp"));
        let subscriptions = vec![
            cx.subscribe(&filter, |_, _, event: &InputEvent, cx| {
                if *event == InputEvent::Changed {
                    cx.notify();
                }
            }),
            cx.subscribe(&editor, |_, _, event: &EditorEvent, cx| {
                if *event == EditorEvent::Changed {
                    cx.notify();
                }
            }),
        ];
        let config = Config::load();
        let first = project.or_else(|| config.recent.first().cloned());
        let mut manager = Self {
            focus,
            config,
            project: None,
            tab: Tab::Skills,
            selected: Default::default(),
            plans: Ok(Vec::new()),
            assigned: BTreeSet::new(),
            filter,
            editor,
            editing: None,
            toaster: cx.new(|_| Toaster::default()),
            prompt: None,
            server_name,
            server_command,
            _subscriptions: subscriptions,
        };
        if let Some(root) = first.filter(|root| root.is_dir()) {
            manager.open(root, window, cx);
        }
        manager
    }

    // ---------- state ----------

    fn open(&mut self, root: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let root = root.canonicalize().unwrap_or(root);
        // The first project that has a skills/ folder becomes the library.
        if self.config.library.is_none() && library::is_library(&root) {
            self.config.library = Some(root.clone());
        }
        if root != scan::home() {
            self.config.opened(root.clone());
        }
        self.save_config(cx);
        let project = scan::scan(&root, self.config.library.as_deref());
        window.set_window_title(&format!("{} — workflow-setup", project.name));
        self.selected = Default::default();
        self.project = Some(project);
        self.read_engine();
        self.settle(cx);
    }

    fn rescan(&mut self, cx: &mut Context<Self>) {
        if let Some(root) = self.project.as_ref().map(|project| project.root.clone()) {
            self.project = Some(scan::scan(&root, self.config.library.as_deref()));
        }
        self.read_engine();
        self.settle(cx);
    }

    fn env(&self) -> Option<Env> {
        self.config.library.clone().map(Env::new)
    }

    /// Reads the manifest's view: the drift, and what the open project is assigned.
    fn read_engine(&mut self) {
        let Some(env) = self.env() else {
            self.plans = Err("Choose a library to see the drift.".to_string());
            self.assigned.clear();
            return;
        };
        self.plans = apply::plan_all(&env).map_err(|error| format!("{error:#}"));
        self.assigned = match &self.project {
            Some(project) => apply::assigned(&env, &project.root).unwrap_or_default(),
            None => BTreeSet::new(),
        };
    }

    fn plan(&self) -> Option<&Plan> {
        let key = self.chosen(Tab::Drift)?;
        self.plans.as_ref().ok()?.iter().find(|plan| plan.scope.root.to_string_lossy() == key)
    }

    /// Assigns a library plugin to the open project in the manifest, or takes
    /// it out, then makes that project match.
    fn assign(&mut self, item: String, on: bool, cx: &mut Context<Self>) {
        let (Some(env), Some(root)) = (self.env(), self.project.as_ref().map(|project| project.root.clone())) else { return };
        let entry = manifest::Entry::Plugin(item.clone());
        let result = manifest::edit(&manifest::read(&env.library), &env.home, &root, &[entry], &[], on)
            .and_then(|text| manifest::write(&env.library, &text))
            .and_then(|()| self.apply_scope(&env, &root, false))
            .map(|done| format!("{} {item}. {done}", if on { "Assigned" } else { "Took out" }));
        self.finish(result, cx);
    }

    fn apply_scope(&self, env: &Env, root: &Path, adopt: bool) -> anyhow::Result<String> {
        let mut plans = apply::plan_all(env)?;
        plans.retain(|plan| plan.scope.root == root);
        let (results, _) = apply::apply(env, &plans, adopt)?;
        let (written, removed, left) = results.iter().fold((0, 0, 0), |sum, applied| (sum.0 + applied.written, sum.1 + applied.removed, sum.2 + applied.left));
        let mut done = format!("{written} written, {removed} removed");
        if left > 0 {
            done.push_str(&format!(", {left} left because workflow-setup does not own them"));
        }
        Ok(done)
    }

    /// After the project or the selection changed: a tab whose item is gone
    /// takes its first, and the editor takes the chosen item's file.
    fn settle(&mut self, cx: &mut Context<Self>) {
        for tab in Tab::ALL {
            let rows = self.rows(tab);
            let slot = &mut self.selected[tab as usize];
            if !rows.iter().any(|row| Some(&row.key) == slot.as_ref()) {
                *slot = rows.first().map(|row| row.key.clone());
            }
        }
        let document = self.document();
        match document {
            Some((path, text)) => {
                self.editor.update(cx, |editor, cx| editor.set_text(text.clone(), cx));
                self.editing = Some(Editing { path, saved: text });
            }
            None => self.editing = None,
        }
        cx.notify();
    }

    fn save_config(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.config.save() {
            self.toast(Severity::Danger, "Settings were not saved", format!("{error:#}"), cx);
        }
    }

    fn chosen(&self, tab: Tab) -> Option<&str> {
        self.selected[tab as usize].as_deref()
    }

    fn skill(&self) -> Option<&Skill> {
        let key = self.chosen(Tab::Skills)?;
        self.project.as_ref()?.skills.iter().find(|skill| skill.name == key)
    }

    fn server(&self) -> Option<&McpServer> {
        let key = self.chosen(Tab::Mcp)?;
        self.project.as_ref()?.mcp.iter().find(|server| server.name == key)
    }

    fn agent(&self) -> Option<&Agent> {
        let key = self.chosen(Tab::Agents)?;
        self.project.as_ref()?.agents.iter().find(|agent| agent.rel == key)
    }

    fn instruction(&self) -> Option<&Instruction> {
        let key = self.chosen(Tab::Instructions)?;
        self.project.as_ref()?.instructions.iter().find(|file| file.rel == key)
    }

    /// The file behind the chosen item, and its text.
    fn document(&self) -> Option<(PathBuf, String)> {
        match self.tab {
            Tab::Skills => self.skill().and_then(|skill| Some((skill.file.clone()?, skill.content.clone()))),
            Tab::Mcp | Tab::Drift => None,
            Tab::Agents => self.agent().map(|agent| (agent.file.clone(), agent.content.clone())),
            Tab::Instructions => self.instruction().map(|file| (file.abs.clone(), file.content.clone())),
        }
    }

    fn dirty(&self, cx: &App) -> bool {
        self.editing
            .as_ref()
            .is_some_and(|editing| self.editor.read(cx).text() != editing.saved)
    }

    fn rows(&self, tab: Tab) -> Vec<Row> {
        let Some(project) = &self.project else { return Vec::new() };
        match tab {
            Tab::Drift => self
                .plans
                .iter()
                .flatten()
                .map(|plan| Row {
                    key: plan.scope.root.to_string_lossy().into_owned(),
                    title: if plan.scope.global { "Global".to_string() } else { apply::tilde(&scan::home(), &plan.scope.root) },
                    description: if plan.in_sync() { "In sync with the manifest".to_string() } else { format!("{} to change", plan.changes.len()) },
                    detail: plan.scope.agents.iter().map(|agent| agent.key()).collect::<Vec<_>>().join(" · "),
                    warning: (!plan.in_sync()).then_some("drift"),
                })
                .collect(),
            Tab::Skills => project
                .skills
                .iter()
                .map(|skill| {
                    let mut tools: Vec<&str> = skill.installs.iter().map(|i| i.tool.label()).collect();
                    tools.dedup();
                    Row {
                        key: skill.name.clone(),
                        title: skill.name.clone(),
                        description: skill.description.clone(),
                        detail: tools.join(" · "),
                        warning: if skill.installs.iter().any(|install| install.broken) {
                            Some("broken link")
                        } else if skill.differs {
                            Some("copies differ")
                        } else if skill.installs.is_empty() {
                            Some("not installed")
                        } else {
                            None
                        },
                    }
                })
                .collect(),
            Tab::Mcp => project
                .mcp
                .iter()
                .map(|server| Row {
                    key: server.name.clone(),
                    title: server.name.clone(),
                    description: server
                        .defs
                        .first()
                        .map(|def| if def.command.is_empty() { def.url.clone() } else { def.command.clone() })
                        .unwrap_or_default(),
                    detail: server
                        .defs
                        .iter()
                        .map(|def| match def.scope {
                            "project" => def.tool.label().to_string(),
                            scope => format!("{} ({scope})", def.tool.label()),
                        })
                        .collect::<Vec<_>>()
                        .join(" · "),
                    warning: server
                        .defs
                        .iter()
                        .find(|def| def.status != Status::Enabled)
                        .map(|def| def.status.label()),
                })
                .collect(),
            Tab::Agents => project
                .agents
                .iter()
                .map(|agent| Row {
                    key: agent.rel.clone(),
                    title: agent.name.clone(),
                    description: agent.description.clone(),
                    detail: [agent.tool.label(), agent.model.as_str()]
                        .into_iter()
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · "),
                    warning: (!agent.installed).then_some("not installed"),
                })
                .collect(),
            Tab::Instructions => project
                .instructions
                .iter()
                .map(|file| Row {
                    key: file.rel.clone(),
                    title: file.rel.clone(),
                    description: format!("{} lines, about {} tokens", file.content.lines().count(), file.content.len() / 4),
                    detail: format!("{} · {}", file.tool, file.scope),
                    warning: None,
                })
                .collect(),
        }
    }

    /// A path as the user reads it: from the project root, or from `~`.
    fn show(&self, path: &Path) -> String {
        let root = self.project.as_ref().map(|project| project.root.as_path());
        if let Some(rel) = root.and_then(|root| path.strip_prefix(root).ok()) {
            return rel.to_string_lossy().into_owned();
        }
        match path.strip_prefix(scan::home()) {
            Ok(rel) => format!("~/{}", rel.to_string_lossy()),
            Err(_) => path.to_string_lossy().into_owned(),
        }
    }

    // ---------- steps ----------

    fn request(&mut self, intent: Intent, window: &mut Window, cx: &mut Context<Self>) {
        if self.dirty(cx) {
            self.prompt = Some(Prompt::Discard(intent));
            cx.notify();
        } else {
            self.apply(intent, window, cx);
        }
    }

    fn apply(&mut self, intent: Intent, window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            Intent::Tab(tab) => {
                self.tab = tab;
                self.settle(cx);
            }
            Intent::Item(key) => {
                self.selected[self.tab as usize] = Some(key);
                self.settle(cx);
            }
            Intent::Open(root) => self.open(root, window, cx),
            Intent::Rescan => self.rescan(cx),
        }
    }

    fn toast(&mut self, severity: Severity, title: impl Into<SharedString>, body: String, cx: &mut Context<Self>) {
        self.toaster.update(cx, |toaster, cx| {
            let toast = Toast::new(title).severity(severity);
            toaster.push(if body.is_empty() { toast } else { toast.body(body) }, cx);
        });
    }

    /// Tells the user how a change went, then reads the project again.
    fn finish(&mut self, result: anyhow::Result<String>, cx: &mut Context<Self>) {
        match result {
            Ok(done) => self.toast(Severity::Success, done, String::new(), cx),
            Err(error) => self.toast(Severity::Danger, "Not done", format!("{error:#}"), cx),
        }
        self.rescan(cx);
    }

    fn save(&mut self, _: &Save, _: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = &self.editing else { return };
        if !self.dirty(cx) {
            return;
        }
        let text = self.editor.read(cx).text().to_string();
        let result = ops::save(&editing.path, &editing.saved, &text).map(|()| format!("Saved {}", self.show(&editing.path)));
        match result {
            Ok(done) => {
                self.toast(Severity::Success, done, String::new(), cx);
                self.rescan(cx);
            }
            // Keep the edits in the editor: a rescan would replace them.
            Err(error) => self.toast(Severity::Danger, "Not saved", format!("{error:#}"), cx),
        }
    }

    fn pick_folder(&mut self, library: bool, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(if library { "Use as library" } else { "Open project" }.into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            let Some(path) = paths.pop() else { return };
            this.update_in(cx, |this, window, cx| {
                if library {
                    this.config.library = Some(path);
                    this.save_config(cx);
                    this.request(Intent::Rescan, window, cx);
                } else {
                    this.request(Intent::Open(path), window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    fn remove(&mut self, removal: Removal, cx: &mut Context<Self>) {
        let result = match &removal {
            Removal::Path { path, .. } => ops::remove(path, &crate::scan::home()).map(|removed| match removed {
                Removed::Unlinked => format!("Removed the link {}", self.show(path)),
                Removed::Trashed => format!("Moved {} to the Trash", self.show(path)),
            }),
            Removal::Mcp { name, path, key, file } => {
                ops::mcp_remove(path, key, name).map(|()| format!("Removed {name} from {file}"))
            }
        };
        self.finish(result, cx);
    }

    fn add_server(&mut self, cx: &mut Context<Self>) {
        let Some(root) = self.project.as_ref().map(|project| project.root.clone()) else { return };
        let name = self.server_name.read(cx).text().trim().to_string();
        let command = self.server_command.read(cx).text().to_string();
        let (file, _, key) = MCP_TARGETS[0];
        let result = if name.is_empty() {
            Err(anyhow::anyhow!("Type a name for the server"))
        } else {
            ops::mcp_config(&command)
                .and_then(|config| ops::mcp_add(&root.join(file), key, &name, config))
                .map(|()| format!("Added {name} to {file}"))
        };
        if result.is_ok() {
            self.tab = Tab::Mcp;
            self.selected[Tab::Mcp as usize] = Some(name);
            for input in [&self.server_name, &self.server_command] {
                input.update(cx, |input, cx| input.set_text("", cx));
            }
        }
        self.finish(result, cx);
    }

    fn create_instruction(&mut self, name: &'static str, cx: &mut Context<Self>) {
        let Some(project) = &self.project else { return };
        let has_agents_md = project.instructions.iter().any(|file| file.rel == "AGENTS.md");
        let text = if name == "CLAUDE.md" && has_agents_md {
            "@AGENTS.md\n".to_string()
        } else {
            format!("# {name}\n")
        };
        let result = ops::create(&project.root.join(name), &text).map(|()| format!("Created {name}"));
        self.selected[Tab::Instructions as usize] = Some(name.to_string());
        self.finish(result, cx);
    }

    // ---------- drawing ----------

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (name, path) = match &self.project {
            Some(project) => (project.name.clone(), self.show(&project.root)),
            None => ("No project".to_string(), "Open a project folder to start.".to_string()),
        };
        let library = match &self.config.library {
            Some(library) => format!("Library: {}", self.show(library)),
            None => "No library".to_string(),
        };
        let mut recent = Menu::new();
        for root in &self.config.recent {
            let this = cx.entity();
            let root = root.clone();
            recent = recent.item(MenuItem::new(self.show(&root)).on_click(move |window, cx| {
                this.update(cx, |this, cx| this.request(Intent::Open(root.clone()), window, cx));
            }));
        }
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(Heading::h4(name))
                    .child(Caption::new(format!("{path}  ·  {library}"))),
            )
            .when(!self.config.recent.is_empty(), |row| {
                row.child(DropdownMenu::new("recent", "Recent", recent).icon(IconName::History))
            })
            .child(
                Button::new("global", "Global")
                    .icon(IconName::Library)
                    .disabled(self.project.as_ref().is_some_and(|project| project.global))
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.request(Intent::Open(scan::home()), window, cx))),
            )
            .child(
                Button::new("library", "Library…")
                    .icon(IconName::Library)
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.pick_folder(true, window, cx))),
            )
            .child(
                Button::new("rescan", "Rescan")
                    .icon(IconName::RefreshCw)
                    .shortcut("cmd-r")
                    .disabled(self.project.is_none())
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.request(Intent::Rescan, window, cx))),
            )
            .child(
                Button::new("open", "Open…")
                    .primary()
                    .icon(IconName::FolderOpen)
                    .shortcut("cmd-o")
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.pick_folder(false, window, cx))),
            )
    }

    fn tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let choices = Tab::ALL.map(|tab| Choice::new(tab.key(), format!("{}  {}", tab.label(), self.rows(tab).len())));
        div()
            .flex_none()
            .px_4()
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                Tabs::new("tabs", choices, self.tab.key()).on_change(cx.listener(
                    |this, key: &SharedString, window, cx| this.request(Intent::Tab(Tab::from_key(key)), window, cx),
                )),
            )
    }

    fn list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, surface) = (theme.colors.border, theme.colors.surface);
        let query = self.filter.read(cx).text().trim().to_lowercase();
        let all = self.rows(self.tab);
        let total = all.len();
        let rows: Vec<Row> = all
            .into_iter()
            .filter(|row| {
                query.is_empty() || format!("{} {}", row.title, row.description).to_lowercase().contains(&query)
            })
            .collect();

        let body = if rows.is_empty() {
            let (title, body) = if total == 0 {
                (format!("No {}", self.tab.label().to_lowercase()), self.empty_hint())
            } else {
                ("No match".to_string(), "Clear the filter to see all items.".to_string())
            };
            div()
                .p_6()
                .child(EmptyState::new("empty", IconName::Inbox, title).body(body))
                .into_any_element()
        } else {
            let mut list = SelectableList::new(("list", self.tab as usize))
                .selected(self.chosen(self.tab).map(str::to_string))
                .on_change(cx.listener(|this, keys: &[SharedString], window, cx| {
                    if let Some(key) = keys.first() {
                        this.request(Intent::Item(key.to_string()), window, cx);
                    }
                }));
            for (ix, row) in rows.into_iter().enumerate() {
                let mut item = ListItem::new(ix, row.title);
                if !row.description.is_empty() {
                    item = item.description(clip(&row.description, 96));
                }
                if !row.detail.is_empty() {
                    item = item.detail(row.detail);
                }
                if let Some(warning) = row.warning {
                    item = item.trailing(Badge::new(warning).tone(Tone::Warning));
                }
                list = list.row(row.key, item);
            }
            list.into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(LIST_WIDTH))
            .h_full()
            .border_r_1()
            .border_color(border)
            .bg(surface)
            .child(div().flex_none().p_3().child(SearchInput::new("filter", &self.filter)))
            .child(ScrollArea::new("list-scroll").flex_1().min_h_0().child(body))
            .children(self.list_actions(cx))
    }

    fn empty_hint(&self) -> String {
        match self.tab {
            Tab::Skills => "The project has no skills, and the library has none to install.",
            Tab::Mcp => "No config file in the project sets an MCP server. Add one below.",
            Tab::Agents => "The project has no agents, and the library has none to install.",
            Tab::Instructions => "The project has no CLAUDE.md or AGENTS.md. Create one below.",
            Tab::Drift => return self.plans.as_ref().err().cloned().unwrap_or_else(|| "manifest.toml has no scope.".to_string()),
        }
        .to_string()
    }

    /// What a tab can create, under its list.
    fn list_actions(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let project = self.project.as_ref()?;
        let buttons: Vec<Button> = match self.tab {
            Tab::Mcp => vec![
                Button::new("new-server", "New server…")
                    .icon(IconName::Plus)
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.prompt = Some(Prompt::NewServer);
                        cx.notify();
                    })),
            ],
            Tab::Instructions => ["AGENTS.md", "CLAUDE.md"]
                .into_iter()
                .filter(|name| !project.instructions.iter().any(|file| file.rel == *name))
                .map(|name| {
                    Button::new(name, format!("Create {name}"))
                        .icon(IconName::FilePlus)
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.create_instruction(name, cx)))
                })
                .collect(),
            Tab::Skills | Tab::Agents | Tab::Drift => Vec::new(),
        };
        if buttons.is_empty() {
            return None;
        }
        Some(
            div()
                .flex()
                .flex_none()
                .flex_wrap()
                .gap_2()
                .p_3()
                .border_t_1()
                .border_color(cx.theme().colors.border)
                .children(buttons)
                .into_any_element(),
        )
    }

    fn detail(&self, cx: &mut Context<Self>) -> AnyElement {
        let summary = match self.tab {
            Tab::Skills => self.skill().map(|skill| self.skill_summary(skill, cx)),
            Tab::Mcp => self.server().map(|server| self.server_summary(server, cx)),
            Tab::Agents => self.agent().map(|agent| self.agent_summary(agent, cx)),
            Tab::Instructions => self.instruction().map(|file| self.instruction_summary(file)),
            Tab::Drift => self.plan().map(|plan| self.drift_summary(plan, cx)),
        };
        let Some(summary) = summary else {
            return div()
                .flex_1()
                .p_8()
                .child(Caption::new("Select an item to see its details."))
                .into_any_element();
        };
        let pane = div().flex().flex_col().flex_1().min_w_0().h_full();
        match &self.editing {
            Some(editing) => pane
                .child(
                    ScrollArea::new(("summary", self.tab as usize))
                        .flex_none()
                        .max_h(px(SUMMARY_MAX_HEIGHT))
                        .child(summary),
                )
                .child(self.editor_bar(editing, cx))
                .child(div().flex_1().min_h_0().child(self.editor.clone())),
            None => pane.child(ScrollArea::new(("summary", self.tab as usize)).flex_1().min_h_0().child(summary)),
        }
        .into_any_element()
    }

    fn editor_bar(&self, editing: &Editing, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let dirty = self.dirty(cx);
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_3()
            .px_6()
            .py_2()
            .border_y_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .child(div().flex_1().min_w_0().child(Caption::new(self.show(&editing.path))))
            .when(dirty, |bar| bar.child(Badge::new("Not saved").tone(Tone::Warning).dot()))
            .child(
                Button::new("save", "Save")
                    .primary()
                    .size(ControlSize::Sm)
                    .icon(IconName::Save)
                    .shortcut("cmd-s")
                    .disabled(!dirty)
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.save(&Save, window, cx))),
            )
    }

    fn skill_summary(&self, skill: &Skill, cx: &mut Context<Self>) -> AnyElement {
        let Some(project) = &self.project else { return div().into_any_element() };
        let mut table = Table::new(["Tool", "Path", "Links to", ""]);
        let mut shown = Vec::new();
        // The tools this app installs for come first, installed or not.
        for (ix, &(rel_dir, tool)) in SKILL_DIRS.iter().enumerate() {
            let installs: Vec<_> = skill.installs.iter().filter(|install| install.rel.starts_with(rel_dir)).collect();
            if installs.is_empty() && ix < SKILL_TARGETS {
                let link = project.root.join(rel_dir).join(&skill.name);
                let (name, label) = (skill.name.clone(), tool.label());
                let button = match skill.library.clone().filter(|_| skill.origin.is_empty()) {
                    Some(library) => Button::new(("install", ix), "Install")
                        .size(ControlSize::Sm)
                        .icon(IconName::Plus)
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            let result = ops::link(&library, &link).map(|()| format!("Installed {name} for {label}"));
                            this.finish(result, cx);
                        })),
                    None if skill.origin.is_empty() => Button::new(("install", ix), "Not in the library").size(ControlSize::Sm).disabled(true),
                    None => Button::new(("install", ix), "Use the switch").size(ControlSize::Sm).disabled(true),
                };
                table = table.row([
                    tool.label().into_any_element(),
                    Caption::new("not installed").into_any_element(),
                    "".into_any_element(),
                    button.into_any_element(),
                ]);
            }
            for install in installs {
                shown.push(install.rel.clone());
                let links_to = match &install.link_target {
                    Some(target) if install.broken => format!("broken link: {}", self.show(target)),
                    Some(target) => self.show(target),
                    None => "a real folder".to_string(),
                };
                let removal = Removal::Path {
                    title: format!("Uninstall {} from {}?", skill.name, tool.label()),
                    path: install.abs.clone(),
                };
                table = table.row([
                    tool.label().into_any_element(),
                    install.rel.clone().into_any_element(),
                    links_to.into_any_element(),
                    Button::new(("uninstall", shown.len()), "Uninstall")
                        .size(ControlSize::Sm)
                        .variant(ButtonVariant::Ghost)
                        .icon(IconName::Trash2)
                        // A skill in a plugin leaves with its plugin, by the switch.
                        .disabled(install.in_plugin)
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.prompt = Some(Prompt::Remove(removal.clone()));
                            cx.notify();
                        }))
                        .into_any_element(),
                ]);
            }
        }

        let mut facts = DescriptionList::new();
        if let Some(library) = &skill.library {
            facts = facts.item("Library", self.show(library));
        }
        if !skill.source.is_empty() {
            facts = facts.item("Lock file source", skill.source.clone());
        }
        facts = facts.item("Files", skill.files.join("  "));

        let toggle = (!skill.origin.is_empty()).then(|| {
            let item = skill.origin.clone();
            let this = cx.entity();
            Switch::new("assigned", self.assigned.contains(&item))
                .label(format!("The plugin {item} is assigned to {} in manifest.toml", if project.global { "global scope" } else { "this project" }))
                .on_change(move |on, _, cx| this.update(cx, |this, cx| this.assign(item.clone(), on, cx)))
        });

        summary(&skill.name, &skill.description)
            .children(toggle)
            .when(skill.differs, |summary| {
                summary.child(Callout::new(Severity::Warning).title("Copies differ").child(
                    "The SKILL.md files are not the same in all locations. The editor has the first one.",
                ))
            })
            .child(table)
            .child(facts)
            .into_any_element()
    }

    fn server_summary(&self, server: &McpServer, cx: &mut Context<Self>) -> AnyElement {
        let Some(project) = &self.project else { return div().into_any_element() };
        let mut body = summary(&server.name, "");
        for (ix, def) in server.defs.iter().enumerate() {
            let mut facts = DescriptionList::new()
                .item("Scope", def.scope)
                .item("File", def.file.clone())
                .item("Status", def.status.label())
                .item("Transport", def.transport.clone());
            for (label, value) in [
                ("Command", def.command.clone()),
                ("URL", def.url.clone()),
                ("Env keys", def.env_keys.join(", ")),
                ("Header keys", def.header_keys.join(", ")),
            ] {
                if !value.is_empty() {
                    facts = facts.item(label, value);
                }
            }
            let action = if def.editable {
                let removal = Removal::Mcp {
                    name: server.name.clone(),
                    path: def.path.clone(),
                    key: def.key,
                    file: def.file.clone(),
                };
                Button::new(("remove-def", ix), "Remove")
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::Trash2)
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.prompt = Some(Prompt::Remove(removal.clone()));
                        cx.notify();
                    }))
                    .into_any_element()
            } else {
                Caption::new(format!("To change this, edit {} by hand.", def.file)).into_any_element()
            };
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().flex().items_center().gap_3().child(Heading::h5(def.tool.label())).child(action))
                    .child(facts),
            );
        }

        // A definition in the common `{ command, args, env }` shape can go to the other tools that use it.
        let portable = server.defs.iter().find(|def| def.key == "mcpServers");
        let missing: Vec<_> = MCP_TARGETS
            .iter()
            .filter(|(file, _, key)| *key == "mcpServers" && !server.defs.iter().any(|def| def.file == *file))
            .collect();
        if let Some(source) = portable.filter(|_| !missing.is_empty()) {
            let buttons = missing.into_iter().enumerate().map(|(ix, &(file, tool, key))| {
                let (name, config, path) = (server.name.clone(), source.raw.clone(), project.root.join(file));
                Button::new(("copy-def", ix), format!("Add to {} ({file})", tool.label()))
                    .size(ControlSize::Sm)
                    .icon(IconName::Plus)
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        let result =
                            ops::mcp_add(&path, key, &name, config.clone()).map(|()| format!("Added {name} to {file}"));
                        this.finish(result, cx);
                    }))
            });
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Overline::new("Install for other tools"))
                    .child(div().flex().flex_wrap().gap_2().children(buttons)),
            );
        }
        body.child(Caption::new("Env values, header values, and URL query values are never shown."))
            .into_any_element()
    }

    fn agent_summary(&self, agent: &Agent, cx: &mut Context<Self>) -> AnyElement {
        let mut facts = DescriptionList::new().item("Tool", agent.tool.label()).item("Path", agent.rel.clone());
        for (label, value) in [
            ("Links to", agent.link_target.as_ref().map(|target| self.show(target)).unwrap_or_default()),
            ("Model", agent.model.clone()),
            ("Tools", agent.tools.clone()),
            ("Library", agent.library.as_ref().map(|library| self.show(library)).unwrap_or_default()),
        ] {
            if !value.is_empty() {
                facts = facts.item(label, value);
            }
        }
        let action = if agent.installed {
            let removal = Removal::Path {
                title: format!("Uninstall the {} agent from {}?", agent.name, agent.tool.label()),
                path: agent.target.clone(),
            };
            Button::new("uninstall-agent", "Uninstall")
                .size(ControlSize::Sm)
                .variant(ButtonVariant::Ghost)
                .icon(IconName::Trash2)
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.prompt = Some(Prompt::Remove(removal.clone()));
                    cx.notify();
                }))
        } else {
            let (from, to, name, label) = (agent.file.clone(), agent.target.clone(), agent.name.clone(), agent.tool.label());
            Button::new("install-agent", format!("Install for {label}"))
                .size(ControlSize::Sm)
                .primary()
                .icon(IconName::Plus)
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    let result = ops::copy(&from, &to).map(|()| format!("Installed {name} for {label}"));
                    this.finish(result, cx);
                }))
        };
        summary(&agent.name, &agent.description)
            .when(!agent.installed, |summary| {
                summary.child(Caption::new("Not installed. The editor below has the library file."))
            })
            .child(div().flex().child(action))
            .child(facts)
            .into_any_element()
    }

    fn instruction_summary(&self, file: &Instruction) -> AnyElement {
        let mut facts = DescriptionList::new().item("Read by", file.tool).item("Scope", file.scope);
        if let Some(target) = &file.link_target {
            facts = facts.item("Links to", self.show(target));
        }
        if !file.imports.is_empty() {
            let imports: Vec<String> = file.imports.iter().map(|path| format!("@{path}")).collect();
            facts = facts.item("Imports", imports.join(", "));
        }
        summary(&file.rel, "").child(facts).into_any_element()
    }

    fn drift_summary(&self, plan: &Plan, cx: &mut Context<Self>) -> AnyElement {
        let home = scan::home();
        let title = if plan.scope.global { "Global".to_string() } else { apply::tilde(&home, &plan.scope.root) };
        let state = if plan.in_sync() {
            "The disk matches manifest.toml.".to_string()
        } else {
            format!("{} items on the disk do not match manifest.toml.", plan.changes.len())
        };
        let button = |id: &'static str, label: &'static str, adopt: bool| {
            let root = plan.scope.root.clone();
            Button::new(id, label)
                .size(ControlSize::Sm)
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.prompt = Some(Prompt::Apply { root: root.clone(), adopt });
                    cx.notify();
                }))
        };
        let mut actions = div().flex().gap_2().child(button("apply", "Apply", false).primary().disabled(plan.in_sync()));
        if plan.can_adopt() {
            actions = actions.child(button("adopt", "Apply and adopt", true));
        }
        let mut pane = summary(&title, &state).child(actions);
        for clash in &plan.clashes {
            pane = pane.child(Callout::new(Severity::Info).title(format!("{} is in two plugins", apply::tilde(&home, &clash.path))).child(format!(
                "{} and {} supply it. The link has no prefix, so {} wins there.",
                clash.winner,
                clash.losers.join(", "),
                clash.winner
            )));
        }
        for double in &plan.doubles {
            let dir = match plan.scope.global {
                true => apply::tilde(&home, &double.dir),
                false => double.dir.strip_prefix(&plan.scope.root).unwrap_or(&double.dir).display().to_string(),
            };
            pane = pane.child(Callout::new(Severity::Warning).title(format!("{} gets skills twice", double.reader)).child(double.message(&dir)));
        }
        for (label, names) in plan.groups(&home) {
            pane = pane.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Overline::new(format!("{label} · {}", names.len())))
                    .child(Paragraph::new(names.join("  "))),
            );
        }
        pane.into_any_element()
    }

    fn dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let this = cx.entity();
        let close = {
            let this = this.clone();
            move |_: &mut Window, cx: &mut App| {
                this.update(cx, |this, cx| {
                    this.prompt = None;
                    cx.notify();
                })
            }
        };
        Some(match self.prompt.as_ref()? {
            Prompt::Discard(intent) => {
                let intent = intent.clone();
                ConfirmDialog::new("discard", "Discard your changes?", "The file in the editor has changes that are not saved.", close)
                    .confirm("Discard")
                    .destructive()
                    .on_confirm(move |window, cx| {
                        this.update(cx, |this, cx| this.apply(intent.clone(), window, cx));
                    })
                    .into_any_element()
            }
            Prompt::Remove(removal) => {
                let (title, message) = match removal {
                    Removal::Path { title, path } if path.is_symlink() => (
                        title.clone(),
                        format!("This removes the link {}. The files that the link points to stay.", self.show(path)),
                    ),
                    Removal::Path { title, path } => (
                        title.clone(),
                        format!("{} is not a link. This moves it to the Trash.", self.show(path)),
                    ),
                    Removal::Mcp { name, file, .. } => (
                        format!("Remove {name} from {file}?"),
                        format!("This deletes the {name} entry from {file}. Other tools keep their own entries."),
                    ),
                };
                let removal = removal.clone();
                ConfirmDialog::new("remove", title, message, close)
                    .confirm("Remove")
                    .destructive()
                    .on_confirm(move |_, cx| {
                        this.update(cx, |this, cx| this.remove(removal.clone(), cx));
                    })
                    .into_any_element()
            }
            Prompt::Apply { root, adopt } => {
                let (root, adopt) = (root.clone(), *adopt);
                let message = if adopt {
                    "This writes the links that manifest.toml assigns, and replaces the links that something else made. A backup keeps the old link targets."
                } else {
                    "This writes the links that manifest.toml assigns. It leaves each item that workflow-setup does not own. A backup keeps the old link targets."
                };
                ConfirmDialog::new("apply", format!("Apply to {}?", apply::tilde(&scan::home(), &root)), message, close)
                    .confirm("Apply")
                    .on_confirm(move |_, cx| {
                        this.update(cx, |this, cx| {
                            let result = this.env().context("No library").and_then(|env| this.apply_scope(&env, &root, adopt));
                            this.finish(result.map(|done| format!("Applied. {done}")), cx);
                        });
                    })
                    .into_any_element()
            }
            Prompt::NewServer => Dialog::new("new-server", "New MCP server", close)
                .detail("The server goes into .mcp.json, which Claude Code reads. You can add it to other tools after that.")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(Input::new(&self.server_name))
                        .child(Input::new(&self.server_command)),
                )
                .action(|close| {
                    Button::new("cancel-server", "Cancel")
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |_, window, cx| close(window, cx))
                })
                .action(move |close| {
                    Button::new("add-server", "Add").primary().on_click(move |_, window, cx| {
                        close(window, cx);
                        this.update(cx, |this, cx| this.add_server(cx));
                    })
                })
                .into_any_element(),
        })
    }
}

/// The head of a detail pane: the item's name, then what it is for.
fn summary(title: &str, description: &str) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(Heading::h3(title.to_string()))
        .when(!description.is_empty(), |summary| summary.child(Paragraph::new(description.to_string())))
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
        let body = if self.project.is_some() {
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(self.list(cx))
                .child(self.detail(cx))
                .into_any_element()
        } else {
            div()
                .flex_1()
                .p_8()
                .child(
                    EmptyState::new("no-project", IconName::FolderOpen, "No project is open")
                        .body("Open a project folder to see its skills, MCP servers, agents, and instruction files."),
                )
                .into_any_element()
        };
        div()
            .size_full()
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(|this, _: &OpenProject, window, cx| this.pick_folder(false, window, cx)))
            .on_action(cx.listener(|this, _: &ChooseLibrary, window, cx| this.pick_folder(true, window, cx)))
            .on_action(cx.listener(|this, _: &Rescan, window, cx| this.request(Intent::Rescan, window, cx)))
            .child(
                FocusScope::new(&self.focus)
                    .root()
                    .size_full()
                    .flex()
                    .flex_col()
                    .bg(colors.bg)
                    .text_color(colors.fg)
                    .child(self.header(cx))
                    .child(self.tabs(cx))
                    .child(body)
                    .children(self.dialog(cx))
                    .child(ToastViewport::new("toasts", &self.toaster)),
            )
    }
}
