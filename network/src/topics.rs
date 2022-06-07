use pluto_macros::define_topics;

use crate::protos::auth::AuthNodeInit;

define_topics! {
    Coordinator {
        Auth -> "coordinator/auth" => AuthNodeInit
    },
    Node {
        Auth -> "node/{id}/auth"
    }
}
