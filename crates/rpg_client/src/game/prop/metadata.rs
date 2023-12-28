use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Metadata {
    props: HashMap<&'static str, &'static str>,
}
