// Remove the Duration import since it's not used here
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

// DAS and ARR settings (in milliseconds)
pub const DAS_DELAY: u64 = 70;  // Delayed Auto Shift - delay before repeating
pub const ARR_DELAY: u64 = 10;   // Auto Repeat Rate - delay between repeats
pub const SOFT_DROP_DELAY: u64 = 0; // Instant soft drop for 40L
pub const KEY_TIMEOUT: u64 = 100; // Timeout for key release detection fallback

// 40L Sprint settings
pub const TARGET_LINES: u32 = 40;   // Lines to clear for 40L sprint
pub const GROUND_TIME: u64 = 500; // Time piece can stay on ground after soft drop (milliseconds)