#[derive(Debug, Clone)]
pub struct AssignmentListItemVm {
    pub title: String,
    pub description: String,
    pub due_label: String,
    pub attachment_count: usize,
}
