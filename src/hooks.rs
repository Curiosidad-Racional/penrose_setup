use penrose::{
    core::{
        data_types::Region,
        hooks::Hook,
        manager::WindowManager,
        helpers::spawn,
        xconnection::{XConn, Xid},
        ring::Selector,
    },
    Result,
};

pub struct CenterFloat {
    conn: xcb::Connection,
    class_names: Vec<String>,
}

impl CenterFloat {
    pub fn new(class_names: Vec<impl Into<String>>) -> Box<Self> {
        let (conn, _) = xcb::Connection::connect(None).unwrap();
        Box::new(Self {
            conn,
            class_names: class_names.into_iter().map(|c| c.into()).collect(),
        })
    }

    fn centered_above<X: XConn>(&self, id: Xid, wm: &mut WindowManager<X>) -> Result<()> {
        if let Some(region) = wm.screen_size(wm.active_screen_index())
        {
            if let Ok(res) = xcb::get_geometry(&self.conn, id).get_reply()
            {
                let w = res.width() as u32;
                let h = res.height() as u32;
                wm.position_client(id, Region::new(
                    region.x + (region.w - w) / 2,
                    region.y + (region.h - h) / 2,
                    w,
                    h,
                ), true)?;
            }
        }
        wm.show_client(id)
    }
}

impl<X: XConn> Hook<X> for CenterFloat {
    fn new_client(&mut self, wm: &mut WindowManager<X>, id: Xid) -> Result<()> {
        if self.class_names.contains(&(wm.client(&Selector::WinId(id)).unwrap()).wm_class().to_string()) {
            self.centered_above(id, wm)?;
        }

        Ok(())
    }
}

pub struct StartupScript {
}

impl StartupScript {
    pub fn new() -> Self {
        Self {}
    }
}

impl<X: XConn> Hook<X> for StartupScript {
    fn startup(&mut self, wm: &mut WindowManager<X>) -> Result<()> {
        let _ = wm.set_root_window_name("root");
        let _ = spawn("lbarstat");
        Ok(())
    }
}
