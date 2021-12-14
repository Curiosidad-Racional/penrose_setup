use penrose;
use tracing::error;
use neovim_lib::{Neovim, NeovimApi, Session};

pub fn command(cmd: &str) -> penrose::Result<()>
{
    match std::fs::read_dir("/tmp") {
        Ok(dir) => dir
            .filter_map(|f| f.ok())
            .filter(|f| {
                let is_dir =
                    matches!(f.file_type().map(|t| t.is_dir()), Ok(true));
                let name_heuristic =
                    f.file_name().to_string_lossy().starts_with("nvim");
                is_dir && name_heuristic
            })
            .filter_map(|dir| {
                Some(
                    std::fs::read_dir(dir.path())
                        .ok()?
                        .filter_map(Result::ok)
                        .map(|d| d.path()),
                )
            })
            .flatten()
            .filter_map(|d| Session::new_unix_socket(d).ok())
            .map(|mut session| {
                session.start_event_loop();
                Neovim::new(session)
            })
            .for_each(|mut nvim| {
                let _ = nvim
                    .command(&cmd)
                    .map_err(|e| error!("Error: {}", e));
            }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => error!("/tmp not found: {}", e),
        Err(e) => error!("/tmp error: {}", e),
    };
    Ok(())
}
