use super::ApiDefinition;

pub const RECHARGE_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "recharge_packages_read",
        method: "GET",
        path_pattern: "/api/recharge-packages",
        name: "用户充值套餐列表",
    },
    ApiDefinition {
        code: "recharge_orders_read",
        method: "GET",
        path_pattern: "/api/recharge-orders",
        name: "用户充值订单列表",
    },
    ApiDefinition {
        code: "recharge_orders_create",
        method: "POST",
        path_pattern: "/api/recharge-orders",
        name: "创建用户充值订单",
    },
    ApiDefinition {
        code: "payment_channels_read",
        method: "GET",
        path_pattern: "/api/payment-channels",
        name: "用户支付渠道列表",
    },
    ApiDefinition {
        code: "admin_recharge_packages_read",
        method: "GET",
        path_pattern: "/api/admin/recharge-packages",
        name: "充值套餐列表",
    },
    ApiDefinition {
        code: "admin_recharge_packages_create",
        method: "POST",
        path_pattern: "/api/admin/recharge-packages",
        name: "创建充值套餐",
    },
    ApiDefinition {
        code: "admin_recharge_packages_update",
        method: "PATCH",
        path_pattern: "/api/admin/recharge-packages/{id}",
        name: "更新充值套餐",
    },
    ApiDefinition {
        code: "admin_recharge_orders_read",
        method: "GET",
        path_pattern: "/api/admin/recharge-orders",
        name: "充值订单列表",
    },
    ApiDefinition {
        code: "admin_payment_callbacks_read",
        method: "GET",
        path_pattern: "/api/admin/payment-callbacks",
        name: "支付回调记录列表",
    },
    ApiDefinition {
        code: "admin_payment_channels_read",
        method: "GET",
        path_pattern: "/api/admin/payment-channels",
        name: "支付渠道列表",
    },
    ApiDefinition {
        code: "admin_payment_channels_update",
        method: "PATCH",
        path_pattern: "/api/admin/payment-channels/{code}",
        name: "更新支付渠道",
    },
];
