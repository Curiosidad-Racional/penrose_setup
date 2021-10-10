#[macro_use]
extern crate penrose;

use penrose::{
    contrib::{
        layouts::paper,
        extensions::Scratchpad,
    },
    core::{
        data_types::WinId,
        workspace::Workspace,
        bindings::MouseEvent,
        config::Config,
        helpers::{spawn, spawn_with_args, spawn_for_output},
        manager::WindowManager,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        ring::Selector,
    },
    draw::{bar::dwm_bar, Color, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw, XcbHooks},
    Backward, Forward, Less, More,
};
use simplelog::{LevelFilter, SimpleLogger};
use std::convert::TryFrom;
// use dirs::home_dir;
mod hooks;
use hooks::CenterFloat;

const HEIGHT: usize = 18;

const FONT: &str = "Iosevka Nerd Font";

const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
// const PURPLE: u32 = 0xb16286ff;
const BLUE: u32 = 0x458588ff;
// const RED: u32 = 0xcc241dff;

use std::{
    // io::Read,
    // process::{Command, Stdio},
    collections::HashMap,
    env,
};


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

extern crate xcb;

fn show_window_text(
    conn: &xcb::base::Connection,
    parent: xcb::ffi::xproto::xcb_window_t,
    text: &str) -> Option<xcb::ffi::xproto::xcb_window_t> {
    let font = conn.generate_id();
    if let Ok(_) = xcb::xproto::open_font_checked(
        conn, font, "-*-fixed-medium-*-*-*-18-*-*-*-*-*-*-*").request_check()
    {
        let window = conn.generate_id();
        xcb::xproto::create_window(
            conn,
            xcb::COPY_FROM_PARENT as u8,
            window,
            parent,
            0, 0,
            (text.len() * 9 + 6) as u16, 18,
            0,
            xcb::WINDOW_CLASS_COPY_FROM_PARENT as u16,
            xcb::base::COPY_FROM_PARENT,
            &[(xcb::xproto::CW_BACK_PIXEL, 0xff000000),
              (xcb::xproto::CW_EVENT_MASK, xcb::xproto::EVENT_MASK_EXPOSURE)]
        );
        xcb::xproto::map_window(conn, window);
        conn.flush();

        let gc = conn.generate_id();
        xcb::xproto::create_gc(
            conn, gc, window,
            &[(xcb::xproto::GC_FOREGROUND, 0xffff2cc4),
              (xcb::xproto::GC_BACKGROUND, 0xff000000),
              (xcb::xproto::GC_FONT, font)],
        );
        xcb::xproto::close_font(conn, font);


        let _ = xcb::xproto::image_text_8_checked(
            conn, window, gc, 4, 14, text).request_check();
        xcb::xproto::free_gc(conn, gc);
        return Some(window)
    }
    None
}


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
        .floating_classes(vec!["rofi", "dmenu", "dunst", "yad"])
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, 1, 0.6),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, 1, 0.6),
            Layout::new("[mono]", LayoutConf{
                floating: false, gapless: true, follow_focus: true, allow_wrapping: true,
            }, monocle, 1, 0.6),
            Layout::new("[papr]", LayoutConf{
                floating: false, gapless: true, follow_focus: true, allow_wrapping: false,
            }, paper, 1, 0.6),
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
        sp_term.get_hook(),
        Box::new(bar),
        CenterFloat::new(config.floating_classes().clone(), 0.9),
    ];

    let cycle_screen_direction = match env::var("MONITORS_LAYOUT") {
        Ok(val) => {
            match val.as_str() {
                "left" => Backward,
                "right" => Forward,
                _ => Forward
            }
        },
        Err(_) => Forward
    };

    let key_bindings = gen_keybindings! {
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-l" => run_internal!(cycle_screen, cycle_screen_direction);
        "M-h" => run_internal!(cycle_screen, cycle_screen_direction.reverse());
        // "M-l" => run_internal!(rotate_clients, Forward);
        // "M-h" => run_internal!(rotate_clients, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-o" => Box::new(|wm: &mut WindowManager<_>| {
            if let Ok((conn, _)) = xcb::Connection::connect(None) {
                let mut letters = String::from("0987654321nbmvcxzytpoiurewqhglkjfdsa");
                let mut window_text_ids: Vec<WinId> = vec![];
                let mut client_ids:HashMap<char, WinId> = HashMap::new();
                for workspace_index in wm.focused_workspaces().iter()
                {
                    if let Some(workspace) = wm.workspace(&Selector::Index(*workspace_index))
                    {
                        for client_id in workspace.client_ids().iter() {
                            if let Some(letter) = letters.pop()
                            {
                                if let Some(window_text_id) = show_window_text(
                                    &conn, *client_id, letter.to_string().as_str())
                                {
                                    window_text_ids.push(window_text_id);
                                    client_ids.insert(letter, *client_id);
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                let choice_result = spawn_for_output("rofi-getch");
                // let window = conn.generate_id();
                // let screen = conn.get_setup().roots().nth(screen_i as usize).unwrap();
                // xcb::create_window(
                //     &conn,
                //     xcb::COPY_FROM_PARENT as u8,
                //     window,
                //     screen.root(),
                //     0,
                //     0,
                //     150,
                //     150,
                //     10,
                //     xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                //     screen.root_visual(),
                //     &[
                //         (xcb::CW_BACK_PIXEL, screen.white_pixel()),
                //         (
                //             xcb::CW_EVENT_MASK,
                //             xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS,
                //         ),
                //     ],
                // );
                // xcb::map_window(&conn, window);
                // conn.flush();
                // loop {
                //     let ev = conn.wait_for_event();
                //     if let Some(ev) = ev {
                //         if ev.response_type() == xcb::KEY_PRESS as u8 {
                //             let key: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&ev) };
                //             println!("Key {}", key.detail());
                //             if key.detail() as char == 'M' {
                //                 println!("M matched")
                //             }
                //             if key.detail() as char == 'm' {
                //                 println!("m matched")
                //             }
                //             if key.detail() == 0x3a {
                //                 xcb::xproto::destroy_window(&conn, window);
                //                 break;
                //             }
                //         }
                //     } else {
                //         xcb::xproto::destroy_window(&conn, window);
                //         break;
                //     }
                // }
                for window_text_id in window_text_ids.iter()
                {
                    xcb::xproto::destroy_window(&conn, *window_text_id);
                }
                if let Ok(choosed) = choice_result
                {
                    if let Some(ch) = choosed.chars().next()
                    {
                        if let Some(&client_id) = client_ids.get(&ch)
                        {
                            let _ = wm.focus_client(&Selector::WinId(client_id));
                        }
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
        "M-S-l" => run_internal!(drag_workspace, Forward);
        "M-S-h" => run_internal!(drag_workspace, Backward);
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
            if let Some(id) = wm.focused_client_id() {
                spawn_with_args("transset", &["--id", &id.to_string(), "0.9"])
            } else {
                Ok(())
            }
        });
        "M-S-b" => Box::new(|_: &mut WindowManager<_>| {
            spawn_with_args("sh", &["-c", "feh --bg-scale --randomize ~/Pictures/wallpapers/*.jpg"])
        });
        "M-Pause" => Box::new(|_: &mut WindowManager<_>| {
            spawn_with_args("xscreensaver-command", &["-lock"])
        });
        "M-Return" => run_external!("ec");
        "M-S-Return" => run_external!("alacritty");
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
    spawn("lbarstat")?;
    wm.grab_keys_and_run(key_bindings, mouse_bindings)?;

    Ok(())
}
