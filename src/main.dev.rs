#[macro_use]
extern crate penrose;

use penrose::{
    contrib::{
        layouts::paper,
        extensions::Scratchpad,
        hooks::ManageExistingClients,
    },
    core::{
        bindings::MouseEvent,
        config::Config,
        helpers::index_selectors,
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
use dirs::home_dir;


const HEIGHT: usize = 18;

const FONT: &str = "Iosevka Nerd Font";

const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
const PURPLE: u32 = 0xb16286ff;
const BLUE: u32 = 0x458588ff;
const RED: u32 = 0xcc241dff;



fn main() -> penrose::Result<()> {
    if let Err(e) = SimpleLogger::init(LevelFilter::Info, simplelog::Config::default()) {
        panic!("unable to set log level: {}", e);
    }

    spawn_for_output!("xrandr-monitors --run")?;
    spawn!("dunst", "-history_length", "100", "-history_key", "mod4+ccedilla",
           "-key", "mod4+shift+ccedilla", "-context_key", "mod4+shift+ntilde",
           "-lto", "10s", "-nto", "15s", "-cto", "20s", "-show_age_threshold", "1m",
           "-idle_threshold", "10m", "-format", "%a: %s %n\\n%b")?;
    spawn!("compton -b --config /dev/null --backend xrender")?;
    spawn!(format!("feh --bg-scale --randomize {}/Pictures/wallpapers/",
                   home_dir().unwrap().display()))?;
    spawn!("keynav","loadconfig ~/.config/keynav/keynavrc")?;
    let config = Config::default()
        .builder()
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .floating_classes(vec!["rofi", "dmenu", "dunst"])
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
            fg: Color::try_from(WHITE)?,
            bg: Some(Color::try_from(BLACK)?),
            padding: (2.0, 2.0),
        },
        BLUE,
        GREY,
        config.workspaces().clone(),
    )?;

    let sp = Scratchpad::new("alacritty", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        ManageExistingClients::new(),
        sp.get_hook(),
        Box::new(bar),
    ];

    let key_bindings = gen_keybindings! {
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-l" => run_internal!(rotate_clients, Forward);
        "M-h" => run_internal!(rotate_clients, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        // "M-S-j" => Box::new(|wm: &mut WindowManager<_>| {
        //     println!(" pre {:#?}", wm.active_workspace().client_ids());
        //     let result = wm.drag_client(Forward);
        //     println!("post {:#?}", wm.active_workspace().client_ids());
        //     result
        // });
        "M-S-q" => run_internal!(kill_client);
        "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-Tab" => run_internal!(toggle_workspace);
        "M-C-Return" => sp.toggle();
        "M-n" => run_internal!(cycle_workspace, Forward);
        "M-p" => run_internal!(cycle_workspace, Backward);
        "M-period" => run_internal!(cycle_screen, Forward);
        "M-comma" => run_internal!(cycle_screen, Backward);
        "M-S-period" => run_internal!(drag_workspace, Forward);
        "M-S-comma" => run_internal!(drag_workspace, Backward);
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
        "M-Return" => run_external!("emacsclient -c -n -a ''");
        "M-S-Return" => run_external!("alacritty");
        "M-A-Escape" => run_internal!(exit);
        map: { "1", "2", "3", "4", "5", "6", "7", "8", "9" } to index_selectors(9) => {
            "M-{}" => focus_workspace (REF);
            "M-S-{}" => client_to_workspace (REF);
        };
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    wm.grab_keys_and_run(key_bindings, mouse_bindings)
}
