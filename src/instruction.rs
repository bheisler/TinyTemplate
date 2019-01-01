pub(crate) type Path<'template> = Vec<&'template str>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum Instruction<'template> {
    Literal(&'template str),
    Value(Path<'template>),
    Branch { path: Path<'template>, invert: bool, taken: usize},
}