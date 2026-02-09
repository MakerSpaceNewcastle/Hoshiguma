#[macro_export]
macro_rules! define_message {
    ($name:ident, (), $id:expr) => {
        #[derive(Debug, defmt::Format, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $name;

        impl $crate::MessagePayload for $name {
            const ID: &'static $crate::MessageId = $id;
        }
    };
    ($name:ident, ( $($fields:tt)* ), $id:expr) => {
        #[derive(Debug, defmt::Format, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $name(
            $($fields)*
        );

        impl $crate::MessagePayload for $name {
            const ID: &'static $crate::MessageId = $id;
        }
    };
    ($name:ident, { $($fields:tt)* }, $id:expr) => {
        #[derive(Debug, defmt::Format, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $($fields)*
        }

        impl $crate::MessagePayload for $name {
            const ID: &'static $crate::MessageId = $id;
        }
    };
}

#[macro_export]
macro_rules! define_request_response {
    ($req:ty, $res:ty) => {
        impl $crate::ExpectedResponse for $req {
            type Response = $res;
        }
    };
}

#[macro_export]
macro_rules! basic_state_response_verification {
    ($req:ty, $res:ty) => {
        impl $crate::ResponseVerification<$res> for $req {
            fn verify_response(&self, response: &$res) -> bool {
                self.0 == response.0
            }
        }
    };
}

#[macro_export]
macro_rules! falible_basic_state_response_verification {
    ($req:ty, $res:ty) => {
        impl $crate::ResponseVerification<$res> for $req {
            fn verify_response(&self, response: &$res) -> bool {
                Ok::<_, &()>(&self.0) == response.0.as_ref()
            }
        }
    };
}

pub trait ExpectedResponse {
    type Response;
}

pub trait ResponseVerification<Response> {
    fn verify_response(&self, response: &Response) -> bool;
}
