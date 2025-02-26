use std::cmp;
use std::sync::LazyLock;
use sysinfo::System;

/// Calculate the render process limits.
fn calculate_max_renderer_process_hosts() -> u64 {
    const ESTIMATED_WEB_CONTENTS_MEMORY_USAGE_MB: u64 = if cfg!(target_pointer_width = "64") {
        85
    } else {
        60
    };

    const MIN_RENDERER_PROCESS_COUNT: u64 = 3;

    let max_renderer_process_count_platform: u64 = get_platform_max_renderer_process_count();

    let sys = System::new_all();
    let total_memory_mb = sys.total_memory() / 1024; // Convert KB to MB

    let mut max_count = total_memory_mb / 2;

    max_count /= ESTIMATED_WEB_CONTENTS_MEMORY_USAGE_MB;
    max_count = cmp::max(max_count, MIN_RENDERER_PROCESS_COUNT);
    max_count = cmp::min(max_count, max_renderer_process_count_platform);

    max_count
}

/// The platform max render count.
fn get_platform_max_renderer_process_count() -> u64 {
    let platform_limit = get_platform_process_limit();
    if platform_limit != u64::MAX {
        platform_limit / 2
    } else {
        82
    }
}

/// Platform render process limit.
fn get_platform_process_limit() -> u64 {
    (num_cpus::get() * 10).try_into().unwrap_or(82)
}

/// The renderer process limit.
pub(crate) static RENDER_PROCESS_LIMIT: LazyLock<String> = LazyLock::new(|| {
    format!(
        "--renderer-process-limit={}",
        calculate_max_renderer_process_hosts()
    )
});
