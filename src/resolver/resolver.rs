use crate::command::Target;

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub name: String,
}

pub struct Resolver;

impl Resolver {

    pub fn resolve(
        target: Target,
    ) -> ResolvedTarget {

        ResolvedTarget {
            name: target.raw,
        }

    }

}