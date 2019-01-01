pub(crate) type Path<'template> = Vec<&'template str>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) struct Branch<'template> {
    pub path: Path<'template>,
    pub invert: bool,
    pub target: usize,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum Instruction<'template> {
    Literal(&'template str),
    Value(Path<'template>),
    Branch(Branch<'template>),
    PushContext(Path<'template>),
    PopContext,
}
