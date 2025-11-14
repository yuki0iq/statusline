use crate::{Environment, Pretty};
use heck::ToPascalCase as _;
use linkme::distributed_slice;
use std::{collections::HashMap, sync::LazyLock};

mod build_info;
mod elapsed;
mod git;
mod hostuser;
mod jobs;
mod mail;
mod nix_shell;
mod return_code;
mod root_shell;
mod separator;
mod ssh;
mod time;
mod venv;
mod workdir;

pub trait Block: Pretty {
    fn new(environ: &Environment) -> Option<Box<dyn Block>>
    where
        Self: Sized;
    fn extend(&mut self) {}
}

type Constructor = fn(&Environment) -> Option<Box<dyn Block>>;
#[distributed_slice]
static BLOCK_KINDS: [(&str, Constructor)];

macro_rules! register_block {
    ($name:ident) => {
        const _: () = {
            #[linkme::distributed_slice($crate::block::BLOCK_KINDS)]
            static _BLOCK_KIND: (&str, $crate::block::Constructor) =
                (stringify!($name), <$name as $crate::block::Block>::new);
        };
    };
}
pub(crate) use register_block;

static BLOCK_KINDS_MAP: LazyLock<HashMap<&str, Constructor>> =
    LazyLock::new(|| BLOCK_KINDS.iter().copied().collect());

pub fn create_blocks(names: &[&str], environ: &Environment) -> Vec<Box<dyn Block>> {
    names
        .iter()
        .map(|name| name.to_pascal_case())
        .map(|name| {
            BLOCK_KINDS_MAP
                .get(&*name)
                .expect("block name should exist")
        })
        .filter_map(|cons| cons(environ))
        .collect()
}
