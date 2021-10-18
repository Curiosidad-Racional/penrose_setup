use penrose::{
    core::{
        hooks::Hook,
        manager::WindowManager,
        helpers::spawn,
        xconnection::{XConn, Xid},
        ring::Selector,
    },
    Result,
};

pub struct CenterFloat {
    class_names: Vec<String>,
    scale: f64,
}

impl CenterFloat {
    pub fn new(class_names: Vec<impl Into<String>>, scale: f64) -> Box<Self> {
        Box::new(Self {
            class_names: class_names.into_iter().map(|c| c.into()).collect(),
            scale,
        })
    }

    fn centered_above<X: XConn>(&self, id: Xid, wm: &mut WindowManager<X>) -> Result<()> {
        if let Some(region) = wm.screen_size(wm.active_screen_index()) {
            let r = region.scale_w(self.scale).scale_h(self.scale);
            wm.position_client(id, r.centered_in(&region)?, true)?;
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
