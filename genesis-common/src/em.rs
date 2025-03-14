#[repr(i32)]
pub enum TaskStatusEnum {
    Init = 0,
    Error = 1,
    Success = 2,
    ManualStop = 3,
}
