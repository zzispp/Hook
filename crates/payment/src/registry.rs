use std::sync::Arc;

use crate::{PaymentCallbackEndpoint, PaymentCallbackEndpointKind, PaymentChannelProvider, PaymentChannelRegistration, RegisteredPaymentCallbackEndpoint};

#[derive(Clone, Default)]
pub struct PaymentChannelRegistry {
    providers: Vec<Arc<dyn PaymentChannelProvider>>,
}

impl PaymentChannelRegistry {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_providers(providers: Vec<Arc<dyn PaymentChannelProvider>>) -> Self {
        Self { providers }
    }

    pub fn registrations(&self) -> Vec<PaymentChannelRegistration> {
        self.providers.iter().map(|provider| provider.registration()).collect()
    }

    pub fn provider(&self, code: &str) -> Option<Arc<dyn PaymentChannelProvider>> {
        self.providers.iter().find(|provider| provider.registration().code == code).cloned()
    }

    pub fn callback_endpoints(&self) -> Vec<RegisteredPaymentCallbackEndpoint> {
        self.registered_callback_endpoints()
    }

    pub fn registered_callback_endpoints(&self) -> Vec<RegisteredPaymentCallbackEndpoint> {
        self.providers
            .iter()
            .flat_map(|provider| {
                let registration = provider.registration();
                provider
                    .callback_endpoints()
                    .into_iter()
                    .map(move |endpoint| registered_endpoint(&registration.code, endpoint))
            })
            .collect()
    }

    pub fn callback_endpoint(&self, code: &str, kind: PaymentCallbackEndpointKind) -> Option<RegisteredPaymentCallbackEndpoint> {
        self.provider(code)?
            .callback_endpoints()
            .into_iter()
            .map(|endpoint| registered_endpoint(code, endpoint))
            .find(|endpoint| endpoint.kind == kind)
    }
}

fn registered_endpoint(channel_code: &str, endpoint: PaymentCallbackEndpoint) -> RegisteredPaymentCallbackEndpoint {
    RegisteredPaymentCallbackEndpoint {
        channel_code: channel_code.to_owned(),
        kind: endpoint.kind,
        methods: endpoint.methods,
        path_pattern: endpoint.path_pattern.replace("{code}", channel_code),
    }
}
