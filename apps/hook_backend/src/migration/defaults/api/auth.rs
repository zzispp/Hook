use super::ApiDefinition;

pub const AUTH_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "auth_me",
        method: "GET",
        path_pattern: "/api/auth/me",
        name: "Current user",
        group: "Auth",
    },
    ApiDefinition {
        code: "navbar_read",
        method: "GET",
        path_pattern: "/api/navbar",
        name: "Navbar",
        group: "System",
    },
];
