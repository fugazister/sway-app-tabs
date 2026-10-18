//! Keep every tiled window of the given apps together in one tabbed container.
//!
//! When a new window of a listed app opens, it is moved next to an existing
//! window of the same app (on whatever workspace that one lives), and the two
//! are wrapped in a tabbed container holding only that app's windows. A lone
//! first window is left untouched.
//!
//! Usage: sway-app-tabs <app_id or X11 class>...   (started from sway config)

use std::{env, fs, path::PathBuf, process::ExitCode};

use swayipc::{Connection, Event, EventType, Node, NodeLayout, NodeType, WindowChange};

const MARK: &str = "_app_tabs";

fn main() -> ExitCode {
    let apps: Vec<String> = env::args().skip(1).collect();
    if apps.is_empty() {
        eprintln!("usage: sway-app-tabs <app_id or X11 class>...");
        return ExitCode::FAILURE;
    }
    replace_previous_instance();

    match run(&apps) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sway-app-tabs: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(apps: &[String]) -> swayipc::Fallible<()> {
    let mut conn = Connection::new()?;
    for event in Connection::new()?.subscribe([EventType::Window])? {
        let Event::Window(ev) = event? else { continue };
        let win = &ev.container;
        if ev.change != WindowChange::New || win.node_type != NodeType::Con {
            continue;
        }
        let Some(app) = app_of(win).filter(|a| apps.iter().any(|x| x == a)) else {
            continue;
        };
        let tree = conn.get_tree()?;
        let Some((other, parent_layout)) = find_other(&tree, app, win.id) else {
            continue;
        };
        // Not grouped yet: wrap the existing window in its own container first,
        // so making it tabbed doesn't pull in other apps sharing the workspace.
        let split = if parent_layout == NodeLayout::Tabbed {
            String::new()
        } else {
            format!("[con_id={other}] split v; ")
        };
        let cmd = format!(
            "{split}[con_id={other}] mark --add {MARK}; \
             [con_id={id}] move container to mark {MARK}, layout tabbed, focus; \
             unmark {MARK}",
            id = win.id
        );
        for res in conn.run_command(&cmd)? {
            if let Err(e) = res {
                eprintln!("sway-app-tabs: {e}");
            }
        }
    }
    Ok(())
}

fn app_of(node: &Node) -> Option<&str> {
    node.app_id
        .as_deref()
        .or_else(|| node.window_properties.as_ref()?.class.as_deref())
}

/// First tiled window of `app` other than `skip`, outside the scratchpad,
/// together with the layout of its parent container.
fn find_other(root: &Node, app: &str, skip: i64) -> Option<(i64, NodeLayout)> {
    root.nodes
        .iter()
        .flat_map(|output| &output.nodes)
        .filter(|ws| ws.name.as_deref() != Some("__i3_scratch"))
        .find_map(|ws| search(ws, app, skip))
}

fn search(parent: &Node, app: &str, skip: i64) -> Option<(i64, NodeLayout)> {
    parent.nodes.iter().find_map(|n| {
        if n.pid.is_some() && n.id != skip && app_of(n) == Some(app) {
            Some((n.id, parent.layout))
        } else {
            search(n, app, skip)
        }
    })
}

/// `exec_always` starts a new instance on every sway reload: stop the old one.
fn replace_previous_instance() {
    let dir = env::var_os("XDG_RUNTIME_DIR").map_or_else(env::temp_dir, PathBuf::from);
    let pidfile = dir.join("sway-app-tabs.pid");
    if let Some(pid) = fs::read_to_string(&pidfile)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
    {
        // Guard against the pid having been reused by an unrelated process.
        let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
        if comm.trim() == "sway-app-tabs" {
            // SAFETY: plain kill(2) on a pid we just verified.
            unsafe { libc_kill(pid, 15) };
        }
    }
    let _ = fs::write(&pidfile, std::process::id().to_string());
}

unsafe extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}
