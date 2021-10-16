use penrose::core::{
    client::Client,
    data_types::{Region, ResizeAction, WinId},
};


fn dwindle_recurisive(
    clients: &[&Client],
    region: &Region,
    horizontal: bool,
    min_size: u32,
) -> Vec<ResizeAction> {
    if clients.len() > 1 {
        if region.w < min_size || region.h < min_size {
            clients
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        (c.id(), Some(*region))
                    } else {
                        (c.id(), None)
                    }
                })
                .collect()
        } else {
            let split = ((if horizontal { region.w } else { region.h } as f32) / 2.) as u32;
            let (main, other) = if horizontal {
                region.split_at_width(split)
            } else {
                region.split_at_height(split)
            }
            .unwrap();

            let mut vec =
                dwindle_recurisive(&clients[..clients.len() - 1], &other, !horizontal, min_size);
            vec.push((clients.last().unwrap().id(), Some(main)));
            vec
        }
    } else {
        clients
            .get(0)
            .map(|c| vec![(c.id(), Some(*region))])
            .unwrap_or(Vec::new())
    }
}

/**
 * A layout based on the dwindle layout from AwesomeWM.
 *
 * The second region is recursively split in two other regions, alternating between
 * splitting horizontally and vertically.
 */
pub fn dwindle(
    clients: &[&Client],
    _: Option<WinId>,
    monitor_region: &Region,
    _: u32,
    _: f32,
) -> Vec<ResizeAction> {
    dwindle_recurisive(clients, monitor_region, true, 50)
}
