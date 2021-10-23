use penrose::{
    core::{
        data_types::Region,
        hooks::Hook,
        manager::WindowManager,
        helpers::{spawn, spawn_with_args},
        xconnection::{XConn, Xid},
        ring::Selector,
    },
    Result,
};

pub struct CustomHook {
    conn: xcb::Connection,
    class_names_center: Vec<String>,
    class_names_opaque: Vec<String>,
    opacity: f32
}

impl CustomHook {
    pub fn new(class_names_center: &Vec<String>,
               class_names_opaque: &Vec<String>,
               opacity: f32) -> Box<Self> {
        let (conn, _) = xcb::Connection::connect(None).unwrap();
        Box::new(Self {
            conn,
            class_names_center: class_names_center.into_iter().map(|c| c.into()).collect(),
            class_names_opaque: class_names_opaque.into_iter().map(|c| c.into()).collect(),
            opacity
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

impl<X: XConn> Hook<X> for CustomHook {
    fn startup(&mut self, wm: &mut WindowManager<X>) -> Result<()> {
        let _ = wm.set_root_window_name("root");
        let _ = spawn("lbarstat");
        Ok(())
    }

    fn new_client(&mut self, wm: &mut WindowManager<X>, id: Xid) -> Result<()> {
        let client_class = match wm.client(&Selector::WinId(id)) {
            Some(client) => client.wm_class().to_string(),
            None => return Ok(())
        };
        if !self.class_names_opaque.contains(&client_class)
        {
            let _ = spawn_with_args("transset", &["--id", &id.to_string(), &self.opacity.to_string()]);
        }

        if self.class_names_center.contains(&client_class)
        {
            let _ = self.centered_above(id, wm);
        }

        Ok(())
    }
}
