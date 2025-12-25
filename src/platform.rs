use accessibility_sys::{
    AXUIElementCopyActionNames, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementPerformAction, AXUIElementRef,
};
use std::error::Error;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFType, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use std::ffi::c_void;
use std::ptr;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

// MARK: - Constants
pub const ROLE_GROUP: &str = "AXGroup";
pub const ROLE_SCROLL_AREA: &str = "AXScrollArea";
pub const ROLE_LIST: &str = "AXList";
pub const ROLE_SPLIT_GROUP: &str = "AXSplitGroup";

pub const SUBROLE_NOTIFICATION_ALERT: &str = "AXNotificationCenterAlert";
pub const SUBROLE_NOTIFICATION_ALERT_STACK: &str = "AXNotificationCenterAlertStack";

pub const ACTION_CLOSE: &str = "Close";
pub const ACTION_CLEAR_ALL: &str = "Clear All";

pub const CONTAINER_ROLES: &[&str] = &[ROLE_GROUP, ROLE_SCROLL_AREA, ROLE_LIST, ROLE_SPLIT_GROUP];
pub const NOTIFICATION_SUBROLES: &[&str] =
    &[SUBROLE_NOTIFICATION_ALERT, SUBROLE_NOTIFICATION_ALERT_STACK];

/// All `unsafe` code is centralized here to interact with the macOS Accessibility (ApplicationServices) C APIs.
/// These APIs use raw pointers and Core Foundation reference counting rules (Create and Get rules).
/// We use `CFType` to safely wrap these pointers once obtained.
pub struct Platform;

impl Platform {
    pub fn get_notification_center_pid() -> Result<i32> {
        let mut sys = System::new_with_specifics(
            sysinfo::RefreshKind::new().with_processes(ProcessRefreshKind::new()),
        );
        sys.refresh_processes(ProcessesToUpdate::All);

        let nc_process = sys.processes().values().find(|p| {
            let name = p.name().to_string_lossy();
            name == "NotificationCenter"
        });

        match nc_process {
            Some(p) => Ok(p.pid().as_u32() as i32),
            None => Err(format!("Could not find NotificationCenter process").into()),
        }
    }

    pub fn create_app_element(pid: i32) -> Result<CFType> {
        unsafe {
            let app_element = AXUIElementCreateApplication(pid);
            if app_element.is_null() {
                return Err(format!("Failed to create AXUIElement for pid {}", pid).into());
            }
            Ok(CFType::wrap_under_create_rule(app_element as *const c_void))
        }
    }

    pub fn get_attribute(element_ref: AXUIElementRef, attribute_name: &str) -> Option<CFType> {
        let attribute = CFString::new(attribute_name);
        let mut value: *const c_void = ptr::null();
        unsafe {
            let result = AXUIElementCopyAttributeValue(
                element_ref,
                attribute.as_concrete_TypeRef(),
                &mut value,
            );
            if result == 0 && !value.is_null() {
                Some(CFType::wrap_under_create_rule(value))
            } else {
                None
            }
        }
    }

    pub fn get_window_list(app_element_ref: AXUIElementRef) -> Result<Vec<CFType>> {
        let windows_attr = match Self::get_attribute(app_element_ref, "AXWindows") {
            Some(w) => w,
            None => return Ok(vec![]),
        };

        unsafe {
            let array_ref = windows_attr.as_CFTypeRef() as CFArrayRef;
            let array: CFArray<CFType> = CFArray::wrap_under_get_rule(array_ref);
            Ok(array.into_iter().map(|item| item.clone()).collect())
        }
    }

    pub fn get_children(element_ref: AXUIElementRef) -> Option<Vec<CFType>> {
        Self::get_attribute(element_ref, "AXChildren").map(|t| unsafe {
            let array_ref = t.as_CFTypeRef() as CFArrayRef;
            let array: CFArray<CFType> = CFArray::wrap_under_get_rule(array_ref);
            array.into_iter().map(|item| item.clone()).collect()
        })
    }

    pub fn get_role(element_ref: AXUIElementRef) -> Option<String> {
        Self::get_attribute(element_ref, "AXRole").map(|t| unsafe {
            CFString::wrap_under_get_rule(t.as_CFTypeRef() as CFStringRef).to_string()
        })
    }

    pub fn get_subrole(element_ref: AXUIElementRef) -> Option<String> {
        Self::get_attribute(element_ref, "AXSubrole").map(|t| unsafe {
            CFString::wrap_under_get_rule(t.as_CFTypeRef() as CFStringRef).to_string()
        })
    }

    pub fn get_actions(element_ref: AXUIElementRef) -> Option<Vec<String>> {
        let mut actions: CFArrayRef = ptr::null();
        unsafe {
            let result = AXUIElementCopyActionNames(element_ref, &mut actions);
            if result == 0 && !actions.is_null() {
                let array: CFArray<CFString> = CFArray::wrap_under_create_rule(actions);
                Some(array.into_iter().map(|s| s.to_string()).collect())
            } else {
                None
            }
        }
    }

    pub fn perform_action(element_ref: AXUIElementRef, action: &str) -> bool {
        let action_name = CFString::new(action);
        unsafe {
            let result = AXUIElementPerformAction(element_ref, action_name.as_concrete_TypeRef());
            result == 0
        }
    }
}
