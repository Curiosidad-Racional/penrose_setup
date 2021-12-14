#[macro_use]
extern crate penrose;
extern crate xcb;

mod easyfocus;
mod hooks;
mod layouts;
mod nvim;

use penrose::{
    contrib::{extensions::Scratchpad, layouts::paper},
    core::{
        bindings::MouseEvent,
        config::Config,
        data_types::Region,
        helpers::spawn_with_args,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Selector,
        workspace::Workspace,
    },
    draw::{bar::dwm_bar, Color, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw, XcbHooks},
    Backward, Forward, Less, More,
};
use simplelog::{LevelFilter, SimpleLogger};
use std::{
    // io::Read,
    // process::{Command, Stdio},
    convert::TryFrom,
};
// use dirs::home_dir;

const HEIGHT: usize = 18;

const FONT: &str = "Iosevka Nerd Font";

const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
// const PURPLE: u32 = 0xb16286ff;
const BLUE: u32 = 0x458588ff;
// const RED: u32 = 0xcc241dff;

// fn spawn_for_output_with_args<S: Into<String>>(cmd: S, args: &[&str]) -> penrose::Result<String> {
//     let cmd = cmd.into();

//     let child = Command::new(&cmd)
//         .stdout(Stdio::piped())
//         .args(args)
//         .spawn()?;

//     let mut buff = String::new();
//     Ok(child
//         .stdout
//         .ok_or(penrose::PenroseError::SpawnProc(cmd))?
//         .read_to_string(&mut buff)
//         .map(|_| buff)?)
// }

