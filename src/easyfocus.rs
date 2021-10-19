extern crate xcb;

use penrose::core::{
    data_types::WinId,
    manager::WindowManager,
    xconnection::XConn,
    ring::Selector,
};
use std::{
    collections::HashMap,
    thread, time,
};


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
              (xcb::xproto::CW_EVENT_MASK,
               xcb::xproto::EVENT_MASK_EXPOSURE)]
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


pub fn easyfocus<X>(wm: &mut WindowManager<X>) -> penrose::Result<()>
where
    X: XConn,
{
    let focused_client_id = wm.focused_client_id();
    let (conn, screen_i) = match xcb::Connection::connect(None)
    {
        Ok(x) => x,
        Err(_) => return Ok(()),
    };
    let root = match conn.get_setup().roots().nth(
        screen_i as usize)
    {
        Some(screen) => screen.root(),
        None => return Ok(()),
    };
    {
        let mut iters = 50;
        let millis = time::Duration::from_millis(10);
        loop
        {
            if let Ok(reply) = xcb::grab_keyboard(
                &conn,
                true,
                root,
                xcb::CURRENT_TIME,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            ).get_reply() {
                if reply.status() == xcb::GRAB_STATUS_SUCCESS as u8 {
                    break;
                }
            };
            thread::sleep(millis);
            iters -= 1;
            if iters <= 0
            {
                return Ok(());
            }
        }
    }
    xcb::xproto::set_input_focus(
        &conn,
        xcb::xproto::INPUT_FOCUS_POINTER_ROOT as u8,
        root,
        xcb::CURRENT_TIME
    );
    conn.flush();

    let mut letters = String::from("0987654321nbmvcxzytpoiurewqhglkjfdsa");
    let mut window_text_ids: Vec<WinId> = vec![];
    let mut client_ids:HashMap<char, WinId> = HashMap::new();
    for workspace_index in wm.focused_workspaces().iter()
    {
        if let Some(workspace) = wm.workspace(
            &Selector::Index(*workspace_index))
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
    let mut choice_result = None;
    loop {
        let ev = conn.wait_for_event();
        if let Some(ev) = ev {
            match ev.response_type() & 0x7F {
                xcb::KEY_PRESS => {
                    let key: &xcb::KeyPressEvent = unsafe {
                        xcb::cast_event(&ev)
                    };
                    match key.detail() {
                        9 => {break;},
                        38 => choice_result = Some('a'),
                        56 => choice_result = Some('b'),
                        54 => choice_result = Some('c'),
                        40 => choice_result = Some('d'),
                        26 => choice_result = Some('e'),
                        41 => choice_result = Some('f'),
                        42 => choice_result = Some('g'),
                        43 => choice_result = Some('h'),
                        31 => choice_result = Some('i'),
                        44 => choice_result = Some('j'),
                        45 => choice_result = Some('k'),
                        46 => choice_result = Some('l'),
                        58 => choice_result = Some('m'),
                        57 => choice_result = Some('n'),
                        32 => choice_result = Some('o'),
                        33 => choice_result = Some('p'),
                        24 => choice_result = Some('q'),
                        27 => choice_result = Some('r'),
                        39 => choice_result = Some('s'),
                        28 => choice_result = Some('t'),
                        30 => choice_result = Some('u'),
                        55 => choice_result = Some('v'),
                        25 => choice_result = Some('w'),
                        53 => choice_result = Some('x'),
                        29 => choice_result = Some('y'),
                        52 => choice_result = Some('z'),
                        10 => choice_result = Some('1'),
                        11 => choice_result = Some('2'),
                        12 => choice_result = Some('3'),
                        13 => choice_result = Some('4'),
                        14 => choice_result = Some('5'),
                        15 => choice_result = Some('6'),
                        16 => choice_result = Some('7'),
                        17 => choice_result = Some('8'),
                        18 => choice_result = Some('9'),
                        19 => choice_result = Some('0'),
                        k => {
                            println!("ev key {}", k);
                            continue;
                        }
                    }
                    break;
                }
                xcb::EXPOSE => {
                    conn.flush();
                }
                xcb::MAPPING_NOTIFY => {
                    conn.flush();
                    break;
                }
                xcb::KEY_RELEASE => {
                    conn.flush();
                }
                _ => {
                    println!("ev code {}", ev.response_type());
                    conn.flush();
                }
            }
        } else {
            break;
        }
    }
    {
        let mut iters = 50;
        let millis = time::Duration::from_millis(10);
        loop
        {
            if let Ok(_) = xcb::ungrab_keyboard_checked(
                &conn, xcb::CURRENT_TIME).request_check()
            {
                break;
            }
            thread::sleep(millis);
            iters -= 1;
            if iters <= 0
            {
                return Ok(());
            }
        }
    }
    for window_text_id in window_text_ids.iter()
    {
        xcb::xproto::destroy_window(&conn, *window_text_id);
    }
    if let Some(ch) = choice_result {
        if let Some(&client_id) = client_ids.get(&ch) {
            let _ = wm.focus_client(&Selector::WinId(client_id));
            return Ok(());
        }
    }
    if let Some(client_id) = focused_client_id {
        let _ = wm.focus_client(&Selector::WinId(client_id));
    }
    Ok(())
}