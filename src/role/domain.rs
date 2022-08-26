use strum_macros::EnumIter;

pub struct ApplicationRole {
    pub name: &'static str,
    pub value: &'static str,
    pub description: &'static str,
    pub permissions: &'static [ApplicationPermission],
}

pub static DEFAULT_ROLES: &'static [ApplicationRole] = &[
    ApplicationRole {
        name: "Admin",
        value: "admin",
        description: "Superadmin of the application",
        permissions: &[
            // Building
            ApplicationPermission::ViewListBuilding,
            ApplicationPermission::CreateBuilding,
            ApplicationPermission::UpdateBuilding,
            ApplicationPermission::DeleteBuilding,
            // Floor
            ApplicationPermission::ViewListFloor,
            ApplicationPermission::CreateFloor,
            ApplicationPermission::UpdateFloor,
            ApplicationPermission::DeleteFloor,
            // Device
            ApplicationPermission::ViewListDevice,
            ApplicationPermission::ViewDeviceDetail,
            ApplicationPermission::CreateDevice,
            ApplicationPermission::UpdateDevice,
            ApplicationPermission::DeleteDevice,
            // Announcement
            ApplicationPermission::ViewListAnnouncement,
            ApplicationPermission::ViewAnnouncementDetail,
            ApplicationPermission::ViewAnnouncementMedia,
            ApplicationPermission::CreateAnnouncement,
            // Request
            ApplicationPermission::ViewListRequest,
            ApplicationPermission::UpdateRequestApproval,
            // User
            ApplicationPermission::ViewListUser,
            ApplicationPermission::UpdateUserApproval,
        ],
    },
    ApplicationRole {
        name: "LSC",
        value: "lsc",
        description: "Lecturer Service Center",
        permissions: &[
            // Building
            ApplicationPermission::ViewListBuilding,
            // Floor
            ApplicationPermission::ViewListFloor,
            // Device
            ApplicationPermission::ViewListDevice,
            ApplicationPermission::ViewDeviceDetail,
            // Announcement
            ApplicationPermission::ViewListAnnouncement,
            ApplicationPermission::ViewAnnouncementDetail,
            ApplicationPermission::ViewAnnouncementMedia,
            ApplicationPermission::CreateAnnouncement,
            // Request
            ApplicationPermission::ViewListRequest,
            ApplicationPermission::UpdateRequestApproval,
        ],
    },
    ApplicationRole {
        name: "BM",
        value: "bm",
        description: "Building Management",
        permissions: &[
            // Building
            ApplicationPermission::ViewListBuilding,
            // Floor
            ApplicationPermission::ViewListFloor,
            // Device
            ApplicationPermission::ViewListDevice,
            ApplicationPermission::ViewDeviceDetail,
            // Announcement
            ApplicationPermission::ViewListAnnouncement,
            ApplicationPermission::ViewAnnouncementDetail,
            ApplicationPermission::ViewAnnouncementMedia,
            ApplicationPermission::CreateAnnouncement,
            // Request
            ApplicationPermission::ViewListRequest,
            ApplicationPermission::UpdateRequestApproval,
        ],
    },
    ApplicationRole {
        name: "SSC",
        value: "ssc",
        description: "Student Services Center",
        permissions: &[
            // Building
            ApplicationPermission::ViewListBuilding,
            // Floor
            ApplicationPermission::ViewListFloor,
            // Device
            ApplicationPermission::ViewListDevice,
            ApplicationPermission::ViewDeviceDetail,
            // Announcement
            ApplicationPermission::ViewListAnnouncement,
            ApplicationPermission::ViewAnnouncementDetail,
            ApplicationPermission::ViewAnnouncementMedia,
            ApplicationPermission::CreateAnnouncement,
            // Request
            ApplicationPermission::ViewListRequest,
        ],
    },
];

