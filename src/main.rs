mod platform;

use crate::platform::{
    ACTION_CLEAR_ALL, ACTION_CLOSE, CONTAINER_ROLES, NOTIFICATION_SUBROLES, Platform,
};
use accessibility_sys::AXUIElementRef;
use std::error::Error;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
use core_foundation::base::{CFType, TCFType};
use log::{debug, info};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn find_notification_alerts(elements: &[CFType]) -> Vec<CFType> {
    let mut alerts = Vec::new();

    for element in elements {
        let element_ref = element.as_CFTypeRef() as AXUIElementRef;
        let role = match Platform::get_role(element_ref) {
            Some(r) => r,
            None => continue,
        };

        let subrole = Platform::get_subrole(element_ref);

        if let Some(ref sr) = subrole {
            if NOTIFICATION_SUBROLES.contains(&sr.as_str()) {
                debug!("Found notification alert: {}", sr);
                alerts.push(element.clone());
                continue;
            }
        }

        if CONTAINER_ROLES.contains(&role.as_str()) {
            debug!("Entering container: {}", role);
            if let Some(children) = Platform::get_children(element_ref) {
                alerts.extend(find_notification_alerts(&children));
            }
        }
    }

    alerts
}

fn get_notification_center_groups() -> Result<Vec<CFType>> {
    let pid = Platform::get_notification_center_pid()?;
    debug!("NotificationCenter PID: {}", pid);

    let app_element = Platform::create_app_element(pid)?;
    let app_element_ref = app_element.as_CFTypeRef() as AXUIElementRef;

    let windows = Platform::get_window_list(app_element_ref)?;
    debug!("Found {} windows", windows.len());

    let window = match windows.into_iter().next() {
        Some(item) => item,
        None => {
            info!("No NotificationCenter windows found");
            return Ok(vec![]);
        }
    };

    let notification_elements = match (|| {
        let window_ref = window.as_CFTypeRef() as AXUIElementRef;
        let window_children = Platform::get_children(window_ref)?;
        let first_child = window_children.first()?;
        let second_level_children =
            Platform::get_children(first_child.as_CFTypeRef() as AXUIElementRef)?;
        let second_child = second_level_children.first()?;
        Platform::get_children(second_child.as_CFTypeRef() as AXUIElementRef)
    })() {
        Some(e) => e,
        None => {
            info!("Could not navigate to notification elements");
            return Ok(vec![]);
        }
    };

    Ok(find_notification_alerts(&notification_elements))
}

fn close_batch_groups(groups: &[CFType]) -> usize {
    let start = Instant::now();
    let closed = AtomicUsize::new(0);

    std::thread::scope(|s| {
        for group in groups.iter().rev() {
            let group_ptr = group.as_CFTypeRef() as usize;
            let closed = &closed;

            // Each group runs in a thread since reading and performing actions is blocking and
            // running everything sequentially can be significantly slower
            s.spawn(move || {
                let group = group_ptr as AXUIElementRef;
                info!(
                    "Closing notification with {}...",
                    start.elapsed().as_millis()
                );
                let Some(actions) = Platform::get_actions(group) else {
                    return;
                };

                for action in &actions {
                    if is_close_action(action) {
                        if Platform::perform_action(group, action) {
                            closed.fetch_add(1, Ordering::SeqCst);
                        }
                        break;
                    }
                }
            });
        }
    });

    closed.into_inner()
}

fn is_close_action(action: &String) -> bool {
    action.contains(&format!("Name:{}", ACTION_CLOSE))
        || action.contains(&format!("Name:{}", ACTION_CLEAR_ALL))
}

fn main() -> Result<()> {
    env_logger::init();
    let start_time = Instant::now();

    let all_groups = get_notification_center_groups()?;

    if all_groups.is_empty() {
        info!("No notifications found...");
        return Ok(());
    }

    info!(
        "Found {} notification group(s), closing...",
        all_groups.len()
    );

    let closed = close_batch_groups(&all_groups);
    let elapsed = start_time.elapsed().as_millis();

    info!("Closed {} notification(s) in {}ms", closed, elapsed);

    Ok(())
}
