macro_rules! __impl_to_string {
    (
        $v:vis enum $t:ident {$(
            $i:ident ($inner:ty)
        ),+ $(,)?}
    ) => {
        $v enum $t {
            $($i ($inner)),+
        }

        impl From<$t> for String {
            fn from(t: $t) -> Self {
                use $t::*;

                (match t {
                    $(
                        $i(a) => a.into()
                    ),+
                })
            }
        }
    };
}

__impl_to_string! {
    pub enum Topic {
        Coordinator(CoordinatorTopic),
    }
}


pub enum CoordinatorTopic {
    Root,
    RegisterNode
}

// TODO: put these in a macro of some sort.

impl From<CoordinatorTopic> for String {
    fn from(t: CoordinatorTopic) -> Self {
        use CoordinatorTopic::*;

        (match t {
            Root => "coordinator",
            RegisterNode => "coordinator/register_node"
        })
        .into()
    }
}