#[derive(EnumIter, Debug, PartialEq, Clone)]
pub enum ApplicationPermission {
    // Building
    ViewListBuilding,
    CreateBuilding,
    UpdateBuilding,
    DeleteBuilding,
    // Floor
    ViewListFloor,
    CreateFloor,
    UpdateFloor,
    DeleteFloor,
    // Device
    ViewListDevice,
    ViewDeviceDetail,
    CreateDevice,
    UpdateDevice,
    DeleteDevice,
    // Announcement
    ViewListAnnouncement,
    ViewAnnouncementDetail,
    ViewAnnouncementMedia,
    CreateAnnouncement,
    // Request
    ViewListRequest,
    UpdateRequestApproval,
    // User
    ViewListUser,
    UpdateUserApproval,
}

impl ApplicationPermission {
    pub fn label(&self) -> &'static str {
        match self {
            ApplicationPermission::ViewListBuilding => "View List Building",
            ApplicationPermission::CreateBuilding => "Create Building",
            ApplicationPermission::UpdateBuilding => "Update Building",
            ApplicationPermission::DeleteBuilding => "Delete Building",
            ApplicationPermission::ViewListFloor => "View List Floor",
            ApplicationPermission::CreateFloor => "Create Floor",
            ApplicationPermission::UpdateFloor => "Update Floor",
            ApplicationPermission::DeleteFloor => "Delete Floor",
            ApplicationPermission::ViewListDevice => "View List Device",
            ApplicationPermission::ViewDeviceDetail => "View Device Detail",
            ApplicationPermission::CreateDevice => "Create Device Detail",
            ApplicationPermission::UpdateDevice => "Update Device",
            ApplicationPermission::DeleteDevice => "Delete Device",
            ApplicationPermission::ViewListAnnouncement => "View List Announcement",
            ApplicationPermission::ViewAnnouncementDetail => "View Announcement Detail",
            ApplicationPermission::ViewAnnouncementMedia => "View Announcement Media",
            ApplicationPermission::CreateAnnouncement => "Create Announcement",
            ApplicationPermission::ViewListRequest => "View List Request",
            ApplicationPermission::UpdateRequestApproval => "Update Request Approval",
            ApplicationPermission::ViewListUser => "View List User",
            ApplicationPermission::UpdateUserApproval => "Update User Approval",
        }
    }

    pub fn value(&self) -> &'static str {
        match self {
            ApplicationPermission::ViewListBuilding => "view_list_building",
            ApplicationPermission::CreateBuilding => "create_building",
            ApplicationPermission::UpdateBuilding => "update_building",
            ApplicationPermission::DeleteBuilding => "delete_building",
            ApplicationPermission::ViewListFloor => "view_list_Floor",
            ApplicationPermission::CreateFloor => "create_floor",
            ApplicationPermission::UpdateFloor => "update_floor",
            ApplicationPermission::DeleteFloor => "delete_floor",
            ApplicationPermission::ViewListDevice => "view_list_device",
            ApplicationPermission::ViewDeviceDetail => "view_device_detail",
            ApplicationPermission::CreateDevice => "create_device",
            ApplicationPermission::UpdateDevice => "update_device",
            ApplicationPermission::DeleteDevice => "delete_device",
            ApplicationPermission::ViewListAnnouncement => "view_list_announcement",
            ApplicationPermission::ViewAnnouncementDetail => "view_announcement_detail",
            ApplicationPermission::ViewAnnouncementMedia => "view_announcement_media",
            ApplicationPermission::CreateAnnouncement => "create_announcement",
            ApplicationPermission::ViewListRequest => "view_list_request",
            ApplicationPermission::UpdateRequestApproval => "update_request_approval",
            ApplicationPermission::ViewListUser => "view_list_user",
            ApplicationPermission::UpdateUserApproval => "update_user_approval",
        }
    }
}

pub struct RoleObject {
    pub name: &'static str,
    pub value: &'static str,
    pub description: &'static str,
    pub permissions: Vec<PermissionObject>,
}

pub struct PermissionObject {
    pub label: &'static str,
    pub value: &'static str,
}

pub enum GetRoleByNameError {
    RoleNotFound(&'static str),
}

impl std::fmt::Display for GetRoleByNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetRoleByNameError::RoleNotFound(message) => write!(f, "{}", message),
        }
    }
}
