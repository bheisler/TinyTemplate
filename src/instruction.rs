pub(crate) type Path<'template> = Vec<&'template str>;
pub(crate) type PathSlice<'a, 'template> = &'a [&'template str];

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum Instruction<'template> {
    Literal(&'template str),
    Value(Path<'template>),
    FormattedValue(Path<'template>, &'template str),
    Branch(Path<'template>, usize),
    PushContext(Path<'template>),
    PushNamedContext(Path<'template>, &'template str),
    PushIterationContext(Path<'template>, &'template str),
    PopContext,
    Iterate(usize),
    Goto(usize),
    Call(&'template str, Path<'template>),
}

pub(crate) fn path_to_str(path: PathSlice) -> String {
    let mut path_str = "".to_string();
    for (i, step) in path.iter().enumerate() {
        path_str.push_str(step);
        if i > 0 {
            path_str.push('.');
        }
    }
    path_str
}
