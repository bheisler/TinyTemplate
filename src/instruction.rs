pub(crate) type Path<'template> = Vec<&'template str>;
pub(crate) type PathSlice<'a, 'template> = &'a [&'template str];

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum Instruction<'template> {
    Literal(&'template str),
    Value(Path<'template>),
    Branch(Path<'template>, usize),
    PushContext(Path<'template>),
    PushNamedContext(Path<'template>, &'template str),
    PushIterationContext(Path<'template>, &'template str),
    PopContext,
    Iterate(usize),
    Goto(usize),
}