fn main() -> penrose::Result<()> {
    if let Err(e) = SimpleLogger::init(LevelFilter::Info, simplelog::Config::default()) {
        panic!("unable to set log level: {}", e);
    }

    // spawn_for_output("xrandr-monitors --run")?;
    // spawn_with_args(
    //     "dunst",
    //     &["-history_length", "100", "-history_key", "mod4+ccedilla",
    //       "-key", "mod4+shift+ccedilla", "-context_key", "mod4+shift+ntilde",
    //       "-lto", "10s", "-nto", "15s", "-cto", "20s",
    //       "-show_age_threshold", "1m", "-idle_threshold", "10m",
    //       "-format", "%a: %s %n\\n%b"])?;
    // spawn("compton -b --config /dev/null --backend xrender")?;
    // spawn(format!("feh --bg-scale --randomize {}/Pictures/wallpapers/",
    //                home_dir().unwrap().display()))?;
    // spawn_with_args("keynav", &["loadconfig ~/.config/keynav/keynavrc"])?;
    let config = Config::default()
        .builder()
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .floating_classes(vec!["rofi", "dmenu", "dunst", "yad", "gcr-prompter"])
        .opaque_classes(vec!["emacs", "Alacritty"])
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, 1, 0.6),
            Layout::new(
                "[mono]",
                LayoutConf {
                    floating: false,
                    gapless: true,
                    follow_focus: true,
                    allow_wrapping: true,
                },
                monocle,
                1,
                0.6,
            ),
            Layout::new(
                "[papr]",
                LayoutConf {
                    floating: false,
                    gapless: true,
                    follow_focus: true,
                    allow_wrapping: false,
                },
                paper,
                1,
                0.6,
            ),
            Layout::new("[dwdl]", LayoutConf::default(), layouts::dwindle, 1, 0.6),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, 1, 0.6),
        ])
        .build()
        .unwrap();
    let bar = dwm_bar(
        XcbDraw::new()?,
        HEIGHT,
        &TextStyle {
            font: FONT.to_string(),
            point_size: 10,
            fg: Color::try_from(WHITE).unwrap(),
            bg: Some(Color::try_from(BLACK).unwrap()),
            padding: (2.0, 2.0),
        },
        BLUE,
        GREY,
        config.workspaces().clone(),
    )?;

    let sp_term = Scratchpad::new("alacritty", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        hooks::CustomHook::new(
            config.floating_classes().clone(),
            config.opaque_classes().clone(),
        ),
        sp_term.get_hook(),
        Box::new(bar),
    ];

    let key_bindings = gen_keybindings! {
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-l" => run_internal!(cycle_screen, Forward);
        "M-h" => run_internal!(cycle_screen, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-S-l" => run_internal!(drag_workspace, Forward);
        "M-S-h" => run_internal!(drag_workspace, Backward);
        "M-S-i" => run_internal!(rotate_clients, Forward);
        "M-S-u" => run_internal!(rotate_clients, Backward);
        "M-o" => Box::new(&easyfocus::easyfocus);
        "Caps_Lock" => Box::new(|_wm: &mut WindowManager<_>| {
            nvim::command("highlight Normal ctermbg=darkRed guibg=darkRed")
        });
        "L-Caps_Lock" => Box::new(|_wm: &mut WindowManager<_>| {
            nvim::command("highlight Normal ctermbg=NONE guibg=NONE")
        });
        "M-S-Right" => Box::new(|wm: &mut WindowManager<_>| {
            if let Some(id) = wm.focused_client_id()
            {
                if let Ok((conn, _)) = xcb::Connection::connect(None)
                {
                    if let Ok(res) = xcb::get_geometry(&conn, id).get_reply()
                    {
                        let w = res.width() as u32;
                        let h = res.height() as u32;
                        wm.position_client(id, Region::new(
                            res.x() as u32,
                            res.y() as u32,
                            w + w / 5,
                            h + h / 5,
                        ), true)?;
                    }
                }
            }
            Ok(())
        });
        "M-S-Left" => Box::new(|wm: &mut WindowManager<_>| {
            if let Some(id) = wm.focused_client_id()
            {
                if let Ok((conn, _)) = xcb::Connection::connect(None)
                {
                    if let Ok(res) = xcb::get_geometry(&conn, id).get_reply()
                    {
                        let w = res.width() as u32;
                        let h = res.height() as u32;
                        wm.position_client(id, Region::new(
                            res.x() as u32,
                            res.y() as u32,
                            w - w / 5,
                            h - h / 5,
                        ), true)?;
                    }
                }
            }
            Ok(())
        });
        "M-S-Up" => Box::new(|wm: &mut WindowManager<_>| {
            if let Some(id) = wm.focused_client_id()
            {
                if let Ok((conn, _)) = xcb::Connection::connect(None)
                {
                    if let Ok(res) = xcb::get_geometry(&conn, id).get_reply()
                    {
                        let w = res.width() as u32;
                        let h = res.height() as u32;
                        let dx = w / 5;
                        let dy = h / 5;
                        wm.position_client(id, Region::new(
                            res.x() as u32 - dx / 2,
                            res.y() as u32 - dy / 2,
                            w + dx,
                            h + dy,
                        ), true)?;
                    }
                }
            }
            Ok(())
        });
        "M-S-Down" => Box::new(|wm: &mut WindowManager<_>| {
            if let Some(id) = wm.focused_client_id()
            {
                if let Ok((conn, _)) = xcb::Connection::connect(None)
                {
                    if let Ok(res) = xcb::get_geometry(&conn, id).get_reply()
                    {
                        let w = res.width() as u32;
                        let h = res.height() as u32;
                        let dx = w / 5;
                        let dy = h / 5;
                        wm.position_client(id, Region::new(
                            res.x() as u32 + dx / 2,
                            res.y() as u32 + dy / 2,
                            w - dx,
                            h - dy,
                        ), true)?;
                    }
                }
            }
            Ok(())
        });
        // "M-u" => Box::new(|wm: &mut WindowManager<_>| {
        //     let mut clients:Vec<String> = vec![];
        //     if let Some(id) = wm.focused_client_id() {
        //         clients.push(format!("{:#010x}", id));
        //     }
        //     for workspace_index in wm.focused_workspaces().iter()
        //     {
        //         if let Some(workspace) = wm.workspace(&Selector::Index(*workspace_index))
        //         {
        //             clients.extend(workspace.client_ids().iter().map(|id| format!("{:#010x}", id)));
        //         } else {
        //             return Ok(())
        //         }
        //     }
        //     if let Ok(mut client) = spawn_for_output_with_args(
        //         "easyfocus_penrose",
        //         &clients.iter().map(AsRef::as_ref).collect::<Vec<&str>>())
        //     {
        //         if client.ends_with('\n')
        //         {
        //             client.pop();
        //             if client.ends_with('\r')
        //             {
        //                 client.pop();
        //             }
        //         }
        //         if client.starts_with("0x")
        //         {
        //             if let Ok(id) = u32::from_str_radix(client.trim_start_matches("0x"), 16) {
        //                 if let Ok(_) = wm.focus_client(&Selector::WinId(id))
        //                 {
        //                     return Ok(());
        //                 }
        //             }
        //         } else {
        //             if let Ok(index) = client.parse::<usize>() {
        //                 if let Ok(_) = wm.focus_workspace(&Selector::Index(index - 1)) {
        //                     return Ok(())
        //                 }
        //             }
        //         }
        //     }
        //     Ok(())
        // });
        "M-c" => run_internal!(warp_cursor);
        "M-S-c" => Box::new(|wm: &mut WindowManager<_>| {
            if let Some(id) = wm.focused_client_id()
            {
                if let Some(region) = wm.screen_size(wm.active_screen_index())
                {
                    let r = region.scale_w(0.9).scale_h(0.9);
                    wm.position_client(id, r.centered_in(&region)?, true)?;
                }
            }
            Ok(())
        });
        "M-S-q" => run_internal!(kill_client);
        "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-Tab" => run_internal!(toggle_workspace);
        "M-C-Return" => sp_term.toggle();
        "M-n" => run_internal!(cycle_workspace, Forward);
        "M-p" => run_internal!(cycle_workspace, Backward);
        "M-A-n" => Box::new(|wm: &mut WindowManager<_>| {
            let focused_workspaces: Vec<&Workspace> = wm.focused_workspaces().iter().map(
                |index| wm.workspace(&Selector::Index(*index)).unwrap()).collect();
            let active_workspace = wm.active_workspace();
            let mut found: u8 = 0;
            let workspaces = wm.all_workspaces(&Selector::Condition(&|_| true));
            let mut index: usize = 0;
            for workspace in workspaces.iter()
            {
                if found == 1 && !focused_workspaces.contains(workspace)
                {
                    found = 2;
                    break;
                } else {
                    if workspace == &active_workspace
                    {
                        found = 1;
                    }
                }
                index += 1;
            }
            if found != 2
            {
                index = 0;
                for workspace in workspaces.iter()
                {
                    if !focused_workspaces.contains(workspace) {
                        found = 2;
                        break;
                    }
                    index += 1;
                }
            }
            if found == 2 {
                let _ = wm.focus_workspace(&Selector::Index(index));
            }
            Ok(())
        });
        "M-A-p" => Box::new(|wm: &mut WindowManager<_>| {
            let focused_workspaces: Vec<&Workspace> = wm.focused_workspaces().iter().map(
                |index| wm.workspace(&Selector::Index(*index)).unwrap()).collect();
            let active_workspace = wm.active_workspace();
            let mut found: u8 = 0;
            let workspaces = wm.all_workspaces(&Selector::Condition(&|_| true));
            let workspaces_len = workspaces.len();
            let mut index = workspaces_len;
            for workspace in workspaces.iter().rev()
            {
                index -= 1;
                if found == 1 && !focused_workspaces.contains(workspace)
                {
                    found = 2;
                    break;
                } else {
                    if workspace == &active_workspace
                    {
                        found = 1;
                    }
                }
            }
            if found != 2
            {
                index = workspaces_len;
                for workspace in workspaces.iter().rev()
                {
                    index -= 1;
                    if !focused_workspaces.contains(workspace) {
                        found = 2;
                        break;
                    }
                }
            }
            if found == 2 {
                let _ = wm.focus_workspace(&Selector::Index(index));
            }
            Ok(())
        });
        "M-bracketright" => run_internal!(cycle_layout, Forward);
        "M-bracketleft" => run_internal!(cycle_layout, Backward);
        "M-A-k" => run_internal!(update_max_main, More);
        "M-A-j" => run_internal!(update_max_main, Less);
        "M-A-l" => run_internal!(update_main_ratio, More);
        "M-A-h" => run_internal!(update_main_ratio, Less);
        "M-d" => run_external!(
            "rofi -m -1 -show run \
             -kb-accept-entry Control+m,Return,KP_Enter -kb-accept-custom \
             Control+j,Control+Return -kb-select-1 ctrl+1 -kb-select-2 ctrl+2 \
             -kb-select-3 ctrl+3 -kb-select-4 ctrl+4 -kb-select-5 ctrl+5 \
             -kb-select-6 ctrl+6 -kb-select-7 ctrl+7 -kb-select-8 ctrl+8 \
             -kb-select-9 ctrl+9 -kb-select-10 ctrl+0 -kb-page-prev alt+p \
             -kb-page-next alt+n -kb-secondary-paste ctrl+y");
        "M-S-d" => run_external!(
            "rofi -m -1 -show drun \
             -kb-accept-entry Control+m,Return,KP_Enter -kb-accept-custom \
             Control+j,Control+Return -kb-select-1 ctrl+1 -kb-select-2 ctrl+2 \
             -kb-select-3 ctrl+3 -kb-select-4 ctrl+4 -kb-select-5 ctrl+5 \
             -kb-select-6 ctrl+6 -kb-select-7 ctrl+7 -kb-select-8 ctrl+8 \
             -kb-select-9 ctrl+9 -kb-select-10 ctrl+0 -kb-page-prev alt+p \
             -kb-page-next alt+n -kb-secondary-paste ctrl+y");
        "M-A-space" => run_external!("remap");
        "M-Prior" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ +5%");
        "M-Next" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ -5%");
        "M-C-t" => Box::new(|wm: &mut WindowManager<_>| {
            wm.set_win_opacity(wm.get_win_opacity() - 0.1)
        });
        "M-S-t" => Box::new(|wm: &mut WindowManager<_>| {
            wm.set_win_opacity(wm.get_win_opacity() + 0.1)
        });
        "M-S-b" => Box::new(|_: &mut WindowManager<_>| {
            spawn_with_args("sh", &["-c", "feh --bg-scale --randomize ~/Pictures/wallpapers/*.jpg"])
        });
        "M-Pause" => Box::new(|_: &mut WindowManager<_>| {
            spawn_with_args("xscreensaver-command", &["-lock"])
        });
        "M-Return" => run_external!("ac");
        "M-S-Return" => run_external!("ec");
        "M-A-Escape" => run_internal!(exit);
        "M-1" => run_internal!(focus_workspace, &Selector::Index(0));
        "M-S-1" => run_internal!(client_to_workspace, &Selector::Index(0));
        "M-2" => run_internal!(focus_workspace, &Selector::Index(1));
        "M-S-2" => run_internal!(client_to_workspace, &Selector::Index(1));
        "M-3" => run_internal!(focus_workspace, &Selector::Index(2));
        "M-S-3" => run_internal!(client_to_workspace, &Selector::Index(2));
        "M-4" => run_internal!(focus_workspace, &Selector::Index(3));
        "M-S-4" => run_internal!(client_to_workspace, &Selector::Index(3));
        "M-5" => run_internal!(focus_workspace, &Selector::Index(4));
        "M-S-5" => run_internal!(client_to_workspace, &Selector::Index(4));
        "M-6" => run_internal!(focus_workspace, &Selector::Index(5));
        "M-S-6" => run_internal!(client_to_workspace, &Selector::Index(5));
        "M-7" => run_internal!(focus_workspace, &Selector::Index(6));
        "M-S-7" => run_internal!(client_to_workspace, &Selector::Index(6));
        "M-8" => run_internal!(focus_workspace, &Selector::Index(7));
        "M-S-8" => run_internal!(client_to_workspace, &Selector::Index(7));
        "M-9" => run_internal!(focus_workspace, &Selector::Index(8));
        "M-S-9" => run_internal!(client_to_workspace, &Selector::Index(8));
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    wm.grab_keys_and_run(key_bindings, mouse_bindings)?;

    Ok(())
}
