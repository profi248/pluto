use pluto_macros::define_topics;

use crate::protos::auth::AuthNodeInit;

define_topics! {
    Coordinator {
        #[message = AuthNodeInit]
        Auth -> "coordinator/auth"
    },
    Node {
        Auth -> "node/{id}/auth"
    }
}