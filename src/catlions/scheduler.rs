// Let the application release an SGA before its new coroutine first polls.
// This keeps uniquely owned SDDF TX buffers eligible for direct submission.
pub const RUN_NEW_TASKS_IMMEDIATELY: bool = false;
