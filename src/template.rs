use instruction::Instruction;

pub(crate) struct Template<'template> {
    instructions: Vec<Instruction<'template>>,
    template_len: usize,
}